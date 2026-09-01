use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Three tiers: `member` (student, the default — everyone who registers),
        // `staff` (teacher — manages their studio's classes and page), `admin`
        // (operator — the cross-studio backoffice). The default backfills every
        // existing row to `member`.
        add_column(
            m,
            "users",
            "role",
            ColType::StringWithDefault("member".to_string()),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "users", "role").await
    }
}
