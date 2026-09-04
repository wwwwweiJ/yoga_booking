//! A teacher's editor for their own studio's customizable page.
use axum::body::Bytes;
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use std::path::Path as StdPath;

use crate::{
    dtos::studio::{
        blocks_from_value, StudioLineSettings, StudioPage, UpdatePageParams, UploadedFile,
    },
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
    let value = serde_json::to_value(&params.blocks).map_err(|e| Error::string(&e.to_string()))?;
    let mut active = org.into_active_model();
    active.page = ActiveValue::set(Some(value));
    let org = active.update(&ctx.db).await?;
    format::json(StudioPage {
        name: org.name.clone(),
        blocks: blocks_from_value(org.page),
    })
}

/// Upload an image (e.g. a gallery photo) for the studio page. Teacher-only.
/// Returns a public URL served by `/api/public/uploads/{name}`.
#[debug_handler]
async fn upload_image(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_staff() {
        return Err(Error::CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail::with_reason("only teachers can upload"),
        ));
    }

    let mut uploaded: Option<(Bytes, String)> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        if field.file_name().is_some() {
            let filename = field.file_name().unwrap_or("image").to_string();
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
        .filter(|e| !e.is_empty() && e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("bin");
    let name = format!("{}.{ext}", uuid::Uuid::new_v4());
    ctx.storage
        .upload(StdPath::new(&format!("studio-uploads/{name}")), &data)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    format::json(UploadedFile {
        url: format!("/api/public/uploads/{name}"),
    })
}

/// The studio's own LINE login settings (teacher-only). Empty strings when
/// unset. Includes the Channel ID — only ever returned to the studio's staff.
#[debug_handler]
async fn get_line(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let org = require_staff_org(&auth, &ctx).await?;
    format::json(StudioLineSettings {
        liff_id: org.line_liff_id.unwrap_or_default(),
        channel_id: org.line_channel_id.unwrap_or_default(),
    })
}

/// Set the studio's LINE LIFF id + Channel ID. Blank values clear the setting
/// (stored as NULL), so "LINE enabled" is simply "these are present".
#[debug_handler]
async fn update_line(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<StudioLineSettings>,
) -> Result<Response> {
    let org = require_staff_org(&auth, &ctx).await?;
    let normalize = |s: String| {
        let trimmed = s.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    };
    let mut active = org.into_active_model();
    active.line_liff_id = ActiveValue::set(normalize(params.liff_id));
    active.line_channel_id = ActiveValue::set(normalize(params.channel_id));
    let org = active.update(&ctx.db).await?;
    format::json(StudioLineSettings {
        liff_id: org.line_liff_id.unwrap_or_default(),
        channel_id: org.line_channel_id.unwrap_or_default(),
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/studio")
        .add("/page", get(get_page))
        .add("/page", put(update_page))
        .add("/line", get(get_line))
        .add("/line", put(update_line))
        .add("/uploads", post(upload_image))
}
