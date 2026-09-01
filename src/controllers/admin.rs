//! The operator (admin) backoffice: create studios and mint teacher accounts.
//! Every endpoint is admin-only.
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;

use crate::{
    dtos::admin::{AdminOrganization, AdminUser, CreateOrganizationParams, CreateStaffParams},
    models::{
        _entities::{organizations, users},
        users::{RegisterParams, ROLE_STAFF},
    },
};

/// Gate every backoffice action to admins; anyone else gets a 403.
async fn require_admin(auth: &auth::JWT, ctx: &AppContext) -> Result<()> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_admin() {
        return Err(Error::CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail::with_reason("admins only"),
        ));
    }
    Ok(())
}

#[debug_handler]
async fn list_organizations(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    require_admin(&auth, &ctx).await?;
    let orgs = organizations::Entity::find()
        .order_by_asc(organizations::Column::Id)
        .all(&ctx.db)
        .await?;
    let items: Vec<AdminOrganization> = orgs.into_iter().map(AdminOrganization::from).collect();
    format::json(items)
}

#[debug_handler]
async fn create_organization(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateOrganizationParams>,
) -> Result<Response> {
    require_admin(&auth, &ctx).await?;
    let org = organizations::ActiveModel {
        name: ActiveValue::set(params.name),
        timezone: ActiveValue::set(params.timezone),
        ..Default::default()
    };
    org.validate()?;
    let org = org.insert(&ctx.db).await?;
    format::render()
        .status(StatusCode::CREATED)
        .json(AdminOrganization::from(org))
}

#[debug_handler]
async fn create_staff(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateStaffParams>,
) -> Result<Response> {
    require_admin(&auth, &ctx).await?;

    if organizations::Entity::find_by_id(params.organization_id)
        .one(&ctx.db)
        .await?
        .is_none()
    {
        return bad_request(format!(
            "organization_id {} does not exist",
            params.organization_id
        ));
    }

    let register = RegisterParams {
        name: params.name,
        email: params.email,
        password: params.password,
        organization_id: params.organization_id,
    };
    let user = match users::Model::create_with_password(&ctx.db, &register).await {
        Ok(user) => user,
        // A taken email is a conflict.
        Err(ModelError::EntityAlreadyExists) => {
            return Err(ModelError::EntityAlreadyExists.into());
        }
        Err(err) => return Err(err.into()),
    };

    // Promote from the default member to a teacher.
    let mut active = user.into_active_model();
    active.role = ActiveValue::set(ROLE_STAFF.to_string());
    let user = active.update(&ctx.db).await?;

    format::render()
        .status(StatusCode::CREATED)
        .json(AdminUser::from(user))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/admin")
        .add("/organizations", get(list_organizations))
        .add("/organizations", post(create_organization))
        .add("/staff", post(create_staff))
}
