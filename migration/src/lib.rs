#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260828_024419_organizations;
mod m20260828_130000_classes;
mod m20260828_140000_bookings;
mod m20260828_150000_add_organization_to_users;
mod m20260831_120000_add_public_id_to_organizations;
mod m20260901_120000_add_role_to_users;
mod m20260901_130000_add_payment;
mod m20260901_140000_add_instructor_photo;
mod m20260901_150000_add_studio_page;
mod m20260902_120000_add_booking_status;
mod m20260903_120000_add_line_user_id_to_users;
mod m20260904_120000_add_line_settings_to_organizations;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260828_024419_organizations::Migration),
            Box::new(m20260828_130000_classes::Migration),
            Box::new(m20260828_140000_bookings::Migration),
            Box::new(m20260828_150000_add_organization_to_users::Migration),
            Box::new(m20260831_120000_add_public_id_to_organizations::Migration),
            Box::new(m20260901_120000_add_role_to_users::Migration),
            Box::new(m20260901_130000_add_payment::Migration),
            Box::new(m20260901_140000_add_instructor_photo::Migration),
            Box::new(m20260901_150000_add_studio_page::Migration),
            Box::new(m20260902_120000_add_booking_status::Migration),
            Box::new(m20260903_120000_add_line_user_id_to_users::Migration),
            Box::new(m20260904_120000_add_line_settings_to_organizations::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
