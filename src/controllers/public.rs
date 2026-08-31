//! Unauthenticated endpoints. Everything here is reachable by a signed-out
//! visitor, so it must expose only the bare minimum — currently just a
//! studio's public name for its per-studio register page.
use loco_rs::prelude::*;

use crate::{dtos::organizations::PublicOrganization, models::organizations};

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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/public")
        .add("/organizations/{token}", get(get_organization))
}
