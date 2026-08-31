use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

#[tokio::test]
#[serial]
async fn list_requires_auth() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/organizations").await;
        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn list_returns_only_my_org() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        // A studio the caller has nothing to do with — must not appear.
        prepare_data::seed_organization(&ctx, "Other Studio", "Asia/Tokyo").await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get("/api/organizations")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(
            body["total_items"].as_u64(),
            Some(1),
            "a user sees exactly their own studio"
        );
        assert_eq!(
            body["items"][0]["id"].as_i64(),
            Some(user.organization_id),
            "and it is their studio"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_my_org() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get(&format!("/api/organizations/{}", user.organization_id))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body["id"].as_i64(), Some(user.organization_id));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn getting_another_org_is_404() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let other = prepare_data::seed_organization(&ctx, "Other Studio", "Asia/Tokyo").await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get(&format!("/api/organizations/{}", other.id))
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            404,
            "another studio is invisible to this user"
        );
    })
    .await;
}
