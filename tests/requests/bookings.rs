use loco_rs::testing::prelude::*;
use loco_rs::TestServer;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

const STARTS_AT: &str = "2026-09-01T10:00:00Z";

/// Create a class in the caller's studio (implicit org) with the given
/// capacity, returning its id.
async fn seed_class(request: &TestServer, token: &str, capacity: i32) -> i64 {
    let (auth_key, auth_value) = prepare_data::auth_header(token);
    let class: Value = request
        .post("/api/classes")
        .add_header(auth_key, auth_value)
        .json(&serde_json::json!({
            "title": "Vinyasa Flow",
            "instructor": "Mei",
            "starts_at": STARTS_AT,
            "duration_minutes": 60,
            "capacity": capacity,
        }))
        .await
        .json();
    class["id"].as_i64().unwrap()
}

async fn book(request: &TestServer, token: &str, class_id: i64) -> axum_test::TestResponse {
    let (auth_key, auth_value) = prepare_data::auth_header(token);
    request
        .post("/api/bookings")
        .add_header(auth_key, auth_value)
        .json(&serde_json::json!({ "class_id": class_id }))
        .await
}

#[tokio::test]
#[serial]
async fn book_requires_auth() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/bookings")
            .json(&serde_json::json!({ "class_id": 1 }))
            .await;
        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_book_a_class() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 20).await;

        let response = book(&request, &user.token, class_id).await;

        assert_eq!(response.status_code(), 201);
        let body: Value = response.json();
        assert_eq!(body["class_id"].as_i64(), Some(class_id));
        assert_eq!(body["class"]["title"], "Vinyasa Flow");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn booking_unknown_class_returns_400() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let response = book(&request, &user.token, 99999).await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn double_booking_returns_409() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 20).await;

        assert_eq!(book(&request, &user.token, class_id).await.status_code(), 201);
        assert_eq!(book(&request, &user.token, class_id).await.status_code(), 409);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn full_class_returns_400() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 1).await;
        assert_eq!(book(&request, &owner.token, class_id).await.status_code(), 201);

        // A second member of the SAME studio finds the class already full.
        let other = prepare_data::register_and_login(
            &request,
            "second@loco.com",
            &owner.organization_public_id,
        )
        .await;
        let response = book(&request, &other.token, class_id).await;

        assert_eq!(response.status_code(), 400, "a full class rejects booking");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_list_my_bookings() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 20).await;
        book(&request, &user.token, class_id).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get("/api/bookings")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body["total_items"].as_u64(), Some(1));
        assert_eq!(body["items"][0]["class"]["title"], "Vinyasa Flow");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_cancel_own_booking() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 20).await;
        let created: Value = book(&request, &user.token, class_id).await.json();
        let booking_id = created["id"].as_i64().unwrap();

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .delete(&format!("/api/bookings/{booking_id}"))
            .add_header(auth_key, auth_value)
            .await;
        assert_eq!(response.status_code(), 204);

        // cancelling frees the seat, so the class can be booked again
        assert_eq!(book(&request, &user.token, class_id).await.status_code(), 201);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_cancel_someone_elses_booking() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 20).await;
        let created: Value = book(&request, &owner.token, class_id).await.json();
        let booking_id = created["id"].as_i64().unwrap();

        let other = prepare_data::register_and_login(
            &request,
            "second@loco.com",
            &owner.organization_public_id,
        )
        .await;
        let (auth_key, auth_value) = prepare_data::auth_header(&other.token);
        let response = request
            .delete(&format!("/api/bookings/{booking_id}"))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            404,
            "another user's booking is invisible, not cancellable"
        );
    })
    .await;
}
