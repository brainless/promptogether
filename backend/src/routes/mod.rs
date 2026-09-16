mod health;
mod ping;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/ping", get(ping::ping))
        .route("/api/health", get(health::health))
}