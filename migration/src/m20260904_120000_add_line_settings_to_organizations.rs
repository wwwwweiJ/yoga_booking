use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // LINE login is per-studio: each studio brings its own LINE Login
        // channel + LIFF app. `line_liff_id` is the public LIFF id the frontend
        // initialises with; `line_channel_id` is what the backend verifies that
        // studio's members' id tokens against. Both NULL until a studio sets up
        // LINE in its settings.
        add_column(m, "organizations", "line_liff_id", ColType::StringNull).await?;
        add_column(m, "organizations", "line_channel_id", ColType::StringNull).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "organizations", "line_channel_id").await?;
        remove_column(m, "organizations", "line_liff_id").await
    }
}
