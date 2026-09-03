use std::collections::HashMap;

use axum::http::StatusCode;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::Deserialize;

use crate::{
    dtos::{bookings::Booking, common::Page},
    models::{
        _entities::{bookings, classes, users},
        bookings::{STATUS_BOOKED, STATUS_WAITLISTED},
    },
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

    // Batch-load the booked classes and their booking counts (two queries, no
    // N+1) and stitch them onto each booking to build the typed page by hand.
    let class_ids: Vec<i64> = res.page.iter().map(|b| b.class_id).collect();
    let classes: HashMap<i64, classes::Model> = classes::Entity::find()
        .filter(classes::Column::Id.is_in(class_ids.clone()))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|c| (c.id, c))
        .collect();
    let counts = bookings::Model::counts_by_class(&ctx.db, &class_ids).await?;

    let items: Vec<Booking> = res
        .page
        .into_iter()
        .filter_map(|b| {
            classes.get(&b.class_id).cloned().map(|c| {
                let booked = counts.get(&b.class_id).copied().unwrap_or(0);
                Booking::from_parts(b, c, booked)
            })
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

    // Can't book a class that has already started.
    if class.starts_at.with_timezone(&chrono::Utc) <= chrono::Utc::now() {
        return bad_request(format!("class {} has already started", class.id));
    }

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

    // A full class doesn't reject — it puts the member on the waitlist. Only
    // `booked` seats count toward capacity.
    let booked = bookings::Model::counts_by_class(&ctx.db, &[class.id])
        .await?
        .get(&class.id)
        .copied()
        .unwrap_or(0);
    let status = if booked >= i64::from(class.capacity) {
        STATUS_WAITLISTED
    } else {
        STATUS_BOOKED
    };

    let booking = bookings::ActiveModel {
        user_id: ActiveValue::set(user.id),
        class_id: ActiveValue::set(class.id),
        status: ActiveValue::set(status.to_string()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    // A waitlisted booking takes no seat; a booked one takes the next.
    let class_booked = if status == STATUS_BOOKED {
        booked + 1
    } else {
        booked
    };
    format::render()
        .status(StatusCode::CREATED)
        .json(Booking::from_parts(booking, class, class_booked))
}

/// Mock payment: flip a pending booking to paid. No real gateway — this is the
/// hook a real one (Stripe / a local gateway) would replace. Idempotent, and
/// scoped to the caller's own booking.
#[debug_handler]
async fn pay(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = current_user(&auth, &ctx).await?;

    let booking = bookings::Entity::find_by_id(id).one(&ctx.db).await?;
    let Some(booking) = booking.filter(|b| b.user_id == user.id) else {
        return not_found();
    };
    let class = classes::Entity::find_by_id(booking.class_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut active = booking.into_active_model();
    active.payment_status = ActiveValue::set("paid".to_string());
    let booking = active.update(&ctx.db).await?;

    let booked = bookings::Model::counts_by_class(&ctx.db, &[class.id])
        .await?
        .get(&class.id)
        .copied()
        .unwrap_or(0);
    format::json(Booking::from_parts(booking, class, booked))
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

    let freed_seat = booking.status == STATUS_BOOKED;
    let class_id = booking.class_id;
    bookings::Entity::delete_by_id(booking.id)
        .exec(&ctx.db)
        .await?;

    // Cancelling a seat promotes the longest-waiting person on the waitlist.
    if freed_seat {
        if let Some(next) = bookings::Entity::find()
            .filter(bookings::Column::ClassId.eq(class_id))
            .filter(bookings::Column::Status.eq(STATUS_WAITLISTED))
            .order_by_asc(bookings::Column::Id)
            .one(&ctx.db)
            .await?
        {
            let mut active = next.into_active_model();
            active.status = ActiveValue::set(STATUS_BOOKED.to_string());
            active.update(&ctx.db).await?;
        }
    }

    format::render().status(StatusCode::NO_CONTENT).empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/bookings")
        .add("/", get(list_mine))
        .add("/", post(create))
        .add("/{id}/pay", post(pay))
        .add("/{id}", delete(remove))
}
