use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub app: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
}

impl HealthResponse {
    pub fn new(app: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Ok,
            app: app.into(),
            version: version.into(),
        }
    }
}
