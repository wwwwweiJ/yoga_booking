use insta::assert_debug_snapshot;
use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serial_test::serial;
use yoga_booking::{app::App, models::organizations};

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("organizations");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_create() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let organization = organizations::ActiveModel {
        name: ActiveValue::set("Sunrise Yoga".to_string()),
        timezone: ActiveValue::set("Asia/Taipei".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .expect("an organization should be created");

    // Narrowed on purpose: snapshot only the fields this test is about so
    // adding a column later doesn't re-bless every organization snapshot.
    assert_debug_snapshot!((organization.name, organization.timezone));
}

#[tokio::test]
#[serial]
async fn can_validate_model() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let invalid_organization = organizations::ActiveModel {
        name: ActiveValue::set("1".to_string()),
        timezone: ActiveValue::set(String::new()),
        ..Default::default()
    };

    let res = invalid_organization.insert(&boot.app_context.db).await;

    assert_debug_snapshot!(res);
}

#[tokio::test]
#[serial]
async fn can_find_by_id() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let created = organizations::ActiveModel {
        name: ActiveValue::set("Moonlight Yoga".to_string()),
        timezone: ActiveValue::set("Asia/Tokyo".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .expect("an organization should be created");

    let found = organizations::Entity::find_by_id(created.id)
        .one(&boot.app_context.db)
        .await
        .expect("query should succeed");

    assert_debug_snapshot!(found.map(|org| (org.name, org.timezone)));
}
