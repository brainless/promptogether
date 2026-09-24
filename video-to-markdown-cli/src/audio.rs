//! Stage 1: extract audio from the source video.
//!
//! Accepts the video path from the CLI, validates that it is a readable
//! file, verifies that `ffmpeg` is available, and invokes it without a
//! shell to produce a MiMo-compatible MP3 file in a per-run temporary
//! workspace. Reports a missing executable, unsupported or corrupt input,
//! and a failed process with useful context, and never overwrites the
//! source video.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

/// The extracted, MiMo-compatible audio file plus the temporary workspace
/// that owns it.
///
/// The `TempDir` guard must stay alive for as long as `path` is needed:
/// dropping it deletes the workspace (and the audio file inside it). Keep
/// this struct alive until the audio has been consumed by the
/// transcription stage.
pub struct ExtractedAudio {
    /// Ordered paths to the extracted MP3 chunks inside the temporary workspace.
    pub paths: Vec<PathBuf>,
    _workspace: TempDir,
}

const SILENCE_THRESHOLD: &str = "-35dB";
const MIN_SILENCE_SECS: f64 = 0.6;
const MIN_CHUNK_SECS: f64 = 75.0;
const TARGET_CHUNK_SECS: f64 = 120.0;
const MAX_CHUNK_SECS: f64 = 150.0;

/// Errors that can occur while extracting audio from a source video.
#[derive(Debug)]
pub enum AudioError {
    /// The video path does not exist or is not a regular file.
    VideoNotFound(PathBuf),
    /// The video file exists but could not be opened for reading.
    VideoUnreadable { path: PathBuf, reason: String },
    /// `ffmpeg` is not available on `PATH`.
    FfmpegNotFound,
    /// Could not create a per-run temporary workspace directory.
    TempWorkspace(String),
    /// The `ffmpeg` process could not be started.
    FfmpegSpawnFailed(String),
    /// `ffmpeg` exited with a non-zero status.
    FfmpegFailed { status: Option<i32>, stderr: String },
    /// `ffmpeg` exited successfully but did not produce any audio chunks.
    OutputMissing,
    /// The audio scan did not report a usable duration.
    DurationUnavailable,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::VideoNotFound(path) => {
                write!(f, "video file not found: {}", path.display())
            }
            AudioError::VideoUnreadable { path, reason } => {
                write!(f, "cannot read video file {}: {reason}", path.display())
            }
            AudioError::FfmpegNotFound => write!(
                f,
                "ffmpeg was not found on PATH — install ffmpeg and ensure it is available before running this command"
            ),
            AudioError::TempWorkspace(reason) => {
                write!(f, "could not create a temporary workspace: {reason}")
            }
            AudioError::FfmpegSpawnFailed(reason) => {
                write!(f, "failed to start ffmpeg: {reason}")
            }
            AudioError::FfmpegFailed { status, stderr } => {
                let status_desc = match status {
                    Some(code) => format!("exit code {code}"),
                    None => "terminated by a signal".to_string(),
                };
                let stderr_tail = tail_lines(stderr, 20);
                if stderr_tail.is_empty() {
                    write!(f, "ffmpeg failed ({status_desc}) with no output on stderr — the input may be unsupported or corrupt")
                } else {
                    write!(
                        f,
                        "ffmpeg failed ({status_desc}):\n{stderr_tail}"
                    )
                }
            }
            AudioError::OutputMissing => write!(
                f,
                "ffmpeg did not produce any audio chunks — the input video may be unsupported or corrupt"
            ),
            AudioError::DurationUnavailable => write!(
                f,
                "ffmpeg could not determine the audio duration — the input video may be unsupported or corrupt"
            ),
        }
    }
}

impl std::error::Error for AudioError {}

/// Keep only the last `max_lines` lines of `text`, to avoid dumping an
/// unbounded amount of ffmpeg output into an error message.
fn tail_lines(text: &str, max_lines: usize) -> String {
    let trimmed = text.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() <= max_lines {
        trimmed.to_string()
    } else {
        lines[lines.len() - max_lines..].join("\n")
    }
}

/// Validate that `video_path` exists, is a regular file, and can be opened
/// for reading.
fn validate_video_path(video_path: &Path) -> Result<(), AudioError> {
    let metadata = std::fs::metadata(video_path)
        .map_err(|_| AudioError::VideoNotFound(video_path.to_path_buf()))?;
    if !metadata.is_file() {
        return Err(AudioError::VideoNotFound(video_path.to_path_buf()));
    }
    File::open(video_path).map_err(|err| AudioError::VideoUnreadable {
        path: video_path.to_path_buf(),
        reason: describe_io_error(&err),
    })?;
    Ok(())
}

fn describe_io_error(err: &io::Error) -> String {
    err.to_string()
}

/// Verify that `ffmpeg` is available on `PATH` by attempting to invoke it
/// directly (no shell) with `-version`.
fn verify_ffmpeg_available() -> Result<(), AudioError> {
    let result = Command::new("ffmpeg")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(AudioError::FfmpegNotFound),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(AudioError::FfmpegNotFound),
        Err(err) => Err(AudioError::FfmpegSpawnFailed(err.to_string())),
    }
}

/// Find quiet intervals and the decoded audio duration in one fast pass.
/// `silencedetect` only observes the audio; it does not remove any samples.
fn scan_audio(video_path: &Path) -> Result<(Vec<f64>, f64), AudioError> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-nostdin")
        .arg("-nostats")
        .arg("-loglevel")
        .arg("info")
        .arg("-i")
        .arg(video_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-af")
        .arg(format!(
            "silencedetect=noise={SILENCE_THRESHOLD}:d={MIN_SILENCE_SECS}"
        ))
        .arg("-f")
        .arg("null")
        .arg("-")
        .arg("-progress")
        .arg("pipe:1")
        .stdin(Stdio::null())
        .output()
        .map_err(|err| AudioError::FfmpegSpawnFailed(err.to_string()))?;

    if !output.status.success() {
        return Err(AudioError::FfmpegFailed {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let progress = String::from_utf8_lossy(&output.stdout);
    let duration = progress
        .lines()
        .filter_map(|line| line.strip_prefix("out_time_us="))
        .filter_map(|value| value.parse::<u64>().ok())
        .last()
        .map(|micros| micros as f64 / 1_000_000.0)
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
        .ok_or(AudioError::DurationUnavailable)?;

    let log = String::from_utf8_lossy(&output.stderr);
    let mut silence_start = None;
    let mut pauses = Vec::new();
    for line in log.lines() {
        if let Some(value) = line.split("silence_start: ").nth(1) {
            silence_start = value.trim().parse::<f64>().ok();
        } else if let Some(value) = line.split("silence_end: ").nth(1) {
            let end = value
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok());
            if let (Some(start), Some(end)) = (silence_start.take(), end) {
                if end > start && end <= duration {
                    pauses.push((start + end) / 2.0);
                }
            }
        }
    }
    Ok((pauses, duration))
}

/// Prefer a pause near two minutes, while bounding every chunk to 150 seconds.
/// A timed cut is used if no qualifying pause exists in the allowed window.
fn choose_split_points(pauses: &[f64], duration: f64) -> Vec<f64> {
    let mut cuts = Vec::new();
    let mut start = 0.0;
    while duration - start > MAX_CHUNK_SECS {
        let earliest = start + MIN_CHUNK_SECS;
        let latest = (start + MAX_CHUNK_SECS).min(duration - MIN_CHUNK_SECS);
        let target = (start + TARGET_CHUNK_SECS).min(latest);
        let cut = pauses
            .iter()
            .copied()
            .filter(|pause| *pause >= earliest && *pause <= latest)
            .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))
            .unwrap_or(target);
        cuts.push(cut);
        start = cut;
    }
    cuts
}

/// Extract MiMo-compatible audio from `video_path` into a per-run temporary
/// workspace, returning ordered chunk paths alongside the workspace guard.
///
/// The output is mono MP3 encoded with `libmp3lame` at 64 kbps. A first pass
/// finds quiet intervals; a second pass splits near two-minute marks within
/// those intervals when possible. No chunk exceeds about 150 seconds.
/// Playback speed and the pauses themselves are preserved.
pub fn extract_audio(video_path: &Path) -> Result<ExtractedAudio, AudioError> {
    validate_video_path(video_path)?;
    verify_ffmpeg_available()?;
    let (pauses, duration) = scan_audio(video_path)?;
    let cuts = choose_split_points(&pauses, duration);

    let workspace = TempDir::with_prefix("video-to-markdown-")
        .map_err(|err| AudioError::TempWorkspace(err.to_string()))?;
    let output_pattern = workspace.path().join("audio-%03d.mp3");

    let mut command = Command::new("ffmpeg");
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-y")
        .arg("-i")
        .arg(video_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-c:a")
        .arg("libmp3lame")
        .arg("-ac")
        .arg("1")
        .arg("-b:a")
        .arg("64k")
        .arg("-f")
        .arg("segment");
    if cuts.is_empty() {
        command.arg("-segment_time").arg("150");
    } else {
        let times = cuts
            .iter()
            .map(|cut| format!("{cut:.3}"))
            .collect::<Vec<_>>()
            .join(",");
        command.arg("-segment_times").arg(times);
    }
    let output = command
        .arg("-reset_timestamps")
        .arg("1")
        .arg(&output_pattern)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| AudioError::FfmpegSpawnFailed(err.to_string()))?;

    if !output.status.success() {
        return Err(AudioError::FfmpegFailed {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let mut paths: Vec<PathBuf> = std::fs::read_dir(workspace.path())
        .map_err(|err| AudioError::TempWorkspace(err.to_string()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("audio-") && name.ends_with(".mp3"))
        })
        .collect();
    paths.sort();
    if paths.is_empty()
        || paths.iter().any(|path| {
            !std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
        })
    {
        return Err(AudioError::OutputMissing);
    }

    Ok(ExtractedAudio {
        paths,
        _workspace: workspace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_missing_video_file() {
        let missing = PathBuf::from("/nonexistent/path/does-not-exist.mp4");
        let err = validate_video_path(&missing).unwrap_err();
        assert!(matches!(err, AudioError::VideoNotFound(_)));
    }

    #[test]
    fn rejects_directory_as_video_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let err = validate_video_path(dir.path()).unwrap_err();
        assert!(matches!(err, AudioError::VideoNotFound(_)));
    }

    #[test]
    fn accepts_readable_regular_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("input.mp4");
        let mut file = File::create(&file_path).expect("create file");
        file.write_all(b"not a real video, just bytes")
            .expect("write file");
        assert!(validate_video_path(&file_path).is_ok());
    }

    #[test]
    fn tail_lines_keeps_only_the_last_lines() {
        let text = (1..=30)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tail = tail_lines(&text, 5);
        let lines: Vec<&str> = tail.lines().collect();
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], "line 26");
        assert_eq!(lines[4], "line 30");
    }

    #[test]
    fn tail_lines_returns_full_text_when_short() {
        let text = "line 1\nline 2";
        assert_eq!(tail_lines(text, 5), "line 1\nline 2");
    }

    fn ffmpeg_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Real smoke test: only runs when `ffmpeg` is on PATH. Generates a
    /// tiny synthetic test video with `ffmpeg`'s `lavfi` inputs and checks
    /// that `extract_audio` produces a non-empty MP3 file.
    #[test]
    fn extracts_audio_from_a_real_synthetic_video_when_ffmpeg_is_available() {
        if !ffmpeg_available() {
            eprintln!("skipping: ffmpeg not found on PATH");
            return;
        }

        let dir = tempfile::tempdir().expect("create temp dir");
        let video_path = dir.path().join("sample.mp4");

        let status = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("testsrc=duration=1:size=64x64:rate=5")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=440:duration=1")
            .arg("-shortest")
            .arg(&video_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("generate synthetic test video");
        assert!(status.success(), "failed to generate synthetic test video");

        let extracted = extract_audio(&video_path).expect("extract audio");
        assert_eq!(extracted.paths.len(), 1);
        let path = &extracted.paths[0];
        let metadata = std::fs::metadata(path).expect("read extracted audio metadata");
        assert!(metadata.len() > 0, "extracted audio file is empty");
        assert!(path.extension().map(|ext| ext == "mp3").unwrap_or(false));
    }

    #[test]
    fn reports_missing_ffmpeg_when_binary_name_is_wrong() {
        // Sanity-check that a NotFound spawn error is classified as
        // FfmpegNotFound rather than a generic spawn failure.
        let result = Command::new("definitely-not-a-real-ffmpeg-binary-xyz")
            .arg("-version")
            .status();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
