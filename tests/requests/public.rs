use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

#[tokio::test]
#[serial]
async fn public_org_lookup_by_token_needs_no_auth() {
    request::<App, _, _>(|request, ctx| async move {
        let org = prepare_data::seed_organization(&ctx, "Sunrise Yoga", "Asia/Taipei").await;

        // No Authorization header — this is the signed-out register page,
        // reaching the studio by its non-guessable token.
        let response = request
            .get(&format!("/api/public/organizations/{}", org.public_id))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body["name"], "Sunrise Yoga");
        // Only the name is exposed — no numeric id, timezone, etc.
        assert!(body.get("id").is_none(), "numeric id must not leak");
        assert!(body.get("timezone").is_none(), "timezone must not leak");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn public_org_lookup_unknown_token_is_404() {
    request::<App, _, _>(|request, _ctx| async move {
        // A well-formed but unused token.
        let response = request
            .get("/api/public/organizations/00000000-0000-0000-0000-000000000000")
            .await;
        assert_eq!(response.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn public_org_lookup_garbage_token_is_404() {
    request::<App, _, _>(|request, _ctx| async move {
        // Not even a UUID — a casual guess like "2".
        let response = request.get("/api/public/organizations/2").await;
        assert_eq!(response.status_code(), 404);
    })
    .await;
}
