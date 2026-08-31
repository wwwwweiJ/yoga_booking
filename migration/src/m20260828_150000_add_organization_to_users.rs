use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Every user belongs to exactly one studio. `add_reference` adds
        // `organization_id BIGINT NOT NULL` → `organizations.id`
        // (ON DELETE CASCADE): removing a studio removes its members.
        add_reference(m, "users", "organizations", "").await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_reference(m, "users", "organizations", "").await
    }
}
