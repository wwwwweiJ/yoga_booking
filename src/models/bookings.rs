use std::collections::HashMap;

use loco_rs::prelude::*;
use sea_orm::{entity::prelude::*, QuerySelect};

pub use super::_entities::bookings::{ActiveModel, Column, Entity, Model};
pub type Bookings = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {
    /// How many bookings each of the given classes currently has, keyed by
    /// class id (classes with none are simply absent). One query, then tally in
    /// memory — no N+1 across a page of classes.
    pub async fn counts_by_class(
        db: &DatabaseConnection,
        class_ids: &[i64],
    ) -> ModelResult<HashMap<i64, i64>> {
        if class_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let class_ids: Vec<i64> = Entity::find()
            .select_only()
            .column(Column::ClassId)
            .filter(Column::ClassId.is_in(class_ids.to_vec()))
            .into_tuple()
            .all(db)
            .await?;

        let mut counts = HashMap::new();
        for class_id in class_ids {
            *counts.entry(class_id).or_insert(0) += 1;
        }
        Ok(counts)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
