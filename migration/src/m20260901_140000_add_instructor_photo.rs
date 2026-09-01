use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Storage key of the teacher's photo for this class (nullable — a class
        // may have none). The bytes live in the storage backend, not the DB.
        add_column(m, "classes", "instructor_photo", ColType::StringNull).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "classes", "instructor_photo").await
    }
}
