use loco_rs::prelude::*;

use crate::{
    dtos::{common::Page, organizations::Organization},
    models::_entities::{organizations, users},
};

/// A user belongs to exactly one studio; every read is scoped to it. Studios
/// themselves are created out-of-band (the `organization:create` task / seed),
/// so this controller is read-only.
async fn current_org(auth: &auth::JWT, ctx: &AppContext) -> Result<organizations::Model> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    organizations::Entity::find_by_id(user.organization_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)
}

#[debug_handler]
async fn list(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    // A user has one studio, so the "list" is a one-item page — but it keeps
    // the same envelope the frontend uses everywhere else.
    let org = current_org(&_auth, &ctx).await?;
    format::json(Page {
        items: vec![Organization::from(org)],
        page: 1,
        page_size: 1,
        total_pages: 1,
        total_items: 1,
    })
}

#[debug_handler]
async fn get_one(
    auth: auth::JWT,
    Path(id): Path<i64>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let org = current_org(&auth, &ctx).await?;
    // Another studio's id is simply not found for this user.
    if org.id != id {
        return not_found();
    }
    format::json(Organization::from(org))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/organizations")
        .add("/", get(list))
        .add("/{id}", get(get_one))
}
