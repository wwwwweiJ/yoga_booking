use axum::http::StatusCode;
use loco_rs::prelude::*;

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
    let org_id = current_org_id(&auth, &ctx).await?;
    let starts_at = parse_starts_at(&params.starts_at)?;

    // The studio is implicit — always the caller's — so it can't be spoofed.
    let item = classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set(params.title),
        instructor: ActiveValue::set(params.instructor),
        starts_at: ActiveValue::set(starts_at),
        duration_minutes: ActiveValue::set(params.duration_minutes),
        capacity: ActiveValue::set(params.capacity),
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
    let org_id = current_org_id(&auth, &ctx).await?;
    let starts_at = parse_starts_at(&params.starts_at)?;
    let mut item = load_item(&ctx, id, org_id).await?.into_active_model();
    item.title = ActiveValue::set(params.title);
    item.instructor = ActiveValue::set(params.instructor);
    item.starts_at = ActiveValue::set(starts_at);
    item.duration_minutes = ActiveValue::set(params.duration_minutes);
    item.capacity = ActiveValue::set(params.capacity);
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
    let org_id = current_org_id(&auth, &ctx).await?;
    let item = load_item(&ctx, id, org_id).await?;
    classes::Entity::delete_by_id(item.id).exec(&ctx.db).await?;
    format::render().status(StatusCode::NO_CONTENT).empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/classes")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
