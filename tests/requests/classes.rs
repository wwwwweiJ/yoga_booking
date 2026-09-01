use loco_rs::testing::prelude::*;
use loco_rs::TestServer;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde_json::Value;
use serial_test::serial;
use yoga_booking::{app::App, models::classes};

use super::prepare_data;

const STARTS_AT: &str = "2026-09-01T10:00:00Z";

/// Create a class through the API. The studio is implicit (the caller's), so
/// the body carries no `organization_id`.
async fn create_class(request: &TestServer, token: &str) -> Value {
    let (auth_key, auth_value) = prepare_data::auth_header(token);
    let response = request
        .post("/api/classes")
        .add_header(auth_key, auth_value)
        .json(&serde_json::json!({
            "title": "Vinyasa Flow",
            "instructor": "Mei",
            "starts_at": STARTS_AT,
            "duration_minutes": 60,
            "capacity": 20,
            "price": 500,
        }))
        .await;
    assert_eq!(response.status_code(), 201, "class create should return 201");
    response.json()
}

/// Insert a class straight into another studio (bypassing the API, which would
/// scope it to the caller) so cross-studio isolation can be tested.
async fn seed_class_in_org(ctx: &loco_rs::app::AppContext, org_id: i64) -> i64 {
    classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set("Other Studio Class".to_string()),
        instructor: ActiveValue::set("Someone".to_string()),
        starts_at: ActiveValue::set(
            chrono::DateTime::parse_from_rfc3339(STARTS_AT).unwrap(),
        ),
        duration_minutes: ActiveValue::set(60),
        capacity: ActiveValue::set(20),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("seed class")
    .id
}

#[tokio::test]
#[serial]
async fn create_requires_auth() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/classes")
            .json(&serde_json::json!({
                "title": "Vinyasa Flow",
                "instructor": "Mei",
                "starts_at": STARTS_AT,
                "duration_minutes": 60,
                "capacity": 20,
            }))
            .await;
        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn member_cannot_create_class() {
    request::<App, _, _>(|request, ctx| async move {
        // A plain student, not a teacher.
        let user = prepare_data::init_member_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .post("/api/classes")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "Vinyasa Flow",
                "instructor": "Mei",
                "starts_at": STARTS_AT,
                "duration_minutes": 60,
                "capacity": 20,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            403,
            "students can't create classes — only teachers"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_create_in_my_org() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let body = create_class(&request, &user.token).await;

        assert_eq!(body["title"], "Vinyasa Flow");
        assert_eq!(
            body["organization_id"].as_i64(),
            Some(user.organization_id),
            "the class is created in the caller's studio, implicitly"
        );
        assert_eq!(
            body["spots_left"].as_i64(),
            Some(20),
            "a fresh class has all its seats free"
        );
        assert_eq!(body["price"].as_i64(), Some(500), "price round-trips");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn create_invalid_returns_400() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .post("/api/classes")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "x",
                "instructor": "y",
                "starts_at": STARTS_AT,
                "duration_minutes": 0,
                "capacity": 0,
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn create_with_bad_starts_at_returns_400() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .post("/api/classes")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "Vinyasa Flow",
                "instructor": "Mei",
                "starts_at": "not-a-date",
                "duration_minutes": 60,
                "capacity": 20,
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn list_returns_only_my_orgs_classes() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        create_class(&request, &user.token).await;
        create_class(&request, &user.token).await;

        // A class in a different studio must not leak into the list.
        let other = prepare_data::seed_organization(&ctx, "Other Studio", "Asia/Tokyo").await;
        seed_class_in_org(&ctx, other.id).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get("/api/classes")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(
            body["total_items"].as_u64(),
            Some(2),
            "only the caller's two classes are listed"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn getting_another_orgs_class_is_404() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let other = prepare_data::seed_organization(&ctx, "Other Studio", "Asia/Tokyo").await;
        let other_class = seed_class_in_org(&ctx, other.id).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get(&format!("/api/classes/{other_class}"))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            404,
            "a class in another studio is not found for this user"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_my_class() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let created = create_class(&request, &user.token).await;
        let id = created["id"].as_i64().unwrap();

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .put(&format!("/api/classes/{id}"))
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "Yin Yoga",
                "instructor": "Aki",
                "starts_at": STARTS_AT,
                "duration_minutes": 90,
                "capacity": 15,
            }))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body["title"], "Yin Yoga");
        assert_eq!(body["capacity"].as_i64(), Some(15));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_delete_my_class() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let created = create_class(&request, &user.token).await;
        let id = created["id"].as_i64().unwrap();

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .delete(&format!("/api/classes/{id}"))
            .add_header(auth_key, auth_value)
            .await;
        assert_eq!(response.status_code(), 204);

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let after = request
            .get(&format!("/api/classes/{id}"))
            .add_header(auth_key, auth_value)
            .await;
        assert_eq!(after.status_code(), 404);
    })
    .await;
}
