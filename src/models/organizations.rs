use loco_rs::prelude::*;
use sea_orm::entity::prelude::*;
use serde::Deserialize;

pub use super::_entities::organizations::{ActiveModel, Column, Entity, Model};
pub type Organizations = Entity;

/// A tenant (a yoga studio). `timezone` is an IANA name (e.g.
/// `Asia/Taipei`) and anchors every schedule and booking the org owns, so it
/// must always be present.
#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 2, message = "Name must be at least 2 characters long."))]
    pub name: String,
    #[validate(length(min = 1, message = "Timezone must be present."))]
    pub timezone: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            name: self.name.as_ref().to_owned(),
            timezone: self.timezone.as_ref().to_owned(),
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
impl Model {
    /// Resolve a studio by its public register token (the value in a
    /// `/register/<token>` link), or `EntityNotFound`.
    pub async fn find_by_public_id(db: &DatabaseConnection, public_id: &Uuid) -> ModelResult<Self> {
        Entity::find()
            .filter(Column::PublicId.eq(*public_id))
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
