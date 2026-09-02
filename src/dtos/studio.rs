use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One section of a studio's customizable page. Tagged by `type`, so the
/// frontend sees a discriminated union it can render per block kind.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub enum Block {
    Hero { heading: String, subheading: String },
    About { text: String },
    Gallery { images: Vec<String> },
    /// Renders the studio's upcoming classes on the public page.
    Schedule { heading: String },
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

/// The stored column may be null (no page yet) or — defensively — unreadable;
/// either way, treat it as an empty page.
#[must_use]
pub fn blocks_from_value(page: Option<serde_json::Value>) -> Vec<Block> {
    page.and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}
