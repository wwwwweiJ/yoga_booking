//! Unauthenticated endpoints. Everything here is reachable by a signed-out
//! visitor, so it must expose only the bare minimum — currently just a
//! studio's public name for its per-studio register page.
use axum::body::Body;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use std::path::Path as StdPath;

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

fn content_type_for(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

/// Serve an uploaded studio image by filename. Public. The name is a single
/// path segment; reject anything but a plain `<uuid>.<ext>` to avoid traversal.
#[debug_handler]
async fn serve_upload(Path(name): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let safe = !name.is_empty()
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if !safe {
        return not_found();
    }
    let data: Vec<u8> = ctx
        .storage
        .download::<Vec<u8>>(StdPath::new(&format!("studio-uploads/{name}")))
        .await
        .map_err(|_| Error::NotFound)?;
    Ok(format::render()
        .header("content-type", content_type_for(&name))
        .response()
        .body(Body::from(data))?)
}

/// Runtime config the SPA reads at boot. Currently just the LINE LIFF id, held
/// in the environment so it can be set/changed without rebuilding the frontend
/// bundle. Empty when LINE login isn't configured — the frontend then hides the
/// LINE button rather than initialising LIFF with a blank id.
#[debug_handler]
async fn get_config() -> Result<Response> {
    let liff_id = std::env::var("LINE_LIFF_ID").unwrap_or_default();
    format::json(serde_json::json!({ "liff_id": liff_id }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/public")
        .add("/config", get(get_config))
        .add("/organizations/{token}", get(get_organization))
        .add("/organizations/{token}/page", get(get_page))
        .add("/organizations/{token}/classes", get(get_classes))
        .add("/uploads/{name}", get(serve_upload))
}
