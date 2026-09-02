//! Unauthenticated endpoints. Everything here is reachable by a signed-out
//! visitor, so it must expose only the bare minimum — currently just a
//! studio's public name for its per-studio register page.
use loco_rs::prelude::*;
use sea_orm::QueryOrder;

use crate::{
    dtos::{
        classes::PublicClass,
        organizations::PublicOrganization,
        studio::{blocks_from_value, StudioPage},
    },
    models::_entities::{bookings, classes},
    models::organizations,
};

/// Look up a studio by its public register token (`/register/<token>`) to show
/// its name. The token is a random UUID, so a visitor can only reach a studio
/// whose link they were actually given — a malformed or unknown token is a 404.
#[debug_handler]
async fn get_organization(
    Path(token): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(public_id) = token.parse::<Uuid>() else {
        return not_found();
    };
    let org = organizations::Model::find_by_public_id(&ctx.db, &public_id)
        .await
        .map_err(|_| Error::NotFound)?;
    format::json(PublicOrganization::from(org))
}

/// The studio's public page (name + blocks) for its `/studio/<token>` page.
#[debug_handler]
async fn get_page(Path(token): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let Ok(public_id) = token.parse::<Uuid>() else {
        return not_found();
    };
    let org = organizations::Model::find_by_public_id(&ctx.db, &public_id)
        .await
        .map_err(|_| Error::NotFound)?;
    format::json(StudioPage {
        name: org.name.clone(),
        blocks: blocks_from_value(org.page),
    })
}

/// A studio's upcoming classes, for the `schedule` block on its public page.
#[debug_handler]
async fn get_classes(Path(token): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let Ok(public_id) = token.parse::<Uuid>() else {
        return not_found();
    };
    let org = organizations::Model::find_by_public_id(&ctx.db, &public_id)
        .await
        .map_err(|_| Error::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let upcoming = classes::Entity::find()
        .filter(classes::Column::OrganizationId.eq(org.id))
        .filter(classes::Column::StartsAt.gte(now))
        .order_by_asc(classes::Column::StartsAt)
        .all(&ctx.db)
        .await?;

    let class_ids: Vec<i64> = upcoming.iter().map(|c| c.id).collect();
    let counts = bookings::Model::counts_by_class(&ctx.db, &class_ids).await?;
    let items: Vec<PublicClass> = upcoming
        .into_iter()
        .map(|c| {
            let booked = counts.get(&c.id).copied().unwrap_or(0);
            PublicClass::from_parts(c, booked)
        })
        .collect();
    format::json(items)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/public")
        .add("/organizations/{token}", get(get_organization))
        .add("/organizations/{token}/page", get(get_page))
        .add("/organizations/{token}/classes", get(get_classes))
}
