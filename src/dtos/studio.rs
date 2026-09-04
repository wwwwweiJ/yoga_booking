use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One section of a studio's customizable page. Tagged by `type`, so the
/// frontend sees a discriminated union it can render per block kind.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub enum Block {
    Hero {
        heading: String,
        subheading: String,
        /// Optional background image URL (uploaded via `/api/studio/uploads`).
        /// Older stored heroes without this field deserialize to `None`.
        #[serde(default)]
        image: Option<String>,
    },
    About {
        text: String,
    },
    Gallery {
        images: Vec<String>,
    },
    /// Renders the studio's upcoming classes on the public page.
    Schedule {
        heading: String,
    },
    /// A "meet the teachers" section: each member has a photo + bio.
    Teachers {
        heading: String,
        members: Vec<TeacherIntro>,
    },
}

/// One teacher shown in a `Teachers` block. Everything but the name is optional
/// (blank when unset), and every field has `#[serde(default)]` so partial
/// entries and older stored blocks deserialize cleanly.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct TeacherIntro {
    pub name: String,
    /// Uploaded photo URL (via `/api/studio/uploads`).
    #[serde(default)]
    pub photo: Option<String>,
    /// Speciality or title, e.g. "陰瑜伽 · 10 年經驗".
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub instagram: String,
    #[serde(default)]
    pub website: String,
}

/// A studio's page: its name plus the ordered blocks that render it.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct StudioPage {
    pub name: String,
    pub blocks: Vec<Block>,
}

/// Body for replacing a studio's page.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UpdatePageParams {
    pub blocks: Vec<Block>,
}

/// Result of uploading an image for a studio page (e.g. a gallery photo): a
/// public URL the block can reference.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct UploadedFile {
    pub url: String,
}

/// A studio's LINE login settings, shown/edited by the studio's own staff
/// (also the PUT body). Both empty when LINE isn't set up. Unlike the public
/// studio payload, this includes the Channel ID — it is only ever returned to
/// the studio's own authenticated staff.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct StudioLineSettings {
    pub liff_id: String,
    pub channel_id: String,
}

/// The stored column may be null (no page yet) or — defensively — unreadable;
/// either way, treat it as an empty page.
#[must_use]
pub fn blocks_from_value(page: Option<serde_json::Value>) -> Vec<Block> {
    page.and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}
