use std::fmt;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
    pub lease_token: Option<String>,
    pub leased_until: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    Fixture,
}

impl fmt::Display for JobKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixture => write!(f, "fixture"),
        }
    }
}

impl FromStr for JobKind {
    type Err = JobError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fixture" => Ok(Self::Fixture),
            _ => Err(JobError::UnknownKind(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixturePayload {
    pub should_fail: bool,
}

impl JobPayload for FixturePayload {
    fn kind() -> JobKind {
        JobKind::Fixture
    }

    fn version() -> u32 {
        1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    UnknownKind(String),
    UnsupportedPayloadVersion { kind: JobKind, version: i64 },
    PayloadDeserialization { kind: JobKind, version: i64, error: String },
}

/// Maximum size, in bytes, of a diagnostic stored in `jobs.last_error`.
///
/// Persisted diagnostics are selected from the allowlisted messages below. They
/// intentionally exclude payload data, parser output, and handler-provided text,
/// any of which may contain secrets. Detailed errors may be handled in memory,
/// but must not cross this persistence boundary.
pub const MAX_PERSISTED_JOB_ERROR_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobFailure {
    UnknownKind,
    UnsupportedPayloadVersion,
    MalformedPayload,
    FixtureIntentionalFailure,
    HandlerFailure,
    LeaseExpiredAfterMaximumAttempts,
}

impl JobFailure {
    pub fn persisted_message(self) -> &'static str {
        let message = match self {
            Self::UnknownKind => "unknown job kind",
            Self::UnsupportedPayloadVersion => "unsupported payload version",
            Self::MalformedPayload => "malformed job payload",
            Self::FixtureIntentionalFailure => "fixture job intentionally failed",
            Self::HandlerFailure => "job handler failed",
            Self::LeaseExpiredAfterMaximumAttempts => {
                "lease expired after maximum attempts"
            }
        };

        if message.len() <= MAX_PERSISTED_JOB_ERROR_BYTES {
            message
        } else {
            // Preserve the storage bound even if a future allowlisted message is
            // accidentally made too long. Never truncate possibly sensitive text.
            "job failure"
        }
    }
}

impl From<&JobError> for JobFailure {
    fn from(error: &JobError) -> Self {
        match error {
            JobError::UnknownKind(_) => Self::UnknownKind,
            JobError::UnsupportedPayloadVersion { .. } => Self::UnsupportedPayloadVersion,
            JobError::PayloadDeserialization { .. } => Self::MalformedPayload,
        }
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind(kind) => write!(f, "unknown job kind: {kind}"),
            Self::UnsupportedPayloadVersion { kind, version } => {
                write!(f, "unsupported payload version {version} for job kind {kind}")
            }
            Self::PayloadDeserialization { kind, version, error } => {
                write!(f, "malformed payload JSON for {kind} v{version}: {error}")
            }
        }
    }
}

impl std::error::Error for JobError {}

pub trait JobPayload: Serialize + DeserializeOwned {
    fn kind() -> JobKind;
    fn version() -> u32;
}

pub struct NewJob {
    pub kind: String,
    pub payload: String,
    pub payload_version: i64,
}

pub fn enqueue<P: JobPayload>(payload: &P) -> Result<NewJob, serde_json::Error> {
    Ok(NewJob {
        kind: P::kind().to_string(),
        payload: serde_json::to_string(payload)?,
        payload_version: P::version() as i64,
    })
}

pub struct JobEnvelope {
    row: JobRow,
    kind: JobKind,
}

impl JobEnvelope {
    pub fn try_from_row(row: JobRow) -> Result<Self, JobError> {
        let kind: JobKind = row.kind.parse()?;
        Ok(Self { row, kind })
    }

    pub fn validate_payload<P: JobPayload>(&self) -> Result<(), JobError> {
        self.parse_payload::<P>().map(|_| ())
    }

    pub fn parse_payload<P: JobPayload>(&self) -> Result<P, JobError> {
        if self.kind != P::kind() {
            return Err(JobError::UnknownKind(self.row.kind.clone()));
        }
        if self.row.payload_version != P::version() as i64 {
            return Err(JobError::UnsupportedPayloadVersion {
                kind: self.kind.clone(),
                version: self.row.payload_version,
            });
        }
        serde_json::from_str::<P>(&self.row.payload).map_err(|e| JobError::PayloadDeserialization {
            kind: self.kind.clone(),
            version: self.row.payload_version,
            error: e.to_string(),
        })
    }

    pub fn kind(&self) -> &JobKind {
        &self.kind
    }

    pub fn row(&self) -> &JobRow {
        &self.row
    }

    pub fn id(&self) -> i64 {
        self.row.id
    }
}
