use loco_rs::prelude::*;
use sea_orm::entity::prelude::*;
use serde::Deserialize;

pub use super::_entities::classes::{ActiveModel, Entity, Model};
pub type Classes = Entity;

/// A scheduled, bookable class session. `capacity` and `duration_minutes` must
/// be positive; the `organization_id` FK is enforced by the database (and the
/// controller checks the studio exists so a bad id is a 4xx, not a 500).
#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 2, message = "Title must be at least 2 characters long."))]
    pub title: String,
    #[validate(length(min = 2, message = "Instructor must be at least 2 characters long."))]
    pub instructor: String,
    #[validate(range(min = 1, message = "Duration must be at least 1 minute."))]
    pub duration_minutes: i32,
    #[validate(range(min = 1, message = "Capacity must be at least 1."))]
    pub capacity: i32,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            title: self.title.as_ref().to_owned(),
            instructor: self.instructor.as_ref().to_owned(),
            duration_minutes: *self.duration_minutes.as_ref(),
            capacity: *self.capacity.as_ref(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.validate()?;
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
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
