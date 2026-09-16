use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::jobs::{JobRow, NewJob};

pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn enqueue(&self, new_job: &NewJob) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            "INSERT INTO jobs (kind, payload, payload_version, status, attempts, available_at, created_at, updated_at)
             VALUES (?, ?, ?, 'pending', 0, datetime('now'), datetime('now'), datetime('now'))",
        )
        .bind(&new_job.kind)
        .bind(&new_job.payload)
        .bind(new_job.payload_version)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn claim(&self, lease_duration_secs: i64) -> Result<Option<JobRow>, sqlx::Error> {
        self.reclaim_expired().await?;

        let row: Option<JobRow> = sqlx::query_as(
            "SELECT id, kind, payload, payload_version, status, attempts, max_attempts,
                    available_at, lease_token, leased_until, last_error, created_at, updated_at
             FROM jobs
             WHERE status = 'pending' AND available_at <= datetime('now')
             ORDER BY available_at ASC, id ASC
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(job) = row else {
            return Ok(None);
        };

        let lease_token = Uuid::new_v4().to_string();

        let updated = sqlx::query(
            "UPDATE jobs
             SET status = 'running',
                 attempts = attempts + 1,
                 lease_token = ?,
                 leased_until = datetime('now', ? || ' seconds'),
                 updated_at = datetime('now')
             WHERE id = ? AND status = 'pending'",
        )
        .bind(&lease_token)
        .bind(lease_duration_secs)
        .bind(job.id)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            return Ok(None);
        }

        let claimed: JobRow = sqlx::query_as(
            "SELECT id, kind, payload, payload_version, status, attempts, max_attempts,
                    available_at, lease_token, leased_until, last_error, created_at, updated_at
             FROM jobs WHERE id = ?",
        )
        .bind(job.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(claimed))
    }

    pub async fn renew_lease(
        &self,
        id: i64,
        lease_token: &str,
        lease_duration_secs: i64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE jobs
             SET leased_until = datetime('now', ? || ' seconds'),
                 updated_at = datetime('now')
             WHERE id = ? AND lease_token = ? AND status = 'running'",
        )
        .bind(lease_duration_secs)
        .bind(id)
        .bind(lease_token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn complete(&self, id: i64, lease_token: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE jobs
             SET status = 'completed',
                 lease_token = NULL,
                 leased_until = NULL,
                 updated_at = datetime('now')
             WHERE id = ? AND lease_token = ?",
        )
        .bind(id)
        .bind(lease_token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn retry(
        &self,
        id: i64,
        lease_token: &str,
        backoff_delay_secs: i64,
        error: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE jobs
             SET status = 'pending',
                 lease_token = NULL,
                 leased_until = NULL,
                 available_at = datetime('now', ? || ' seconds'),
                 last_error = ?,
                 updated_at = datetime('now')
             WHERE id = ? AND lease_token = ?",
        )
        .bind(backoff_delay_secs)
        .bind(error)
        .bind(id)
        .bind(lease_token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn terminal_failure(
        &self,
        id: i64,
        lease_token: &str,
        error: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE jobs
             SET status = 'failed',
                 lease_token = NULL,
                 leased_until = NULL,
                 last_error = ?,
                 updated_at = datetime('now')
             WHERE id = ? AND lease_token = ?",
        )
        .bind(error)
        .bind(id)
        .bind(lease_token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn reclaim_expired(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE jobs
             SET status = 'pending',
                 lease_token = NULL,
                 leased_until = NULL,
                 updated_at = datetime('now')
             WHERE status = 'running' AND leased_until < datetime('now')",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
