use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
#[allow(dead_code)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

#[allow(dead_code)]
impl AppError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({ "error": self.message });
        (self.status, axum::Json(body)).into_response()
    }
}

impl<E: std::fmt::Display> From<E> for AppError {
    fn from(err: E) -> Self {
        Self::internal(err.to_string())
    }
}
