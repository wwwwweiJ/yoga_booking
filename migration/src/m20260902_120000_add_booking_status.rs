use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // `booked` holds a seat and counts toward capacity; `waitlisted` is in
        // line for one and gets promoted when a seat frees. Existing rows are
        // real bookings, so they default to `booked`.
        add_column(
            m,
            "bookings",
            "status",
            ColType::StringWithDefault("booked".to_string()),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "bookings", "status").await
    }
}
