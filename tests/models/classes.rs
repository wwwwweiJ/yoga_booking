use insta::assert_debug_snapshot;
use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serial_test::serial;
use yoga_booking::{
    app::App,
    models::{classes, organizations},
};

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("classes");
        let _guard = settings.bind_to_scope();
    };
}

/// Every class needs a studio to belong to (the FK is NOT NULL), so tests
/// start by making one.
async fn seed_org(db: &sea_orm::DatabaseConnection) -> i64 {
    organizations::ActiveModel {
        name: ActiveValue::set("Sunrise Yoga".to_string()),
        timezone: ActiveValue::set("Asia/Taipei".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("an organization should be created")
    .id
}

fn starts_at() -> sea_orm::prelude::DateTimeWithTimeZone {
    chrono::DateTime::parse_from_rfc3339("2030-01-01T10:00:00Z").expect("valid datetime")
}

#[tokio::test]
#[serial]
async fn can_create() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    let org_id = seed_org(&boot.app_context.db).await;

    let class = classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set("Vinyasa Flow".to_string()),
        instructor: ActiveValue::set("Mei".to_string()),
        starts_at: ActiveValue::set(starts_at()),
        duration_minutes: ActiveValue::set(60),
        capacity: ActiveValue::set(20),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .expect("a class should be created");

    assert_eq!(class.organization_id, org_id);
    assert_debug_snapshot!((class.title, class.instructor, class.capacity));
}

#[tokio::test]
#[serial]
async fn can_validate_model() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    let org_id = seed_org(&boot.app_context.db).await;

    let invalid = classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set("x".to_string()),
        instructor: ActiveValue::set("y".to_string()),
        starts_at: ActiveValue::set(starts_at()),
        duration_minutes: ActiveValue::set(0),
        capacity: ActiveValue::set(0),
        ..Default::default()
    };

    let res = invalid.insert(&boot.app_context.db).await;

    assert_debug_snapshot!(res);
}

#[tokio::test]
#[serial]
async fn deleting_org_cascades_to_classes() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    let org_id = seed_org(&boot.app_context.db).await;

    let class = classes::ActiveModel {
        organization_id: ActiveValue::set(org_id),
        title: ActiveValue::set("Vinyasa Flow".to_string()),
        instructor: ActiveValue::set("Mei".to_string()),
        starts_at: ActiveValue::set(starts_at()),
        duration_minutes: ActiveValue::set(60),
        capacity: ActiveValue::set(20),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await
    .expect("a class should be created");

    organizations::Entity::delete_by_id(org_id)
        .exec(&boot.app_context.db)
        .await
        .expect("deleting the organization should succeed");

    let found = classes::Entity::find_by_id(class.id)
        .one(&boot.app_context.db)
        .await
        .expect("query should succeed");

    assert!(
        found.is_none(),
        "deleting a studio should cascade-delete its classes"
    );
}
