use std::collections::HashMap;

use axum::http::StatusCode;
use loco_rs::prelude::*;
use sea_orm::PaginatorTrait;
use serde::Deserialize;

use crate::{
    dtos::{bookings::Booking, common::Page},
    models::_entities::{bookings, classes, users},
};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(flatten)]
    pub pagination: query::PaginationQuery,
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    pub class_id: i64,
}

/// Resolve the caller (the JWT carries the user's `pid`). Every booking action
/// is scoped to this user — you can only see and cancel your own.
async fn current_user(auth: &auth::JWT, ctx: &AppContext) -> Result<users::Model> {
    Ok(users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?)
}

#[debug_handler]
async fn list_mine(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<Response> {
    let user = current_user(&auth, &ctx).await?;

    let res = query::paginate(
        &ctx.db,
        bookings::Entity::find().filter(bookings::Column::UserId.eq(user.id)),
        None,
        &params.pagination,
    )
    .await?;

    // Batch-load the booked classes (one query, no N+1) and stitch them onto
    // each booking to build the typed page by hand.
    let class_ids: Vec<i64> = res.page.iter().map(|b| b.class_id).collect();
    let classes: HashMap<i64, classes::Model> = classes::Entity::find()
        .filter(classes::Column::Id.is_in(class_ids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| (c.id, c))
        .collect();

    let items: Vec<Booking> = res
        .page
        .into_iter()
        .filter_map(|b| classes.get(&b.class_id).cloned().map(|c| Booking::from_parts(b, c)))
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
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(body): Json<CreateBody>,
) -> Result<Response> {
    let user = current_user(&auth, &ctx).await?;

    // A user can only book classes at their own studio. A class in another
    // studio is treated as non-existent from here.
    let Some(class) = classes::Entity::find_by_id(body.class_id)
        .one(&ctx.db)
        .await?
        .filter(|c| c.organization_id == user.organization_id)
    else {
        return bad_request(format!("class_id {} does not exist", body.class_id));
    };

    // Already booked? Friendlier than letting the unique index throw.
    let already = bookings::Entity::find()
        .filter(bookings::Column::UserId.eq(user.id))
        .filter(bookings::Column::ClassId.eq(class.id))
        .one(&ctx.db)
        .await?
        .is_some();
    if already {
        return Err(ModelError::EntityAlreadyExists.into());
    }

    // Capacity check.
    let booked = bookings::Entity::find()
        .filter(bookings::Column::ClassId.eq(class.id))
        .count(&ctx.db)
        .await?;
    if booked >= u64::try_from(class.capacity).unwrap_or(0) {
        return bad_request(format!("class {} is full", class.id));
    }

    let booking = bookings::ActiveModel {
        user_id: ActiveValue::set(user.id),
        class_id: ActiveValue::set(class.id),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::render()
        .status(StatusCode::CREATED)
        .json(Booking::from_parts(booking, class))
}

#[debug_handler]
async fn remove(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = current_user(&auth, &ctx).await?;

    // Treat "not yours" the same as "not found" so a booking's existence
    // isn't leaked across users.
    let booking = bookings::Entity::find_by_id(id).one(&ctx.db).await?;
    let Some(booking) = booking.filter(|b| b.user_id == user.id) else {
        return not_found();
    };

    bookings::Entity::delete_by_id(booking.id)
        .exec(&ctx.db)
        .await?;
    format::render().status(StatusCode::NO_CONTENT).empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/bookings")
        .add("/", get(list_mine))
        .add("/", post(create))
        .add("/{id}", delete(remove))
}
