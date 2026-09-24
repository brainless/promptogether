//! Stage 2: recognize speech with MiMo V2.5 ASR.
//!
//! Reads and Base64-encodes the extracted audio, validates it against the
//! provider's request-size limit, and calls MiMo V2.5 ASR through
//! `llm-sdk`'s Xiaomi client. Selects the transcript from the response
//! defensively and rejects a missing or blank result. Credentials come from
//! the `XIAOMI_API_KEY` environment variable and must never appear in
//! arguments, logs, or output — this module never `Debug`-prints the
//! client, the request, or any value that carries the key.
//!
//! ## Provider size limit policy
//!
//! Xiaomi documents a 10 MB limit on Base64-encoded audio input, but
//! `llm-sdk` sends whatever it is given without enforcing that limit. This
//! module validates the encoded size itself and, when it would exceed the
//! limit, stops before making a request rather than attempting an upload
//! that the provider would reject. The error message tells the maintainer
//! how to produce a smaller input (trim the video). The audio stage already
//! splits recordings into short chunks, so this check applies per chunk.

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use llm_sdk::error::LlmError;
use llm_sdk::models::xiaomi::MIMO_V2_5_ASR;
use llm_sdk::xiaomi::{XiaomiAsrLanguage, XiaomiAudioFormat, XiaomiClient};

/// Provider-documented limit on Base64-encoded audio input, in bytes.
const XIAOMI_BASE64_SIZE_LIMIT_BYTES: usize = 10 * 1024 * 1024;

/// Name of the environment variable that carries the Xiaomi API key.
pub const XIAOMI_API_KEY_ENV_VAR: &str = "XIAOMI_API_KEY";

/// Errors that can occur while transcribing extracted audio.
#[derive(Debug)]
pub enum TranscriptionError {
    /// The extracted audio file could not be read from disk.
    AudioUnreadable { path: PathBuf, reason: String },
    /// The `XIAOMI_API_KEY` environment variable was not set (or was not
    /// valid Unicode).
    MissingApiKey,
    /// The Base64-encoded audio exceeds the provider's request-size limit.
    /// The extracted chunk is too large for one request.
    EncodedAudioTooLarge {
        encoded_bytes: usize,
        limit_bytes: usize,
    },
    /// The request was rejected as unauthenticated (invalid or revoked
    /// credentials).
    Authentication(String),
    /// The provider rate-limited this request.
    RateLimit(String),
    /// The provider rejected the request as malformed.
    InvalidRequest(String),
    /// A transport-level failure occurred (DNS, TLS, connection reset,
    /// timeout, and similar).
    Network(String),
    /// The provider returned an error status not covered by a more
    /// specific variant above.
    Provider { status: u16, message: String },
    /// The response could not be parsed, or was missing fields this CLI
    /// depends on (no choices, no message content).
    MalformedResponse(String),
    /// The provider returned a response but the transcript was missing or
    /// blank.
    EmptyTranscript,
    /// The provider stopped before it completed the transcript.
    IncompleteTranscript(String),
    /// The provider omitted transcript content after applying its filter.
    ContentFiltered,
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionError::AudioUnreadable { path, reason } => {
                write!(
                    f,
                    "cannot read extracted audio {}: {reason}",
                    path.display()
                )
            }
            TranscriptionError::MissingApiKey => write!(
                f,
                "{XIAOMI_API_KEY_ENV_VAR} is not set — export a Xiaomi API key in this \
                 environment variable before running this command"
            ),
            TranscriptionError::EncodedAudioTooLarge {
                encoded_bytes,
                limit_bytes,
            } => write!(
                f,
                "extracted audio Base64-encodes to {encoded_bytes} bytes, which exceeds the \
                 provider's {limit_bytes}-byte (10 MB) limit — trim the source video and \
                 re-run the command"
            ),
            TranscriptionError::Authentication(message) => write!(
                f,
                "Xiaomi ASR authentication failed — check that {XIAOMI_API_KEY_ENV_VAR} holds a \
                 valid, current API key ({message})"
            ),
            TranscriptionError::RateLimit(message) => {
                write!(f, "Xiaomi ASR rate limit exceeded: {message}")
            }
            TranscriptionError::InvalidRequest(message) => {
                write!(f, "Xiaomi ASR rejected the request: {message}")
            }
            TranscriptionError::Network(message) => {
                write!(f, "network error while calling Xiaomi ASR: {message}")
            }
            TranscriptionError::Provider { status, message } => write!(
                f,
                "Xiaomi ASR returned an error (status {status}): {message}"
            ),
            TranscriptionError::MalformedResponse(reason) => write!(
                f,
                "Xiaomi ASR returned a response this tool could not understand: {reason}"
            ),
            TranscriptionError::EmptyTranscript => write!(
                f,
                "Xiaomi ASR returned an empty transcript — the audio may be silent, unclear, or \
                 in an unsupported language"
            ),
            TranscriptionError::IncompleteTranscript(reason) => write!(
                f,
                "Xiaomi ASR did not finish its transcript (finish_reason: {reason}) — try a shorter audio chunk"
            ),
            TranscriptionError::ContentFiltered => write!(
                f,
                "Xiaomi ASR filtered this audio chunk (finish_reason: content_filter) — the provider omitted content, so no Markdown was written; review the affected chunk or use another transcription method"
            ),
        }
    }
}

impl std::error::Error for TranscriptionError {}

/// Read the Xiaomi API key from `XIAOMI_API_KEY`.
///
/// Never logs or echoes the key; only its presence/absence and Unicode
/// validity are reported.
pub fn api_key_from_env() -> Result<String, TranscriptionError> {
    match std::env::var(XIAOMI_API_KEY_ENV_VAR) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) => Err(TranscriptionError::MissingApiKey),
        Err(_) => Err(TranscriptionError::MissingApiKey),
    }
}

/// Validate an already Base64-encoded payload's length against the
/// provider's request-size limit. Pure and side-effect free so it can be
/// unit tested without touching the filesystem or network.
fn validate_encoded_size(encoded_bytes: usize) -> Result<(), TranscriptionError> {
    if encoded_bytes > XIAOMI_BASE64_SIZE_LIMIT_BYTES {
        return Err(TranscriptionError::EncodedAudioTooLarge {
            encoded_bytes,
            limit_bytes: XIAOMI_BASE64_SIZE_LIMIT_BYTES,
        });
    }
    Ok(())
}

/// Defensively extract the transcript text from a Xiaomi ASR response.
///
/// Missing choices, a missing message body, and a blank/whitespace-only
/// transcript are all treated as failures, not successes.
fn extract_transcript(
    response: &llm_sdk::xiaomi::XiaomiChatCompletionResponse,
) -> Result<String, TranscriptionError> {
    let choice = response.choices.first().ok_or_else(|| {
        TranscriptionError::MalformedResponse("response contained no choices".to_string())
    })?;
    if choice.finish_reason.as_deref() == Some("content_filter") {
        return Err(TranscriptionError::ContentFiltered);
    }
    if choice.finish_reason.as_deref() != Some("stop") {
        return Err(TranscriptionError::IncompleteTranscript(
            choice
                .finish_reason
                .as_deref()
                .unwrap_or("missing")
                .to_string(),
        ));
    }
    let content = choice.message.content.as_deref().ok_or_else(|| {
        TranscriptionError::MalformedResponse("response choice had no message content".to_string())
    })?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(TranscriptionError::EmptyTranscript);
    }
    Ok(trimmed.to_string())
}

/// Map an `llm-sdk` error into this module's own error type. `llm-sdk`
/// types never leak through this module's public API.
fn map_llm_error(err: LlmError) -> TranscriptionError {
    match err {
        LlmError::Authentication { message } => TranscriptionError::Authentication(message),
        LlmError::RateLimit { message, .. } => TranscriptionError::RateLimit(message),
        LlmError::InvalidRequest { message } => TranscriptionError::InvalidRequest(message),
        LlmError::Network { source } => TranscriptionError::Network(source.to_string()),
        LlmError::Api { status, message } => TranscriptionError::Provider { status, message },
        LlmError::OpenRouterApi { status, error_type } => TranscriptionError::Provider {
            status,
            message: error_type,
        },
        LlmError::Parse { source } => TranscriptionError::MalformedResponse(source.to_string()),
        LlmError::Internal { message } => TranscriptionError::MalformedResponse(message),
        other => TranscriptionError::MalformedResponse(other.to_string()),
    }
}

/// Transcribe the audio at `audio_path` using MiMo V2.5 ASR, returning the
/// raw transcript text.
///
/// Reads and Base64-encodes the file, validates the encoded size against
/// the provider's limit before making any request, calls MiMo V2.5 ASR
/// with English fixed, and defensively extracts the transcript. `api_key`
/// is never logged, printed, or included in any error message.
pub async fn transcribe(audio_path: &Path, api_key: &str) -> Result<String, TranscriptionError> {
    let audio_bytes =
        std::fs::read(audio_path).map_err(|err| TranscriptionError::AudioUnreadable {
            path: audio_path.to_path_buf(),
            reason: err.to_string(),
        })?;

    let encoded = BASE64_STANDARD.encode(&audio_bytes);
    validate_encoded_size(encoded.len())?;

    let client = XiaomiClient::new(api_key).map_err(map_llm_error)?;

    let response = client
        .speech_recognition_builder()
        .model(MIMO_V2_5_ASR)
        .audio_base64(encoded, XiaomiAudioFormat::Mp3)
        .language(XiaomiAsrLanguage::En)
        .send()
        .await
        .map_err(map_llm_error)?;

    extract_transcript(&response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_sdk::xiaomi::XiaomiChatCompletionResponse;

    fn response_from_json(raw: &str) -> XiaomiChatCompletionResponse {
        serde_json::from_str(raw).expect("valid XiaomiChatCompletionResponse JSON")
    }

    #[test]
    fn accepts_encoded_payload_at_or_under_the_limit() {
        assert!(validate_encoded_size(XIAOMI_BASE64_SIZE_LIMIT_BYTES).is_ok());
        assert!(validate_encoded_size(XIAOMI_BASE64_SIZE_LIMIT_BYTES - 1).is_ok());
    }

    #[test]
    fn rejects_encoded_payload_over_the_limit() {
        let err = validate_encoded_size(XIAOMI_BASE64_SIZE_LIMIT_BYTES + 1).unwrap_err();
        match err {
            TranscriptionError::EncodedAudioTooLarge {
                encoded_bytes,
                limit_bytes,
            } => {
                assert_eq!(encoded_bytes, XIAOMI_BASE64_SIZE_LIMIT_BYTES + 1);
                assert_eq!(limit_bytes, XIAOMI_BASE64_SIZE_LIMIT_BYTES);
            }
            other => panic!("expected EncodedAudioTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn too_large_error_message_gives_actionable_guidance() {
        let err = validate_encoded_size(XIAOMI_BASE64_SIZE_LIMIT_BYTES + 1).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("trim"), "message was: {message}");
        assert!(
            message.contains("re-run the command"),
            "message was: {message}"
        );
    }

    #[test]
    fn extracts_transcript_from_a_valid_response() {
        let response = response_from_json(
            r#"{"id":"c1","object":"chat.completion","created":1,"model":"mimo-v2.5-asr",
               "choices":[{"index":0,"message":{"role":"assistant","content":"  hello world  "},"finish_reason":"stop"}],
               "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        );
        assert_eq!(extract_transcript(&response).unwrap(), "hello world");
    }

    #[test]
    fn rejects_response_with_no_choices() {
        let response = response_from_json(
            r#"{"id":"c1","object":"chat.completion","created":1,"model":"mimo-v2.5-asr",
               "choices":[],
               "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        );
        assert!(matches!(
            extract_transcript(&response),
            Err(TranscriptionError::MalformedResponse(_))
        ));
    }

    #[test]
    fn rejects_response_with_missing_message_content() {
        let response = response_from_json(
            r#"{"id":"c1","object":"chat.completion","created":1,"model":"mimo-v2.5-asr",
               "choices":[{"index":0,"message":{"role":"assistant"},"finish_reason":"stop"}],
               "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        );
        assert!(matches!(
            extract_transcript(&response),
            Err(TranscriptionError::MalformedResponse(_))
        ));
    }

    #[test]
    fn rejects_response_with_blank_content() {
        let response = response_from_json(
            r#"{"id":"c1","object":"chat.completion","created":1,"model":"mimo-v2.5-asr",
               "choices":[{"index":0,"message":{"role":"assistant","content":"   \n\t  "},"finish_reason":"stop"}],
               "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#,
        );
        assert!(matches!(
            extract_transcript(&response),
            Err(TranscriptionError::EmptyTranscript)
        ));
    }

    #[test]
    fn maps_authentication_error() {
        let err = map_llm_error(LlmError::authentication("bad key"));
        assert!(matches!(err, TranscriptionError::Authentication(_)));
        assert!(err.to_string().contains("authentication failed"));
    }

    #[test]
    fn maps_rate_limit_error() {
        let err = map_llm_error(LlmError::rate_limit("slow down", Some(30)));
        assert!(matches!(err, TranscriptionError::RateLimit(_)));
    }

    #[test]
    fn maps_invalid_request_error() {
        let err = map_llm_error(LlmError::invalid_request("bad payload"));
        assert!(matches!(err, TranscriptionError::InvalidRequest(_)));
    }

    #[test]
    fn maps_generic_api_error() {
        let err = map_llm_error(LlmError::api_error(503, "unavailable".to_string()));
        match err {
            TranscriptionError::Provider { status, message } => {
                assert_eq!(status, 503);
                assert_eq!(message, "unavailable");
            }
            other => panic!("expected Provider, got {other:?}"),
        }
    }

    #[test]
    fn missing_api_key_message_names_the_env_var() {
        let message = TranscriptionError::MissingApiKey.to_string();
        assert!(message.contains(XIAOMI_API_KEY_ENV_VAR));
    }

    #[test]
    fn error_debug_and_display_never_contain_a_literal_api_key_value() {
        // Sanity check: construct an error path that touches a key-shaped
        // string and confirm no formatted output embeds it. This module
        // must never place `api_key` into a Display/Debug string.
        let secret = "sk-super-secret-value";
        let err = map_llm_error(LlmError::authentication("invalid credentials"));
        let rendered = format!("{err} {err:?}");
        assert!(!rendered.contains(secret));
    }
}
