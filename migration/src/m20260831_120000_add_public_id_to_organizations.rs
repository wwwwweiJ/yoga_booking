use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // A non-guessable public handle for the per-studio register link, so a
        // visitor can't reach another studio by editing a sequential id.
        // `gen_random_uuid()` (Postgres 13+) is a column default, so existing
        // rows are backfilled and every future insert gets one for free — no
        // app-side generation needed.
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE organizations \
             ADD COLUMN public_id uuid NOT NULL DEFAULT gen_random_uuid()",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE UNIQUE INDEX "idx-organizations-public_id" ON organizations (public_id)"#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE organizations DROP COLUMN public_id")
            .await?;
        Ok(())
    }
}
