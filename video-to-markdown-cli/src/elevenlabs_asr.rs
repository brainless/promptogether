//! Optional Scribe v2 fallback for MP3 chunks filtered by Xiaomi ASR.
//!
//! Only the affected chunk is sent to ElevenLabs. The API key and transcript
//! are never printed, and the raw transcript remains in memory only until
//! Xiaomi cleanup has consumed it.

use std::path::Path;

use llm_sdk::elevenlabs::{ElevenLabsClient, TranscriptionRequest, TranscriptionResult};
use llm_sdk::error::LlmError;

pub const API_KEY_ENV_VAR: &str = "ELEVENLABS_API_KEY";

#[derive(Debug)]
pub enum ElevenLabsAsrError {
    MissingApiKey,
    AudioUnreadable(String),
    Authentication,
    RateLimit,
    InvalidRequest(String),
    Network(String),
    Provider(u16),
    MalformedResponse(String),
    UnexpectedResponse,
    EmptyTranscript,
}

impl std::fmt::Display for ElevenLabsAsrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingApiKey => write!(f, "{API_KEY_ENV_VAR} is not set"),
            Self::AudioUnreadable(reason) => write!(f, "cannot read filtered MP3 chunk: {reason}"),
            Self::Authentication => write!(f, "ElevenLabs rejected {API_KEY_ENV_VAR}"),
            Self::RateLimit => write!(f, "ElevenLabs rate limit exceeded"),
            Self::InvalidRequest(reason) => write!(f, "ElevenLabs rejected request: {reason}"),
            Self::Network(reason) => write!(f, "network error calling ElevenLabs: {reason}"),
            Self::Provider(status) => write!(f, "ElevenLabs returned HTTP status {status}"),
            Self::MalformedResponse(reason) => {
                write!(f, "ElevenLabs returned a malformed response: {reason}")
            }
            Self::UnexpectedResponse => write!(
                f,
                "ElevenLabs returned a webhook or multichannel response instead of a transcript"
            ),
            Self::EmptyTranscript => write!(f, "ElevenLabs returned an empty transcript"),
        }
    }
}

impl std::error::Error for ElevenLabsAsrError {}

pub fn api_key_from_env() -> Result<String, ElevenLabsAsrError> {
    match std::env::var(API_KEY_ENV_VAR) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(ElevenLabsAsrError::MissingApiKey),
    }
}

fn map_llm_error(err: LlmError) -> ElevenLabsAsrError {
    match err {
        LlmError::Authentication { .. } => ElevenLabsAsrError::Authentication,
        LlmError::RateLimit { .. } => ElevenLabsAsrError::RateLimit,
        LlmError::InvalidRequest { message } => ElevenLabsAsrError::InvalidRequest(message),
        LlmError::Network { source } => ElevenLabsAsrError::Network(source.to_string()),
        LlmError::Api { status, .. } => ElevenLabsAsrError::Provider(status),
        LlmError::Parse { source } => ElevenLabsAsrError::MalformedResponse(source.to_string()),
        LlmError::Internal { message } => ElevenLabsAsrError::MalformedResponse(message),
        _ => ElevenLabsAsrError::UnexpectedResponse,
    }
}

pub async fn transcribe(
    mp3_path: &Path,
    api_key: &str,
    diagnostics: bool,
) -> Result<String, ElevenLabsAsrError> {
    let audio = std::fs::read(mp3_path)
        .map_err(|err| ElevenLabsAsrError::AudioUnreadable(err.to_string()))?;
    if diagnostics {
        eprintln!(
            "diagnostics: ElevenLabs request MP3 {} bytes, language en",
            audio.len()
        );
    }

    let client = ElevenLabsClient::new(api_key).map_err(map_llm_error)?;
    let mut request = TranscriptionRequest::file(audio, "filtered-chunk.mp3");
    request.language_code = Some("en".to_string());
    request.tag_audio_events = Some(false);
    request.no_verbatim = Some(false);
    let response = client.transcribe(request).await.map_err(map_llm_error)?;
    let transcript = match response {
        TranscriptionResult::Transcript(transcript) => transcript,
        TranscriptionResult::Multichannel(_) | TranscriptionResult::Accepted(_) => {
            return Err(ElevenLabsAsrError::UnexpectedResponse);
        }
    };
    if diagnostics {
        eprintln!(
            "diagnostics: ElevenLabs response language={}, text_bytes={}, audio_duration_secs={}",
            transcript.language_code,
            transcript.text.len(),
            transcript
                .audio_duration_secs
                .map(|seconds| format!("{seconds:.1}"))
                .unwrap_or_else(|| "missing".to_string())
        );
    }
    let text = transcript.text.trim();
    if text.is_empty() {
        return Err(ElevenLabsAsrError::EmptyTranscript);
    }
    Ok(text.to_string())
}
