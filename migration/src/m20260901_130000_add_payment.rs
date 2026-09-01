use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // A class has a price (whole currency units; 0 = free). A booking
        // tracks payment: `pending` until the (mock) payment succeeds, then
        // `paid`. Defaults keep existing rows valid.
        add_column(m, "classes", "price", ColType::IntegerWithDefault(0)).await?;
        add_column(
            m,
            "bookings",
            "payment_status",
            ColType::StringWithDefault("pending".to_string()),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "bookings", "payment_status").await?;
        remove_column(m, "classes", "price").await?;
        Ok(())
    }
}
