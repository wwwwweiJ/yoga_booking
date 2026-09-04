use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::_entities::{organizations, users};

/// A studio as the operator sees it — includes the `public_id` register token
/// so the admin can hand out `/register/<public_id>` links.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct AdminOrganization {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub timezone: String,
    pub public_id: String,
}

impl From<organizations::Model> for AdminOrganization {
    fn from(o: organizations::Model) -> Self {
        Self {
            id: o.id,
            name: o.name,
            timezone: o.timezone,
            public_id: o.public_id.to_string(),
        }
    }
}

/// Body for creating a studio from the backoffice.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateOrganizationParams {
    pub name: String,
    pub timezone: String,
}

/// Body for minting a teacher (staff) account for a studio.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateStaffParams {
    #[ts(type = "number")]
    pub organization_id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
}

/// A user as returned to the operator after creating them.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct AdminUser {
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl From<users::Model> for AdminUser {
    fn from(u: users::Model) -> Self {
        Self {
            pid: u.pid.to_string(),
            name: u.name,
            email: u.email,
            role: u.role,
        }
    }
}

/// A user in a studio, for the backoffice roster + role management. `is_line`
/// flags LINE-authenticated accounts (whose email is a synthetic placeholder),
/// so the UI can show a LINE badge instead of that address.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct AdminUserListItem {
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_line: bool,
}

impl From<users::Model> for AdminUserListItem {
    fn from(u: users::Model) -> Self {
        Self {
            pid: u.pid.to_string(),
            name: u.name,
            email: u.email,
            role: u.role,
            is_line: u.line_user_id.is_some(),
        }
    }
}

/// Body for changing a user's role from the backoffice (student ↔ teacher).
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct SetRoleParams {
    pub role: String,
}
