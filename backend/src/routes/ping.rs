use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn ping(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "app": crate::domain::app_name(),
        "version": crate::domain::app_version(),
        "bind_address": state.config.bind_address.to_string(),
    }))
}
