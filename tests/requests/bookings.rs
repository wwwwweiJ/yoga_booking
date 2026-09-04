use loco_rs::testing::prelude::*;
use loco_rs::TestServer;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

const STARTS_AT: &str = "2030-01-01T10:00:00Z";

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
        assert_eq!(
            body["class"]["spots_left"].as_i64(),
            Some(19),
            "booking takes one of the 20 seats"
        );
        assert_eq!(
            body["payment_status"], "pending",
            "a fresh booking is unpaid until the mock payment"
        );
        assert_eq!(body["status"], "booked", "a seat is available");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn staff_sees_the_class_roster() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 1).await;

        let m1 = prepare_data::register_and_login(
            &request,
            "m1@loco.com",
            &owner.organization_public_id,
        )
        .await;
        assert_eq!(book(&request, &m1.token, class_id).await.status_code(), 201);
        let m2 = prepare_data::register_and_login(
            &request,
            "m2@loco.com",
            &owner.organization_public_id,
        )
        .await;
        assert_eq!(book(&request, &m2.token, class_id).await.status_code(), 201);

        let (auth_key, auth_value) = prepare_data::auth_header(&owner.token);
        let roster: Value = request
            .get(&format!("/api/classes/{class_id}/bookings"))
            .add_header(auth_key, auth_value)
            .await
            .json();

        let list = roster.as_array().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0]["status"], "booked", "booked members come first");
        assert_eq!(list[0]["email"], "m1@loco.com");
        assert_eq!(list[1]["status"], "waitlisted");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn member_cannot_see_the_roster() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 20).await;
        let member =
            prepare_data::register_and_login(&request, "m@loco.com", &owner.organization_public_id)
                .await;

        let (auth_key, auth_value) = prepare_data::auth_header(&member.token);
        let response = request
            .get(&format!("/api/classes/{class_id}/bookings"))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 403);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_pay_for_a_booking() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 20).await;
        let created: Value = book(&request, &user.token, class_id).await.json();
        let booking_id = created["id"].as_i64().unwrap();

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .post(&format!("/api/bookings/{booking_id}/pay"))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body["payment_status"], "paid", "mock payment marks it paid");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_pay_for_someone_elses_booking() {
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
            .post(&format!("/api/bookings/{booking_id}/pay"))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            404,
            "you can't pay another's booking"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn booking_reduces_spots_left() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &user.token, 3).await;

        book(&request, &user.token, class_id).await;

        // The class endpoint reflects the seat that was just taken.
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let class: Value = request
            .get(&format!("/api/classes/{class_id}"))
            .add_header(auth_key, auth_value)
            .await
            .json();
        assert_eq!(
            class["spots_left"].as_i64(),
            Some(2),
            "3 capacity - 1 booked"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_book_a_started_class() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        // Create a class whose start time is already in the past.
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let class: Value = request
            .post("/api/classes")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "Past Class",
                "instructor": "Mei",
                "starts_at": "2020-01-01T10:00:00Z",
                "duration_minutes": 60,
                "capacity": 20,
            }))
            .await
            .json();
        let class_id = class["id"].as_i64().unwrap();

        let response = book(&request, &user.token, class_id).await;
        assert_eq!(
            response.status_code(),
            400,
            "a class that has already started can't be booked"
        );
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

        assert_eq!(
            book(&request, &user.token, class_id).await.status_code(),
            201
        );
        assert_eq!(
            book(&request, &user.token, class_id).await.status_code(),
            409
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn full_class_waitlists() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 1).await;
        assert_eq!(
            book(&request, &owner.token, class_id).await.status_code(),
            201
        );

        // A second member of the SAME studio joins the waitlist instead.
        let other = prepare_data::register_and_login(
            &request,
            "second@loco.com",
            &owner.organization_public_id,
        )
        .await;
        let response = book(&request, &other.token, class_id).await;

        assert_eq!(
            response.status_code(),
            201,
            "a full class waitlists, not rejects"
        );
        let body: Value = response.json();
        assert_eq!(body["status"], "waitlisted");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cancelling_a_seat_promotes_the_waitlist() {
    request::<App, _, _>(|request, ctx| async move {
        let owner = prepare_data::init_user_login(&request, &ctx).await;
        let class_id = seed_class(&request, &owner.token, 1).await;
        let owner_booking: Value = book(&request, &owner.token, class_id).await.json();
        let owner_booking_id = owner_booking["id"].as_i64().unwrap();

        let other = prepare_data::register_and_login(
            &request,
            "second@loco.com",
            &owner.organization_public_id,
        )
        .await;
        let waitlisted: Value = book(&request, &other.token, class_id).await.json();
        assert_eq!(waitlisted["status"], "waitlisted");

        // Owner cancels — the waitlisted member should be promoted.
        let (auth_key, auth_value) = prepare_data::auth_header(&owner.token);
        request
            .delete(&format!("/api/bookings/{owner_booking_id}"))
            .add_header(auth_key, auth_value)
            .await;

        let (auth_key, auth_value) = prepare_data::auth_header(&other.token);
        let mine: Value = request
            .get("/api/bookings")
            .add_header(auth_key, auth_value)
            .await
            .json();
        assert_eq!(
            mine["items"][0]["status"], "booked",
            "the waitlisted member takes the freed seat"
        );
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
        assert_eq!(
            book(&request, &user.token, class_id).await.status_code(),
            201
        );
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
