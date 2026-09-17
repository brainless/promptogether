use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::watch;

use crate::config::WorkerConfig;
use crate::domain::jobs::{JobEnvelope, JobFailure, JobKind, JobRow};
use crate::domain::jobs_lifecycle::{handle_job_outcome, JobLifecycleConfig};
use crate::domain::jobs_repo::JobRepository;

#[derive(Debug)]
pub enum WorkerError {
    Database(sqlx::Error),
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for WorkerError {}

impl From<sqlx::Error> for WorkerError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

pub async fn run(pool: SqlitePool, config: WorkerConfig) -> Result<(), WorkerError> {
    let repo = JobRepository::new(&pool);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency));
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    let mut handles = Vec::new();

    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        let _ = shutdown_tx_clone.send(true);
    });

    tracing::info!(
        concurrency = config.concurrency,
        poll_interval_ms = config.poll_interval_ms,
        lease_duration_secs = config.lease_duration_secs,
        shutdown_timeout_secs = config.shutdown_timeout_secs,
        "starting worker"
    );

    loop {
        handles.retain(|h: &tokio::task::JoinHandle<_>| !h.is_finished());

        if *shutdown_rx.borrow() {
            break;
        }

        let permit = tokio::select! {
            _ = shutdown_rx.changed() => break,
            permit = semaphore.clone().acquire_owned() => {
                match permit {
                    Ok(permit) => permit,
                    Err(_) => break,
                }
            }
        };

        match repo.claim(config.lease_duration_secs).await {
            Ok(Some(job)) => {
                let pool_clone = pool.clone();
                let config_clone = config.clone();
                let handle = tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(err) = run_handler(pool_clone, job, config_clone).await {
                        tracing::error!(error = %err, "handler error");
                    }
                });
                handles.push(handle);
            }
            Ok(None) => {
                drop(permit);
                tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    _ = tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)) => {}
                }
            }
            Err(err) => {
                drop(permit);
                tracing::error!(error = %err, "failed to claim job");
                tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    _ = tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)) => {}
                }
            }
        }
    }

    drop(shutdown_tx);

    tracing::info!("waiting for active handlers to finish");
    let timeout = Duration::from_secs(config.shutdown_timeout_secs);
    let deadline = tokio::time::Instant::now() + timeout;

    for mut handle in handles {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            handle.abort();
            continue;
        }
        tokio::select! {
            result = &mut handle => {
                match result {
                    Ok(_) => {}
                    Err(err) if err.is_cancelled() => {
                        tracing::warn!("handler was cancelled during shutdown");
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "handler panicked during shutdown");
                    }
                }
            }
            _ = tokio::time::sleep(remaining) => {
                handle.abort();
                tracing::warn!("handler timed out during shutdown");
            }
        }
    }

    tracing::info!("worker stopped");
    Ok(())
}

async fn run_handler(
    pool: SqlitePool,
    job: JobRow,
    config: WorkerConfig,
) -> Result<(), WorkerError> {
    let lease_token = match &job.lease_token {
        Some(token) => token.clone(),
        None => {
            tracing::error!(job_id = job.id, "claimed job has no lease token");
            return Ok(());
        }
    };

    tracing::info!(
        job_id = job.id,
        kind = %job.kind,
        payload_version = job.payload_version,
        attempt = job.attempts,
        "job started"
    );

    let start = tokio::time::Instant::now();

    let renewal_interval =
        Duration::from_secs((config.lease_duration_secs as u64 / 2).max(1));
    let result = {
        let handler = dispatch_job(&job);
        tokio::pin!(handler);
        let mut renewal_enabled = true;

        loop {
            tokio::select! {
                result = &mut handler => break result,
                _ = tokio::time::sleep(renewal_interval), if renewal_enabled => {
                    let repo = JobRepository::new(&pool);
                    match repo
                        .renew_lease(job.id, &lease_token, config.lease_duration_secs)
                        .await
                    {
                        Ok(true) => {
                            tracing::debug!(job_id = job.id, "lease renewed");
                        }
                        Ok(false) => {
                            tracing::warn!(
                                job_id = job.id,
                                "lease renewal failed, job may have been reclaimed"
                            );
                            renewal_enabled = false;
                        }
                        Err(err) => {
                            tracing::error!(error = %err, job_id = job.id, "failed to renew lease");
                            renewal_enabled = false;
                        }
                    }
                }
            }
        }
    };
    let duration_ms = start.elapsed().as_millis() as u64;

    let outcome = if result.is_ok() { "completed" } else { "failed" };

    let repo = JobRepository::new(&pool);
    let lifecycle_config = JobLifecycleConfig {
        lease_duration_secs: config.lease_duration_secs,
        ..Default::default()
    };
    handle_job_outcome(&repo, &job, &lease_token, result, &lifecycle_config).await?;

    tracing::info!(
        job_id = job.id,
        kind = %job.kind,
        payload_version = job.payload_version,
        attempt = job.attempts,
        duration_ms = duration_ms,
        outcome = outcome,
        "job finished"
    );

    Ok(())
}

async fn dispatch_job(job: &JobRow) -> Result<(), JobFailure> {
    use crate::domain::jobs::FixturePayload;

    let envelope = JobEnvelope::try_from_row(job.clone())
        .map_err(|error| JobFailure::from(&error))?;
    match envelope.kind() {
        JobKind::Fixture => {
            let payload: FixturePayload = envelope
                .parse_payload()
                .map_err(|error| JobFailure::from(&error))?;
            if payload.should_fail {
                Err(JobFailure::FixtureIntentionalFailure)
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");
        tracing::info!("received SIGINT, shutting down worker");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
        tracing::info!("received SIGTERM, shutting down worker");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
