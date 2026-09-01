use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use std::path::Path as StdPath;

use crate::{
    dtos::{
        classes::{Class, CreateClassParams, UpdateClassParams},
        common::Page,
    },
    models::_entities::{bookings, classes, users},
};

/// The caller's studio id. Every class read and write is scoped to it — a user
/// only ever sees and touches their own studio's classes.
async fn current_org_id(auth: &auth::JWT, ctx: &AppContext) -> Result<i64> {
    Ok(users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await?
        .organization_id)
}

/// Managing classes is a teacher (staff) action. Students get a 403. Returns
/// the caller's studio id so writes stay scoped to it.
async fn require_staff_org_id(auth: &auth::JWT, ctx: &AppContext) -> Result<i64> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_staff() {
        return Err(Error::CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail::with_reason("only teachers can manage classes"),
        ));
    }
    Ok(user.organization_id)
}

/// Load a class that belongs to `org_id`, or 404. A class in another studio is
/// indistinguishable from one that doesn't exist.
async fn load_item(ctx: &AppContext, id: i64, org_id: i64) -> Result<classes::Model> {
    classes::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .filter(|c| c.organization_id == org_id)
        .ok_or_else(|| Error::NotFound)
}

/// `starts_at` crosses the wire as an RFC 3339 string; a malformed value is the
/// caller's fault (400), not a server error.
fn parse_starts_at(value: &str) -> Result<DateTimeWithTimeZone> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| Error::BadRequest("starts_at must be an RFC 3339 datetime".to_string()))
}

/// Current booking count for a single class (for its `spots_left`).
async fn booked_count(ctx: &AppContext, class_id: i64) -> Result<i64> {
    Ok(bookings::Model::counts_by_class(&ctx.db, &[class_id])
        .await?
        .get(&class_id)
        .copied()
        .unwrap_or(0))
}

#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(pagination): Query<query::PaginationQuery>,
) -> Result<Response> {
    let org_id = current_org_id(&auth, &ctx).await?;
    let res = query::paginate(
        &ctx.db,
        classes::Entity::find().filter(classes::Column::OrganizationId.eq(org_id)),
        None,
        &pagination,
    )
    .await?;

    // One booking-count query for the whole page, then attach spots_left.
    let class_ids: Vec<i64> = res.page.iter().map(|c| c.id).collect();
    let counts = bookings::Model::counts_by_class(&ctx.db, &class_ids).await?;
    let items = res
        .page
        .into_iter()
        .map(|c| {
            let booked = counts.get(&c.id).copied().unwrap_or(0);
            Class::from_parts(c, booked)
        })
        .collect();

    format::json(Page {
        items,
        page: res.meta.page,
        page_size: res.meta.page_size,
        total_pages: res.meta.total_pages,
        total_items: res.meta.total_items,
    })
}

#[debug_handler]
async fn get_one(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let org_id = current_org_id(&auth, &ctx).await?;
    let item = load_item(&ctx, id, org_id).await?;
    let booked = booked_count(&ctx, item.id).await?;
    format::json(Class::from_parts(item, booked))
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateClassParams>,
) -> Result<Response> {
    let org_id = require_staff_org_id(&auth, &ctx).await?;
    let starts_at = parse_starts_at(&params.starts_at)?;

    // The studio is implicit — always the caller's — so it can't be spoofed.
    let item = classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set(params.title),
        instructor: ActiveValue::set(params.instructor),
        starts_at: ActiveValue::set(starts_at),
        duration_minutes: ActiveValue::set(params.duration_minutes),
        capacity: ActiveValue::set(params.capacity),
        price: ActiveValue::set(params.price),
        ..Default::default()
    };
    item.validate()?;
    let item = item.insert(&ctx.db).await?;
    // A brand-new class has no bookings yet.
    format::render()
        .status(StatusCode::CREATED)
        .json(Class::from_parts(item, 0))
}

#[debug_handler]
async fn update(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateClassParams>,
) -> Result<Response> {
    let org_id = require_staff_org_id(&auth, &ctx).await?;
    let starts_at = parse_starts_at(&params.starts_at)?;
    let mut item = load_item(&ctx, id, org_id).await?.into_active_model();
    item.title = ActiveValue::set(params.title);
    item.instructor = ActiveValue::set(params.instructor);
    item.starts_at = ActiveValue::set(starts_at);
    item.duration_minutes = ActiveValue::set(params.duration_minutes);
    item.capacity = ActiveValue::set(params.capacity);
    item.price = ActiveValue::set(params.price);
    item.validate()?;
    let item = item.update(&ctx.db).await?;
    let booked = booked_count(&ctx, item.id).await?;
    format::json(Class::from_parts(item, booked))
}

#[debug_handler]
async fn remove(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let org_id = require_staff_org_id(&auth, &ctx).await?;
    let item = load_item(&ctx, id, org_id).await?;
    classes::Entity::delete_by_id(item.id).exec(&ctx.db).await?;
    format::render().status(StatusCode::NO_CONTENT).empty()
}

fn content_type_for(key: &str) -> &'static str {
    match key.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

/// Upload (or replace) the instructor photo for a class. Teacher-only, scoped
/// to the caller's studio. The bytes go to the storage backend; the class keeps
/// only the key.
#[debug_handler]
async fn upload_photo(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let org_id = require_staff_org_id(&auth, &ctx).await?;
    let item = load_item(&ctx, id, org_id).await?;

    let mut uploaded: Option<(Bytes, String)> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        if field.file_name().is_some() {
            let filename = field.file_name().unwrap_or("photo").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| Error::BadRequest(e.to_string()))?;
            uploaded = Some((data, filename));
            break;
        }
    }
    let Some((data, filename)) = uploaded else {
        return bad_request("no file uploaded");
    };

    let ext = filename
        .rsplit('.')
        .next()
        .filter(|e| !e.is_empty() && e.len() <= 5)
        .unwrap_or("bin");
    let key = format!("class-photos/{id}-{}.{ext}", uuid::Uuid::new_v4());
    ctx.storage
        .upload(StdPath::new(&key), &data)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    let mut item = item.into_active_model();
    item.instructor_photo = ActiveValue::set(Some(key));
    let item = item.update(&ctx.db).await?;
    let booked = booked_count(&ctx, item.id).await?;
    format::json(Class::from_parts(item, booked))
}

/// Serve a class's instructor photo. Public (no auth) so an `<img>` tag can
/// load it — photos are meant to be seen.
#[debug_handler]
async fn serve_photo(Path(id): Path<i64>, State(ctx): State<AppContext>) -> Result<Response> {
    let class = classes::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let Some(key) = class.instructor_photo else {
        return not_found();
    };
    let data: Vec<u8> = ctx
        .storage
        .download::<Vec<u8>>(StdPath::new(&key))
        .await
        .map_err(|_| Error::NotFound)?;
    Ok(format::render()
        .header("content-type", content_type_for(&key))
        .response()
        .body(Body::from(data))?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/classes")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
        .add("/{id}/photo", post(upload_photo))
        .add("/{id}/photo", get(serve_photo))
}
