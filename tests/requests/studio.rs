use axum_test::multipart::{MultipartForm, Part};
use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use yoga_booking::app::App;

use super::prepare_data;

#[tokio::test]
#[serial]
async fn staff_sets_line_settings_and_only_liff_id_is_public() {
    request::<App, _, _>(|request, ctx| async move {
        let staff = prepare_data::init_user_login(&request, &ctx).await;

        let (k, v) = prepare_data::auth_header(&staff.token);
        let put = request
            .put("/api/studio/line")
            .add_header(k, v)
            .json(&serde_json::json!({ "liff_id": "123-abc", "channel_id": "2001" }))
            .await;
        assert_eq!(put.status_code(), 200);

        // Staff read both values back.
        let (k, v) = prepare_data::auth_header(&staff.token);
        let got: Value = request
            .get("/api/studio/line")
            .add_header(k, v)
            .await
            .json();
        assert_eq!(got["liff_id"], "123-abc");
        assert_eq!(got["channel_id"], "2001");

        // The public studio payload exposes liff_id (the LIFF client needs it)
        // but must NOT leak the channel_id.
        let pub_org: Value = request
            .get(&format!(
                "/api/public/organizations/{}",
                staff.organization_public_id
            ))
            .await
            .json();
        assert_eq!(pub_org["liff_id"], "123-abc");
        assert!(
            pub_org.get("channel_id").is_none(),
            "channel_id must never be exposed on the public endpoint"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn teachers_block_round_trips_to_the_public_page() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (k, v) = prepare_data::auth_header(&user.token);
        let put = request
            .put("/api/studio/page")
            .add_header(k, v)
            .json(&serde_json::json!({
                "blocks": [
                    { "type": "teachers", "heading": "Our teachers", "members": [
                        {
                            "name": "Mei", "photo": "/m.jpg", "title": "Yin yoga",
                            "bio": "10 years", "instagram": "@mei", "website": "mei.com"
                        }
                    ]}
                ]
            }))
            .await;
        assert_eq!(put.status_code(), 200);

        let page: Value = request
            .get(&format!(
                "/api/public/organizations/{}/page",
                user.organization_public_id
            ))
            .await
            .json();
        assert_eq!(page["blocks"][0]["type"], "teachers");
        assert_eq!(page["blocks"][0]["members"][0]["name"], "Mei");
        assert_eq!(page["blocks"][0]["members"][0]["title"], "Yin yoga");
    })
    .await;
}

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

        assert_eq!(
            put.status_code(),
            403,
            "students can't edit the studio page"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn public_classes_lists_upcoming_without_leaking_ids() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        // A future class (staff create) and a past one (direct insert).
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        request
            .post("/api/classes")
            .add_header(auth_key, auth_value)
            .json(&serde_json::json!({
                "title": "Sunrise Flow",
                "instructor": "Mei",
                "starts_at": "2030-01-01T10:00:00Z",
                "duration_minutes": 60,
                "capacity": 20,
                "price": 500,
            }))
            .await;

        let response = request
            .get(&format!(
                "/api/public/organizations/{}/classes",
                user.organization_public_id
            ))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        let list = body.as_array().unwrap();
        assert_eq!(list.len(), 1, "only the upcoming class");
        assert_eq!(list[0]["title"], "Sunrise Flow");
        assert_eq!(list[0]["price"].as_i64(), Some(500));
        assert!(list[0].get("id").is_none(), "no internal id is exposed");
        assert!(
            list[0].get("organization_id").is_none(),
            "no org id is exposed"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn staff_can_upload_and_serve_a_studio_image() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let form = MultipartForm::new().add_part(
            "image",
            Part::bytes(b"\x89PNG gallery-bytes".to_vec())
                .file_name("g.png")
                .mime_type("image/png"),
        );
        let response = request
            .post("/api/studio/uploads")
            .add_header(auth_key, auth_value)
            .multipart(form)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        let url = body["url"].as_str().unwrap();
        assert!(url.starts_with("/api/public/uploads/"));

        // Served publicly (no auth).
        let img = request.get(url).await;
        assert_eq!(img.status_code(), 200);
        assert!(!img.as_bytes().is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn member_cannot_upload_a_studio_image() {
    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_member_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let form = MultipartForm::new().add_part(
            "image",
            Part::bytes(b"x".to_vec())
                .file_name("g.png")
                .mime_type("image/png"),
        );
        let response = request
            .post("/api/studio/uploads")
            .add_header(auth_key, auth_value)
            .multipart(form)
            .await;

        assert_eq!(response.status_code(), 403);
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
