//! Optional local transcription for audio chunks Xiaomi filters.
//!
//! whisper.cpp's `whisper-cli` accepts 16-bit WAV input. Convert only the
//! affected MP3 chunk in a temporary workspace, then read its plain-text
//! output. Neither process receives the Xiaomi key or logs transcript text.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;

#[derive(Debug)]
pub enum LocalAsrError {
    ModelUnavailable(String),
    WhisperUnavailable(String),
    TempWorkspace(String),
    ConversionFailed(String),
    WhisperFailed(String),
    TranscriptUnreadable(String),
    EmptyTranscript,
}

impl std::fmt::Display for LocalAsrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelUnavailable(reason) => write!(f, "local ASR model unavailable: {reason}"),
            Self::WhisperUnavailable(reason) => write!(f, "whisper-cli unavailable: {reason}"),
            Self::TempWorkspace(reason) => {
                write!(f, "local ASR temporary workspace failed: {reason}")
            }
            Self::ConversionFailed(reason) => {
                write!(f, "local ASR WAV conversion failed: {reason}")
            }
            Self::WhisperFailed(reason) => write!(f, "whisper-cli transcription failed: {reason}"),
            Self::TranscriptUnreadable(reason) => {
                write!(f, "cannot read whisper-cli transcript: {reason}")
            }
            Self::EmptyTranscript => write!(f, "whisper-cli returned an empty transcript"),
        }
    }
}

impl std::error::Error for LocalAsrError {}

/// Check the opt-in model and executable before starting remote ASR calls.
pub fn validate_setup(model_path: &Path) -> Result<(), LocalAsrError> {
    let metadata = fs::metadata(model_path).map_err(|err| {
        LocalAsrError::ModelUnavailable(format!("{}: {err}", model_path.display()))
    })?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(LocalAsrError::ModelUnavailable(format!(
            "{} is not a nonempty model file",
            model_path.display()
        )));
    }
    Command::new("whisper-cli")
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| LocalAsrError::WhisperUnavailable(err.to_string()))?;
    // Some whisper.cpp releases exit nonzero after printing help. A process
    // that started is enough to confirm the executable is available.
    Ok(())
}

/// Transcribe an MP3 chunk with whisper.cpp entirely on the local machine.
pub fn transcribe(mp3_path: &Path, model_path: &Path) -> Result<String, LocalAsrError> {
    let workspace = TempDir::with_prefix("video-to-markdown-local-asr-")
        .map_err(|err| LocalAsrError::TempWorkspace(err.to_string()))?;
    let wav_path = workspace.path().join("chunk.wav");
    let output_prefix = workspace.path().join("transcript");

    let conversion = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(mp3_path)
        .arg("-vn")
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(&wav_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| LocalAsrError::ConversionFailed(err.to_string()))?;
    if !conversion.success() {
        return Err(LocalAsrError::ConversionFailed(format!(
            "ffmpeg exited with {conversion}"
        )));
    }

    let whisper = Command::new("whisper-cli")
        .arg("-m")
        .arg(model_path)
        .arg("-f")
        .arg(&wav_path)
        .arg("-l")
        .arg("en")
        .arg("-nt")
        .arg("-otxt")
        .arg("-of")
        .arg(&output_prefix)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| LocalAsrError::WhisperFailed(err.to_string()))?;
    if !whisper.success() {
        return Err(LocalAsrError::WhisperFailed(format!(
            "process exited with {whisper}"
        )));
    }

    let transcript_path = output_prefix.with_extension("txt");
    let transcript = fs::read_to_string(transcript_path)
        .map_err(|err| LocalAsrError::TranscriptUnreadable(err.to_string()))?;
    let transcript = transcript.trim();
    if transcript.is_empty() {
        return Err(LocalAsrError::EmptyTranscript);
    }
    Ok(transcript.to_string())
}
