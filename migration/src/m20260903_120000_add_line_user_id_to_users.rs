use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // LINE-authenticated users are keyed by their LINE id, but scoped to a
        // single studio — the same person at two studios is two rows (the app's
        // one-user-per-studio rule). Email-only users leave this NULL; a PARTIAL
        // unique index enforces uniqueness only on the rows that have a value, so
        // the many NULLs never collide (a plain unique index would forbid two
        // email users, and a global one would forbid one LINE person at two
        // studios).
        add_column(m, "users", "line_user_id", ColType::StringNull).await?;
        m.get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX "idx-users-org-line_user_id" ON users (organization_id, line_user_id) WHERE line_user_id IS NOT NULL"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "idx-users-org-line_user_id""#)
            .await?;
        remove_column(m, "users", "line_user_id").await
    }
}
