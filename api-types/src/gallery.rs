use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A lightweight entry for a single prompt-gallery project as shown in lists.
///
/// Identifiers and timestamps are exposed as strings on the wire so the backend
/// can evolve its underlying key and time representation without breaking the
/// public JSON contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct GalleryProjectSummary {
    /// Stable slug used as the project identifier on the wire.
    pub slug: String,
    pub title: String,
    pub summary: String,
}

/// The phase of a project file in the gallery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
pub enum GalleryProjectFilePhase {
    Initial,
    Result,
}

/// A single file belonging to a gallery project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct GalleryProjectFile {
    pub phase: GalleryProjectFilePhase,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub content: String,
}

/// Full detail for a single gallery project, including the prompt and its
/// ordered initial and result files.
///
/// Timestamps are exposed as strings on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct GalleryProjectDetail {
    pub slug: String,
    pub title: String,
    pub summary: String,
    /// The prompt text that a coding agent was given for this project.
    pub prompt: String,
    pub created_at: String,
    pub updated_at: String,
    pub initial_files: Vec<GalleryProjectFile>,
    pub result_files: Vec<GalleryProjectFile>,
}
