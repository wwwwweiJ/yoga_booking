use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Two FKs: `user_id` and `class_id`, both NOT NULL / ON DELETE CASCADE
        // (cancel a class or delete a user → their bookings go too).
        create_table(
            m,
            "bookings",
            &[("id", ColType::PkAuto)],
            &[("users", ""), ("classes", "")],
        )
        .await?;

        // One booking per (user, class): the DB is the last line of defence
        // against a double-booking that slips past the controller check.
        m.create_index(
            Index::create()
                .name("idx-bookings-user_id-class_id")
                .table(Alias::new("bookings"))
                .col(Alias::new("user_id"))
                .col(Alias::new("class_id"))
                .unique()
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "bookings").await
    }
}
