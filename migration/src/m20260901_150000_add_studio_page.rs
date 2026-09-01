use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // The customizable studio page: an ordered JSON array of blocks
        // (hero / about / gallery / …). Null = no page yet (renders empty).
        add_column(m, "organizations", "page", ColType::JsonBinaryNull).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "organizations", "page").await
    }
}
