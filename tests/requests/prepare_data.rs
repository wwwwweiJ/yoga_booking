use axum::http::{HeaderName, HeaderValue};
use loco_rs::{app::AppContext, TestServer};
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use yoga_booking::{
    models::{_entities::organizations, users},
    views::auth::LoginResponse,
};

const USER_EMAIL: &str = "test@loco.com";
const USER_PASSWORD: &str = "1234";

pub struct LoggedInUser {
    pub user: users::Model,
    pub token: String,
    /// The studio this user belongs to — the scope for everything they can do.
    pub organization_id: i64,
    /// That studio's public register token (for registering more members into
    /// the same studio).
    pub organization_public_id: String,
}

/// Insert a studio directly, returning the full row (its `public_id` register
/// token is assigned by the DB default). Studios are created by an operator,
/// never through the API, so tests seed them the same way.
pub async fn seed_organization(
    ctx: &AppContext,
    name: &str,
    timezone: &str,
) -> organizations::Model {
    organizations::ActiveModel {
        name: ActiveValue::set(name.to_string()),
        timezone: ActiveValue::set(timezone.to_string()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("seed organization")
}

/// Register + log in a **staff** (teacher) user bound to a freshly-seeded
/// studio. Most tests exercise class management, which is staff-only, so this
/// is the default. Use [`init_member_login`] for student-only scenarios.
pub async fn init_user_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let mut logged = init_member_login(request, ctx).await;
    set_role(ctx, &logged.user.email, "staff").await;
    logged.user = users::Model::find_by_email(&ctx.db, &logged.user.email)
        .await
        .unwrap();
    logged
}

/// Register + log in a plain **member** (student) bound to a freshly-seeded
/// studio.
pub async fn init_member_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let org = seed_organization(ctx, "Test Studio", "Asia/Taipei").await;
    let login = register_and_login(request, USER_EMAIL, &org.public_id.to_string()).await;
    LoggedInUser {
        user: users::Model::find_by_email(&ctx.db, USER_EMAIL)
            .await
            .unwrap(),
        token: login.token,
        organization_id: org.id,
        organization_public_id: org.public_id.to_string(),
    }
}

/// Set a user's role directly (roles are assigned out-of-band — via the
/// `user:create` task or the admin backoffice — never through registration).
pub async fn set_role(ctx: &AppContext, email: &str, role: &str) {
    let user = users::Model::find_by_email(&ctx.db, email)
        .await
        .expect("user exists");
    let mut active = user.into_active_model();
    active.role = ActiveValue::set(role.to_string());
    active.update(&ctx.db).await.expect("role updated");
}

/// Register (into the studio identified by `organization_token`) + log in an
/// arbitrary user, returning the login response. Handy for a second account.
pub async fn register_and_login(
    request: &TestServer,
    email: &str,
    organization_token: &str,
) -> LoginResponse {
    request
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "name": "loco",
            "email": email,
            "password": USER_PASSWORD,
            "organization_token": organization_token,
        }))
        .await;

    // Login doesn't require verification (see `can_login_without_verify`), so
    // the registered user can authenticate straight away.
    let response = request
        .post("/api/auth/login")
        .json(&serde_json::json!({ "email": email, "password": USER_PASSWORD }))
        .await;
    serde_json::from_str(&response.text()).unwrap()
}

pub fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();

    (HeaderName::from_static("authorization"), auth_header_value)
}
