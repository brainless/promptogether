use axum::{
    extract::{rejection::PathRejection, Path, State},
    Json,
};

use api_types::gallery::{GalleryProjectDetail, GalleryProjectSummary};

use crate::domain::gallery::{project_detail, GalleryFileRow, GalleryProjectRow, GalleryProjectSummaryRow};
use crate::error::AppError;
use crate::state::AppState;

fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty() || slug.len() > 100 {
        return Err(AppError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_gallery_slug",
            "Slug must be 1-100 characters.",
        ));
    }

    let mut prev_was_dash = false;
    for ch in slug.chars() {
        match ch {
            'a'..='z' | '0'..='9' => {
                prev_was_dash = false;
            }
            '-' => {
                if prev_was_dash {
                    return Err(AppError::new(
                        axum::http::StatusCode::BAD_REQUEST,
                        "invalid_gallery_slug",
                        "Slug must not contain consecutive dashes.",
                    ));
                }
                prev_was_dash = true;
            }
            _ => {
                return Err(AppError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    "invalid_gallery_slug",
                    "Slug must contain only lowercase ASCII letters, digits, and hyphens.",
                ));
            }
        }
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(AppError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_gallery_slug",
            "Slug must not start or end with a hyphen.",
        ));
    }

    Ok(())
}

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<GalleryProjectSummary>>, AppError> {
    let rows: Vec<GalleryProjectSummaryRow> = sqlx::query_as(
        "SELECT slug, title, summary
         FROM gallery_projects
         WHERE publication_state = 'published'
         ORDER BY display_order ASC, slug ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    let summaries: Vec<GalleryProjectSummary> = rows.into_iter().map(GalleryProjectSummary::from).collect();
    Ok(Json(summaries))
}

pub async fn get_project(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<GalleryProjectDetail>, AppError> {
    let Path(slug) = path.map_err(|_| {
        AppError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_gallery_slug",
            "Slug must be valid UTF-8.",
        )
    })?;

    validate_slug(&slug)?;

    let row: GalleryProjectRow = sqlx::query_as(
        "SELECT id, slug, title, summary, prompt, publication_state, display_order, created_at, updated_at
         FROM gallery_projects
         WHERE slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        AppError::new(
            axum::http::StatusCode::NOT_FOUND,
            "gallery_project_not_found",
            "Gallery project not found.",
        )
    })?;

    if row.publication_state != "published" {
        return Err(AppError::new(
            axum::http::StatusCode::NOT_FOUND,
            "gallery_project_not_found",
            "Gallery project not found.",
        ));
    }

    let files: Vec<GalleryFileRow> = sqlx::query_as(
        "SELECT id, project_id, phase, path, language, content, display_order
         FROM gallery_files
         WHERE project_id = ?
         ORDER BY display_order ASC, path ASC, id ASC",
    )
    .bind(row.id)
    .fetch_all(&state.pool)
    .await?;

    for file in &files {
        if file.phase != "initial" && file.phase != "result" {
            tracing::error!(
                project_id = row.id,
                file_id = file.id,
                phase = %file.phase,
                "unknown gallery file phase"
            );
            return Err(AppError::internal("An internal error occurred."));
        }
    }

    Ok(Json(project_detail(row, files)))
}
