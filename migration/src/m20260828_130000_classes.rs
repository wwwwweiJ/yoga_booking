use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // `("organizations", "")` adds `organization_id BIGINT NOT NULL`
        // referencing `organizations.id` with ON DELETE CASCADE — deleting a
        // studio takes its classes with it.
        create_table(
            m,
            "classes",
            &[
                ("id", ColType::PkAuto),
                ("title", ColType::String),
                ("instructor", ColType::String),
                ("starts_at", ColType::TimestampWithTimeZone),
                ("duration_minutes", ColType::Integer),
                ("capacity", ColType::Integer),
            ],
            &[("organizations", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "classes").await
    }
}
