//! A teacher's editor for their own studio's customizable page.
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;

use crate::{
    dtos::studio::{blocks_from_value, StudioPage, UpdatePageParams},
    models::_entities::{organizations, users},
};

/// The caller's studio — teachers only (students can't edit the page).
async fn require_staff_org(auth: &auth::JWT, ctx: &AppContext) -> Result<organizations::Model> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_staff() {
        return Err(Error::CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail::with_reason("only teachers can edit the studio page"),
        ));
    }
    organizations::Entity::find_by_id(user.organization_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)
}

#[debug_handler]
async fn get_page(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let org = require_staff_org(&auth, &ctx).await?;
    format::json(StudioPage {
        name: org.name.clone(),
        blocks: blocks_from_value(org.page),
    })
}

#[debug_handler]
async fn update_page(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdatePageParams>,
) -> Result<Response> {
    let org = require_staff_org(&auth, &ctx).await?;
    let value =
        serde_json::to_value(&params.blocks).map_err(|e| Error::string(&e.to_string()))?;
    let mut active = org.into_active_model();
    active.page = ActiveValue::set(Some(value));
    let org = active.update(&ctx.db).await?;
    format::json(StudioPage {
        name: org.name.clone(),
        blocks: blocks_from_value(org.page),
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/studio")
        .add("/page", get(get_page))
        .add("/page", put(update_page))
}
