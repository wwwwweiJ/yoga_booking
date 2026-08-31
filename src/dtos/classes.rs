use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::_entities::classes;

/// The wire shape of a class session. `starts_at` is an RFC 3339 string (same
/// policy as the timestamps in [`crate::dtos::organizations::Organization`]).
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct Class {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub organization_id: i64,
    pub title: String,
    pub instructor: String,
    pub starts_at: String,
    #[ts(type = "number")]
    pub duration_minutes: i32,
    #[ts(type = "number")]
    pub capacity: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<classes::Model> for Class {
    fn from(c: classes::Model) -> Self {
        Self {
            id: c.id,
            organization_id: c.organization_id,
            title: c.title,
            instructor: c.instructor,
            starts_at: c.starts_at.to_rfc3339(),
            duration_minutes: c.duration_minutes,
            capacity: c.capacity,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

/// Body for creating a class. The owning studio is implicit — always the
/// authenticated user's — so it is not part of the request. `starts_at` is
/// parsed from RFC 3339 in the controller.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateClassParams {
    pub title: String,
    pub instructor: String,
    pub starts_at: String,
    #[ts(type = "number")]
    pub duration_minutes: i32,
    #[ts(type = "number")]
    pub capacity: i32,
}

/// Body for a full replacement (PUT) of a class. The owning studio is fixed at
/// creation, so it is not part of the update.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UpdateClassParams {
    pub title: String,
    pub instructor: String,
    pub starts_at: String,
    #[ts(type = "number")]
    pub duration_minutes: i32,
    #[ts(type = "number")]
    pub capacity: i32,
}
