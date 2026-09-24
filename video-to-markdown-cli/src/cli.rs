//! Command-line argument parsing for the `video-to-markdown` binary.
//!
//! Owns user-facing argument definitions and `--help` text only. Stage logic
//! lives in the `audio`, `transcription`, `cleanup`, and `output` modules;
//! sequencing lives in `workflow`.

use std::path::PathBuf;

use clap::Parser;

/// Transcribe a local video into a reviewed Markdown transcript under
/// `video-from-text/`.
#[derive(Debug, Parser)]
#[command(name = "video-to-markdown", version, about)]
pub struct Cli {
    /// Path to the local video file to transcribe.
    pub video_path: PathBuf,

    /// Overwrite an existing Markdown transcript at the destination path.
    ///
    /// Without this flag, a run that would collide with an existing
    /// transcript under `video-from-text/` is refused rather than
    /// silently replacing reviewed content.
    #[arg(long)]
    pub overwrite: bool,

    /// Use this whisper.cpp model locally if Xiaomi filters an ASR chunk.
    /// Requires `whisper-cli` on PATH; other chunks still use Xiaomi.
    #[arg(long, value_name = "MODEL_PATH")]
    pub local_asr_model: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn parses_a_valid_invocation_with_overwrite_defaulting_to_false() {
        let cli = Cli::try_parse_from(["video-to-markdown", "video.mp4"])
            .expect("a video path alone should parse");

        assert_eq!(cli.video_path, PathBuf::from("video.mp4"));
        assert!(!cli.overwrite);
        assert!(cli.local_asr_model.is_none());
    }

    #[test]
    fn overwrite_flag_sets_overwrite_to_true() {
        let cli = Cli::try_parse_from(["video-to-markdown", "video.mp4", "--overwrite"])
            .expect("--overwrite should parse alongside a video path");

        assert!(cli.overwrite);
    }

    #[test]
    fn missing_video_path_fails_to_parse() {
        let result = Cli::try_parse_from(["video-to-markdown"]);

        let err = result.expect_err("a missing required video path should fail to parse");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn help_flag_does_not_panic_and_reports_display_help() {
        let result = Cli::try_parse_from(["video-to-markdown", "--help"]);

        let err = result.expect_err("--help exits parsing via an error-shaped result");
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    }
}
