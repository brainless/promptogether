use api_types::HealthResponse;
use axum::Json;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::new(
        crate::domain::app_name(),
        crate::domain::app_version(),
    ))
}