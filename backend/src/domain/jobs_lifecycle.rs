use std::fmt;
use std::str::FromStr;

use crate::domain::jobs::{JobFailure, JobRow};
use crate::domain::jobs_repo::JobRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownStatusError(String);

impl fmt::Display for UnknownStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown job status: {}", self.0)
    }
}

impl std::error::Error for UnknownStatusError {}

impl FromStr for JobStatus {
    type Err = UnknownStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(UnknownStatusError(other.to_string())),
        }
    }
}

pub struct JobLifecycleConfig {
    pub lease_duration_secs: i64,
    pub base_backoff_secs: i64,
    pub max_backoff_secs: i64,
    pub max_attempts: i32,
}

impl Default for JobLifecycleConfig {
    fn default() -> Self {
        Self {
            lease_duration_secs: 30,
            base_backoff_secs: 1,
            max_backoff_secs: 300,
            max_attempts: 3,
        }
    }
}

pub fn backoff_delay(attempts: i32, config: &JobLifecycleConfig) -> i64 {
    let exp = (attempts - 1).max(0) as u32;
    let delay = config.base_backoff_secs.saturating_mul(1i64 << exp);
    delay.min(config.max_backoff_secs)
}

pub async fn handle_job_outcome(
    repo: &JobRepository,
    job: &JobRow,
    lease_token: &str,
    result: Result<(), JobFailure>,
    config: &JobLifecycleConfig,
) -> Result<(), sqlx::Error> {
    match result {
        Ok(()) => {
            repo.complete(job.id, lease_token).await?;
        }
        Err(error) => {
            if job.attempts < job.max_attempts {
                let delay = backoff_delay(job.attempts as i32, config);
                repo.retry(job.id, lease_token, delay, error).await?;
            } else {
                repo.terminal_failure(job.id, lease_token, error).await?;
            }
        }
    }
    Ok(())
}
