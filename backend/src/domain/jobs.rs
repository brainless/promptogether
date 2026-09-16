#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JobRow {
    pub id: i64,
    pub kind: String,
    pub payload: String,
    pub payload_version: i64,
    pub status: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub available_at: String,
    pub leased_until: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
