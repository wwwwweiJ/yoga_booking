use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

#[tokio::test]
#[serial]
async fn non_admin_cannot_reach_backoffice() {
    request::<App, _, _>(|request, ctx| async move {
        // A teacher (staff) is still not an admin.
        let staff = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&staff.token);
        let response = request
            .get("/api/admin/organizations")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 403, "only admins reach the backoffice");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn admin_can_create_and_list_studios() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::init_admin_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let created: Value = request
            .post("/api/admin/organizations")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({ "name": "New Studio", "timezone": "Asia/Taipei" }))
            .await
            .json();
        assert_eq!(created["name"], "New Studio");
        assert!(
            created["public_id"].as_str().is_some(),
            "the studio comes back with its register token"
        );

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let list: Value = request
            .get("/api/admin/organizations")
            .add_header(auth_key, auth_value)
            .await
            .json();
        // The admin's own seeded studio + the one just created.
        let names: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|o| o["name"].as_str())
            .collect();
        assert!(names.contains(&"New Studio"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn admin_can_create_a_teacher() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::init_admin_login(&request, &ctx).await;
        let org = prepare_data::seed_organization(&ctx, "Studio B", "Asia/Tokyo").await;

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let response = request
            .post("/api/admin/staff")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "organization_id": org.id,
                "name": "Teacher Mei",
                "email": "mei-teacher@example.com",
                "password": "secret12",
            }))
            .await;

        assert_eq!(response.status_code(), 201);
        let body: Value = response.json();
        assert_eq!(body["role"], "staff", "the new account is a teacher");
        assert_eq!(body["email"], "mei-teacher@example.com");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn create_teacher_duplicate_email_is_409() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::init_admin_login(&request, &ctx).await;
        let org = prepare_data::seed_organization(&ctx, "Studio B", "Asia/Tokyo").await;

        let payload = serde_json::json!({
            "organization_id": org.id,
            "name": "Teacher Mei",
            "email": "dup-teacher@example.com",
            "password": "secret12",
        });

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let first = request
            .post("/api/admin/staff")
            .add_header(auth_key, auth_value)
            .json(&payload)
            .await;
        assert_eq!(first.status_code(), 201);

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let second = request
            .post("/api/admin/staff")
            .add_header(auth_key, auth_value)
            .json(&payload)
            .await;
        assert_eq!(second.status_code(), 409);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn create_teacher_unknown_studio_is_400() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::init_admin_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&admin.token);
        let response = request
            .post("/api/admin/staff")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "organization_id": 99999,
                "name": "Teacher",
                "email": "t@example.com",
                "password": "secret12",
            }))
            .await;

        assert_eq!(response.status_code(), 400);
    })
    .await;
}
