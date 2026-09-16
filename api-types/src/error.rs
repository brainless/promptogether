use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// A shared error envelope returned by the HTTP API.
///
/// The `code` field is a short, machine-readable error kind (e.g. `not_found`),
/// `message` is a human-readable explanation, and `details` may carry
/// structured, endpoint-specific information.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional = nullable)]
    pub details: Option<Value>,
}
