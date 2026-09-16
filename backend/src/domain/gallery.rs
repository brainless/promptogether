use api_types::gallery::{
    GalleryProjectDetail, GalleryProjectFile, GalleryProjectFilePhase, GalleryProjectSummary,
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GalleryProjectRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub prompt: String,
    pub publication_state: String,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GalleryFileRow {
    pub id: i64,
    pub project_id: i64,
    pub phase: String,
    pub path: String,
    pub language: Option<String>,
    pub content: String,
    pub display_order: i64,
}

impl From<GalleryProjectRow> for GalleryProjectSummary {
    fn from(row: GalleryProjectRow) -> Self {
        Self {
            slug: row.slug,
            title: row.title,
            summary: row.summary,
        }
    }
}

impl From<&GalleryFileRow> for GalleryProjectFile {
    fn from(row: &GalleryFileRow) -> Self {
        let phase = match row.phase.as_str() {
            "initial" => GalleryProjectFilePhase::Initial,
            _ => GalleryProjectFilePhase::Result,
        };
        Self {
            phase,
            path: row.path.clone(),
            language: row.language.clone(),
            content: row.content.clone(),
        }
    }
}

pub fn project_detail(row: GalleryProjectRow, files: Vec<GalleryFileRow>) -> GalleryProjectDetail {
    let initial_files = files
        .iter()
        .filter(|f| f.phase == "initial")
        .map(GalleryProjectFile::from)
        .collect();
    let result_files = files
        .iter()
        .filter(|f| f.phase == "result")
        .map(GalleryProjectFile::from)
        .collect();
    GalleryProjectDetail {
        slug: row.slug,
        title: row.title,
        summary: row.summary,
        prompt: row.prompt,
        created_at: row.created_at,
        updated_at: row.updated_at,
        initial_files,
        result_files,
    }
}
