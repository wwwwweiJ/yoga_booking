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
    /// Whole currency units; 0 = free.
    #[ts(type = "number")]
    pub price: i32,
    /// Remaining seats = `capacity - current bookings`, never negative.
    #[ts(type = "number")]
    pub spots_left: i32,
    /// URL of the instructor photo, or null if none was uploaded.
    pub photo_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Class {
    /// Build the DTO from a class row plus how many bookings it already has.
    /// (`spots_left` can't come from the model alone, so there's no `From`.)
    #[must_use]
    pub fn from_parts(c: classes::Model, booked: i64) -> Self {
        let spots_left = i32::try_from((i64::from(c.capacity) - booked).max(0)).unwrap_or(0);
        let photo_url = c
            .instructor_photo
            .as_ref()
            .map(|_| format!("/api/classes/{}/photo", c.id));
        Self {
            id: c.id,
            organization_id: c.organization_id,
            title: c.title,
            instructor: c.instructor,
            starts_at: c.starts_at.to_rfc3339(),
            duration_minutes: c.duration_minutes,
            capacity: c.capacity,
            price: c.price,
            spots_left,
            photo_url,
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
    #[serde(default)]
    #[ts(type = "number")]
    pub price: i32,
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
    #[serde(default)]
    #[ts(type = "number")]
    pub price: i32,
}
