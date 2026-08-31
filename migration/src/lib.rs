#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260828_024419_organizations;
mod m20260828_130000_classes;
mod m20260828_140000_bookings;
mod m20260828_150000_add_organization_to_users;
mod m20260831_120000_add_public_id_to_organizations;
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
            // inject-above (do not remove this comment)
        ]
    }
}
