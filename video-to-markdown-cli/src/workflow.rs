//! Sequences the video-to-Markdown stages and owns temporary-file cleanup.
//!
//! `workflow::run` is the single entry point `main.rs` calls. It runs
//! `transcription::api_key_from_env`, then `audio::extract_audio`, then
//! `transcription::transcribe`, then `cleanup::clean_transcript`, then
//! `output::write_markdown`, in order, propagating each stage's typed error
//! and printing concise stage progress to stderr along the way.
//!
//! The extracted audio's `ExtractedAudio` value (not just its `.path`) is
//! held for as long as the audio file is needed — through the transcription
//! call — so its `TempDir` guard cleans up the temporary workspace on both
//! success and failure once it goes out of scope. Nothing about that cleanup
//! is deferred or manual: it is ordinary Rust drop order.

use std::path::{Path, PathBuf};

use crate::audio::{self, AudioError};
use crate::cleanup::{self, CleanupError};
use crate::output::{self, OutputError};
use crate::transcription::{self, TranscriptionError};

/// Errors that can occur while running the end-to-end workflow, tagged by
/// the stage that failed.
#[derive(Debug)]
pub enum WorkflowError {
    /// Audio extraction failed.
    Audio(AudioError),
    /// Speech transcription failed.
    Transcription(TranscriptionError),
    /// Transcript cleanup failed.
    Cleanup(CleanupError),
    /// Markdown output failed.
    Output(OutputError),
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowError::Audio(err) => write!(f, "audio stage failed: {err}"),
            WorkflowError::Transcription(err) => {
                write!(f, "transcription stage failed: {err}")
            }
            WorkflowError::Cleanup(err) => write!(f, "cleanup stage failed: {err}"),
            WorkflowError::Output(err) => write!(f, "output stage failed: {err}"),
        }
    }
}

impl std::error::Error for WorkflowError {}

impl From<AudioError> for WorkflowError {
    fn from(err: AudioError) -> Self {
        WorkflowError::Audio(err)
    }
}

impl From<TranscriptionError> for WorkflowError {
    fn from(err: TranscriptionError) -> Self {
        WorkflowError::Transcription(err)
    }
}

impl From<CleanupError> for WorkflowError {
    fn from(err: CleanupError) -> Self {
        WorkflowError::Cleanup(err)
    }
}

impl From<OutputError> for WorkflowError {
    fn from(err: OutputError) -> Self {
        WorkflowError::Output(err)
    }
}

/// Run the full video-to-Markdown workflow for `video_path`, returning the
/// path to the written Markdown file.
///
/// Stage progress is printed to stderr, one concise line per stage, naming
/// only the stage — never API keys, Base64 audio, or transcript contents.
/// The caller is responsible for printing the returned path to stdout.
pub async fn run(video_path: &Path, overwrite: bool) -> Result<PathBuf, WorkflowError> {
    // Read the credential first: it is a cheap, local check, and failing
    // fast on a missing API key avoids spending time extracting audio (and
    // invoking ffmpeg) only to fail at the transcription stage anyway. The
    // same key is reused for both the transcription and cleanup stages.
    eprintln!("Checking for XIAOMI_API_KEY...");
    let api_key = transcription::api_key_from_env()?;

    eprintln!("Extracting audio from video...");
    let extracted_audio = audio::extract_audio(video_path)?;
    eprintln!("Audio extraction complete.");

    eprintln!("Transcribing audio via MiMo V2.5 ASR...");
    let raw_transcript = transcription::transcribe(&extracted_audio.path, &api_key).await?;
    eprintln!("Transcription complete.");

    // The extracted audio is no longer needed once transcription has read
    // it; `extracted_audio` (and its `TempDir` guard) is dropped naturally
    // at the end of this function, cleaning up the temporary workspace.

    eprintln!("Cleaning up transcript...");
    let cleaned_transcript = cleanup::clean_transcript(&raw_transcript, &api_key).await?;
    eprintln!("Transcript cleanup complete.");

    eprintln!("Writing Markdown...");
    let markdown_path = output::write_markdown(video_path, &cleaned_transcript, overwrite)?;
    eprintln!("Markdown written.");

    Ok(markdown_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_error_display_is_clear_and_tags_the_stage() {
        let err: WorkflowError = AudioError::FfmpegNotFound.into();
        let message = err.to_string();

        assert!(message.starts_with("audio stage failed:"));
        assert!(message.contains("ffmpeg"));
    }

    #[test]
    fn transcription_error_display_is_clear_and_tags_the_stage() {
        let err: WorkflowError = TranscriptionError::MissingApiKey.into();
        let message = err.to_string();

        assert!(message.starts_with("transcription stage failed:"));
        assert!(message.contains("XIAOMI_API_KEY"));
    }

    #[test]
    fn cleanup_error_display_is_clear_and_tags_the_stage() {
        let err: WorkflowError = CleanupError::EmptyResult.into();
        let message = err.to_string();

        assert!(message.starts_with("cleanup stage failed:"));
    }

    #[test]
    fn output_error_display_is_clear_and_tags_the_stage() {
        let err: WorkflowError = OutputError::AlreadyExists {
            path: PathBuf::from("video-from-text/example.md"),
        }
        .into();
        let message = err.to_string();

        assert!(message.starts_with("output stage failed:"));
        assert!(message.contains("video-from-text/example.md"));
    }

    #[test]
    fn workflow_error_display_never_contains_a_literal_api_key_value() {
        // Sanity check mirroring the non-leakage tests in transcription.rs
        // and cleanup.rs: construct a stage error via the same
        // `TranscriptionError::MissingApiKey` path `workflow::run` actually
        // hits when the key is absent, and confirm the wrapping
        // `WorkflowError`'s Display never embeds a key-shaped secret value
        // (which it cannot, since `MissingApiKey` never carries one).
        let secret = "sk-super-secret-value";
        let err: WorkflowError = TranscriptionError::MissingApiKey.into();

        assert!(!err.to_string().contains(secret));
    }

    #[test]
    fn from_audio_error_preserves_the_underlying_message() {
        let stage_err = AudioError::VideoNotFound(PathBuf::from("missing.mp4"));
        let stage_message = stage_err.to_string();

        let workflow_err: WorkflowError = stage_err.into();

        assert!(workflow_err.to_string().contains(&stage_message));
    }

    #[test]
    fn from_transcription_error_preserves_the_underlying_message() {
        let stage_err = TranscriptionError::RateLimit("too many requests".to_string());
        let stage_message = stage_err.to_string();

        let workflow_err: WorkflowError = stage_err.into();

        assert!(workflow_err.to_string().contains(&stage_message));
    }

    #[test]
    fn from_cleanup_error_preserves_the_underlying_message() {
        let stage_err = CleanupError::MalformedResponse("missing choices array".to_string());
        let stage_message = stage_err.to_string();

        let workflow_err: WorkflowError = stage_err.into();

        assert!(workflow_err.to_string().contains(&stage_message));
    }

    #[test]
    fn from_output_error_preserves_the_underlying_message() {
        let stage_err = OutputError::Persist {
            path: PathBuf::from("video-from-text/example.md"),
            reason: "cross-device link".to_string(),
        };
        let stage_message = stage_err.to_string();

        let workflow_err: WorkflowError = stage_err.into();

        assert!(workflow_err.to_string().contains(&stage_message));
    }

    #[tokio::test]
    async fn run_fails_fast_on_a_nonexistent_video_path_without_an_api_key() {
        // `run` checks for the API key before touching the video path or
        // invoking ffmpeg, so this exercises the real early-exit path
        // end-to-end without needing ffmpeg, network access, or a live API
        // key. Guard against interference from a developer's real
        // environment by only asserting the failure when the variable is
        // genuinely absent from this process.
        if std::env::var(transcription::XIAOMI_API_KEY_ENV_VAR).is_ok() {
            eprintln!(
                "skipping: {} is set in this environment",
                transcription::XIAOMI_API_KEY_ENV_VAR
            );
            return;
        }

        let result = run(Path::new("/nonexistent/video-does-not-exist.mp4"), false).await;

        match result {
            Err(WorkflowError::Transcription(TranscriptionError::MissingApiKey)) => {}
            other => panic!("expected a missing-API-key transcription error, got {other:?}"),
        }
    }
}
