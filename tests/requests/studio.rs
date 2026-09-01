use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

#[tokio::test]
#[serial]
async fn staff_can_edit_and_public_can_read_the_page() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let put = request
            .put("/api/studio/page")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "blocks": [
                    { "type": "hero", "heading": "Welcome", "subheading": "Flow with us" },
                    { "type": "about", "text": "A calm little studio." },
                    { "type": "gallery", "images": ["/a.jpg", "/b.jpg"] }
                ]
            }))
            .await;

        assert_eq!(put.status_code(), 200);
        let body: Value = put.json();
        assert_eq!(body["blocks"].as_array().unwrap().len(), 3);
        assert_eq!(body["blocks"][0]["type"], "hero");
        assert_eq!(body["blocks"][0]["heading"], "Welcome");

        // The public page endpoint needs no auth and returns name + blocks.
        let page = request
            .get(&format!(
                "/api/public/organizations/{}/page",
                user.organization_public_id
            ))
            .await;
        assert_eq!(page.status_code(), 200);
        let public: Value = page.json();
        assert_eq!(public["name"], "Test Studio");
        assert_eq!(public["blocks"][2]["type"], "gallery");
        assert_eq!(public["blocks"][2]["images"][0], "/a.jpg");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn member_cannot_edit_the_page() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_member_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let put = request
            .put("/api/studio/page")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({ "blocks": [] }))
            .await;

        assert_eq!(put.status_code(), 403, "students can't edit the studio page");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn public_page_unknown_token_is_404() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .get("/api/public/organizations/00000000-0000-0000-0000-000000000000/page")
            .await;
        assert_eq!(response.status_code(), 404);
    })
    .await;
}
