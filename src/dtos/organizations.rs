use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::_entities::organizations;

/// The wire shape of an organization (a yoga studio / tenant). Timestamps are
/// RFC 3339 strings so the JSON contract stays obvious and the frontend gets a
/// plain `string` rather than a chrono-specific type.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct Organization {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub timezone: String,
    /// The studio's public token — its `/studio/<token>` page and
    /// `/register/<token>` link. A user only ever sees their own studio here.
    pub public_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<organizations::Model> for Organization {
    fn from(o: organizations::Model) -> Self {
        Self {
            id: o.id,
            name: o.name,
            timezone: o.timezone,
            public_id: o.public_id.to_string(),
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

// Studios are created out-of-band (the `organization:create` task / seed), so
// there are no create/update request DTOs here — the API is read-only.

/// The only studio data exposed to an *unauthenticated* visitor on the
/// per-studio register page: just the name, to show "Join ⟨name⟩". The numeric
/// id never leaves the server (registration uses the URL token), and no
/// timezone/timestamps/member/class data leaks.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct PublicOrganization {
    pub name: String,
    /// The studio's LINE LIFF id if it has enabled LINE login, else an empty
    /// string. Public because the LIFF client needs it to initialise; the
    /// Channel ID (used server-side to verify tokens) is never exposed here.
    pub liff_id: String,
}

impl From<organizations::Model> for PublicOrganization {
    fn from(o: organizations::Model) -> Self {
        Self {
            name: o.name,
            liff_id: o.line_liff_id.unwrap_or_default(),
        }
    }
}
