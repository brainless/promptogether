//! Stage 4: write the cleaned transcript as Markdown.
//!
//! Derives a safe, predictable `.md` filename from the video filename's
//! stem, creates `video-from-text/` when needed, and writes the cleaned
//! paragraphs as UTF-8 Markdown with minimal front matter. Refuses to
//! overwrite an existing transcript unless the caller explicitly opts in
//! via `--overwrite`, and publishes atomically via a temporary file
//! followed by a rename so a failed run never leaves a partial final
//! document.
//!
//! ## Filename derivation and sanitization
//!
//! The output filename is derived solely from the source video's filename
//! stem (for example, `my-talk.mp4` becomes `video-from-text/my-talk.md`).
//! There is no separate `--slug` option. The video path is a local,
//! maintainer-supplied argument rather than hostile input, but the epic
//! calls for "safe, predictable" naming, so the stem is still defensively
//! sanitized before it is used as a filename:
//!
//! - Only the last path component of the stem is considered (any embedded
//!   path separator — `/` or `\` — is treated as a component boundary, not
//!   copied into the filename), so a crafted or unusual video path cannot
//!   escape `video-from-text/` or traverse into an unrelated directory.
//! - A stem that is empty, or that is exactly `.` or `..`, is replaced with
//!   the fallback name `transcript` rather than producing a hidden,
//!   parent-referencing, or empty filename.
//! - Every character that is not an ASCII letter, digit, `-`, or `_` is
//!   replaced with `-` (this also removes null bytes and other control
//!   characters, which are not valid in filenames on most platforms).
//! - Leading `.` characters are stripped so the result never becomes an
//!   unexpectedly hidden dotfile.
//! - Runs of `-` are collapsed to one, and leading/trailing `-`/`_` are
//!   trimmed.
//! - If sanitization leaves nothing behind, the fallback name `transcript`
//!   is used.
//! - The sanitized stem is truncated to 100 bytes (at a UTF-8 char
//!   boundary) to stay well within filesystem filename limits.
//!
//! ## Atomic publication and overwrite handling
//!
//! `write_markdown` creates `video-from-text/` (or the caller-supplied
//! output directory) if it does not already exist, then:
//!
//! 1. If `overwrite` is `false` and the destination `.md` file already
//!    exists, it returns [`OutputError::AlreadyExists`] immediately without
//!    writing anything.
//! 2. Otherwise, it writes the full Markdown content (front matter plus
//!    cleaned transcript) to a `NamedTempFile` created *in the same output
//!    directory* as the destination, so the final rename is a same-
//!    filesystem, atomic rename rather than a cross-filesystem copy.
//! 3. It then persists (renames) the temporary file onto the destination
//!    path. A process that fails or is interrupted before this rename
//!    leaves only the temporary file (which `NamedTempFile` cleans up on
//!    drop if persistence never happens) — never a partially written
//!    destination file.
//!
//! This is a "don't leave partial files" guarantee for a single-writer,
//! locally-run tool, not a security-hardened defense against concurrent
//! writers: the existence check in step 1 and the rename in step 3 are not
//! part of one atomic transaction, so a file that appears between the
//! check and the rename could still be replaced. That is an accepted,
//! documented trade-off for this maintainer tool.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::NamedTempFile;

/// Fallback filename stem used when the video's filename stem is missing,
/// empty, or sanitizes away to nothing.
const FALLBACK_STEM: &str = "transcript";

/// Maximum length, in bytes, of the sanitized filename stem.
const MAX_STEM_LEN: usize = 100;

/// Default output directory, relative to the current working directory,
/// that `write_markdown` publishes into.
pub const DEFAULT_OUTPUT_DIR: &str = "video-from-text";

/// Errors that can occur while writing the final Markdown output.
#[derive(Debug)]
pub enum OutputError {
    /// The output directory could not be created.
    CreateOutputDir { path: PathBuf, reason: String },
    /// A Markdown transcript already exists at the destination path and
    /// `overwrite` was not set.
    AlreadyExists { path: PathBuf },
    /// A temporary file for atomic publication could not be created.
    TempFile { dir: PathBuf, reason: String },
    /// The Markdown content could not be written to the temporary file.
    Write { path: PathBuf, reason: String },
    /// The temporary file could not be published (renamed) to the
    /// destination path.
    Persist { path: PathBuf, reason: String },
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::CreateOutputDir { path, reason } => write!(
                f,
                "could not create output directory {}: {reason}",
                path.display()
            ),
            OutputError::AlreadyExists { path } => write!(
                f,
                "a transcript already exists at {} — pass --overwrite to replace it",
                path.display()
            ),
            OutputError::TempFile { dir, reason } => write!(
                f,
                "could not create a temporary file in {}: {reason}",
                dir.display()
            ),
            OutputError::Write { path, reason } => {
                write!(
                    f,
                    "could not write Markdown to {}: {reason}",
                    path.display()
                )
            }
            OutputError::Persist { path, reason } => write!(
                f,
                "could not publish the Markdown transcript to {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for OutputError {}

/// Derive a safe, sanitized `.md` filename from a video path's filename
/// stem. See the module documentation for the exact sanitization rules.
fn derive_markdown_filename(video_path: &Path) -> String {
    let raw_stem = video_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    format!("{}.md", sanitize_stem(&raw_stem))
}

/// Sanitize a raw filename stem into a safe fragment usable as a filename.
/// See the module documentation for the exact rules.
fn sanitize_stem(raw_stem: &str) -> String {
    // Only consider the last path-separator-delimited component, in case
    // the stem contains an embedded separator (for example, from a lossy
    // UTF-8 conversion or an unusual platform path).
    let last_component = raw_stem
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .to_string();

    let candidate = if last_component == "." || last_component == ".." {
        String::new()
    } else {
        last_component
    };

    let mut sanitized: String = candidate
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Strip leading dots defensively; the character map above already
    // turns '.' into '-', but this guards the fallback path below too.
    while sanitized.starts_with('.') {
        sanitized.remove(0);
    }

    // Collapse runs of '-'.
    let mut collapsed = String::with_capacity(sanitized.len());
    let mut prev_dash = false;
    for c in sanitized.drain(..) {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }

    let trimmed = collapsed.trim_matches(|c| c == '-' || c == '_');

    let stem = if trimmed.is_empty() {
        FALLBACK_STEM
    } else {
        trimmed
    };

    truncate_at_char_boundary(stem, MAX_STEM_LEN).to_string()
}

/// Truncate `s` to at most `max_len` bytes, cutting at a UTF-8 char
/// boundary rather than splitting a multi-byte character.
fn truncate_at_char_boundary(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Format the current UTC time as an ISO-8601-ish timestamp
/// (`YYYY-MM-DDTHH:MM:SSZ`), without pulling in a date/time dependency.
fn format_utc_timestamp(now: SystemTime) -> String {
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = duration.as_secs() as i64;
    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);

    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Convert a day count since the Unix epoch (1970-01-01) into a
/// (year, month, day) civil date, using Howard Hinnant's well-known
/// `civil_from_days` algorithm (proleptic Gregorian calendar).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Render the full Markdown document: minimal YAML front matter (source
/// filename and a generation timestamp) followed by the cleaned
/// transcript, with a trailing note that the transcript is model-edited
/// and not a verbatim or accessibility-certified record.
fn render_markdown(source_filename: &str, cleaned_transcript: &str, now: SystemTime) -> String {
    let timestamp = format_utc_timestamp(now);
    let escaped_source = source_filename.replace('\\', "\\\\").replace('"', "\\\"");

    format!(
        "---\nsource: \"{escaped_source}\"\ngenerated_at: \"{timestamp}\"\n---\n\n{}\n\n> This transcript was produced with model-assisted transcription and cleanup. It is not a verbatim or accessibility-certified transcript and requires human review before publishing.\n",
        cleaned_transcript.trim()
    )
}

/// Write `cleaned_transcript` as Markdown derived from `video_path` into
/// the `video-from-text/` directory (relative to the current working
/// directory), returning the path to the written file.
///
/// Creates `video-from-text/` if it does not already exist. Refuses to
/// overwrite an existing transcript unless `overwrite` is `true`, and
/// publishes atomically via a temporary file and rename. See the module
/// documentation for the filename sanitization rules and the exact
/// atomicity/overwrite sequencing.
pub fn write_markdown(
    video_path: &Path,
    cleaned_transcript: &str,
    overwrite: bool,
) -> Result<PathBuf, OutputError> {
    write_markdown_into(
        Path::new(DEFAULT_OUTPUT_DIR),
        video_path,
        cleaned_transcript,
        overwrite,
    )
}

/// Same as [`write_markdown`], but publishes into `output_dir` instead of
/// the fixed `video-from-text/` directory. Exists mainly so tests can
/// exercise the real filesystem logic against a temporary directory
/// instead of the repository's real `video-from-text/` directory or the
/// process's current working directory.
pub fn write_markdown_into(
    output_dir: &Path,
    video_path: &Path,
    cleaned_transcript: &str,
    overwrite: bool,
) -> Result<PathBuf, OutputError> {
    std::fs::create_dir_all(output_dir).map_err(|err| OutputError::CreateOutputDir {
        path: output_dir.to_path_buf(),
        reason: err.to_string(),
    })?;

    let filename = derive_markdown_filename(video_path);
    let destination = output_dir.join(&filename);

    if !overwrite && destination.exists() {
        return Err(OutputError::AlreadyExists { path: destination });
    }

    let source_filename = video_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| video_path.to_string_lossy().into_owned());

    let content = render_markdown(&source_filename, cleaned_transcript, SystemTime::now());

    let mut temp_file = NamedTempFile::new_in(output_dir).map_err(|err| OutputError::TempFile {
        dir: output_dir.to_path_buf(),
        reason: err.to_string(),
    })?;

    temp_file
        .write_all(content.as_bytes())
        .and_then(|_| temp_file.flush())
        .map_err(|err| OutputError::Write {
            path: destination.clone(),
            reason: err.to_string(),
        })?;

    temp_file
        .persist(&destination)
        .map_err(|err| OutputError::Persist {
            path: destination.clone(),
            reason: err.error.to_string(),
        })?;

    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_simple_filename_from_stem() {
        let path = Path::new("my-talk.mp4");
        assert_eq!(derive_markdown_filename(path), "my-talk.md");
    }

    #[test]
    fn derives_filename_from_stem_with_spaces_and_symbols() {
        let path = Path::new("My Talk (Take 2)!.mov");
        assert_eq!(derive_markdown_filename(path), "My-Talk-Take-2.md");
    }

    #[test]
    fn sanitizes_path_traversal_attempts_in_stem() {
        // A stem that itself looks like a traversal sequence sanitizes to
        // dashes rather than being interpreted as directory components.
        let path = Path::new("../../etc/passwd.mp4");
        let name = derive_markdown_filename(path);
        assert!(!name.contains(".."));
        assert!(!name.contains('/'));
    }

    #[test]
    fn rejects_dot_and_dotdot_stems_with_fallback() {
        assert_eq!(sanitize_stem("."), FALLBACK_STEM);
        assert_eq!(sanitize_stem(".."), FALLBACK_STEM);
    }

    #[test]
    fn strips_leading_dot_from_dotfile_style_stem() {
        // `Path::file_stem` treats ".mp4" as a dotfile with no extension,
        // so its "stem" is the whole ".mp4" string; the leading dot is
        // stripped so the result is not an unexpectedly hidden file.
        assert_eq!(derive_markdown_filename(Path::new(".mp4")), "mp4.md");
    }

    #[test]
    fn strips_null_bytes_and_control_characters() {
        let path = Path::new("weird\u{0000}name.mp4");
        let name = derive_markdown_filename(path);
        assert!(!name.contains('\u{0000}'));
    }

    #[test]
    fn falls_back_to_transcript_when_stem_sanitizes_to_empty() {
        let path = Path::new("!!!.mp4");
        assert_eq!(derive_markdown_filename(path), "transcript.md");
    }

    #[test]
    fn truncates_very_long_stems() {
        let long_stem = "a".repeat(500);
        let path = PathBuf::from(format!("{long_stem}.mp4"));
        let name = derive_markdown_filename(&path);
        // ".md" is 3 bytes.
        assert!(name.len() <= MAX_STEM_LEN + 3);
    }

    #[test]
    fn no_existing_file_succeeds() {
        let dir = tempfile::tempdir().expect("temp dir");
        let video_path = Path::new("intro.mp4");
        let result = write_markdown_into(dir.path(), video_path, "Hello world.", false)
            .expect("write markdown");
        assert_eq!(result, dir.path().join("intro.md"));
        assert!(result.exists());
    }

    #[test]
    fn refuses_to_overwrite_without_flag() {
        let dir = tempfile::tempdir().expect("temp dir");
        let video_path = Path::new("intro.mp4");
        write_markdown_into(dir.path(), video_path, "First version.", false)
            .expect("first write succeeds");

        let err = write_markdown_into(dir.path(), video_path, "Second version.", false)
            .expect_err("second write without overwrite should fail");
        assert!(matches!(err, OutputError::AlreadyExists { .. }));

        let content = std::fs::read_to_string(dir.path().join("intro.md")).expect("read file");
        assert!(content.contains("First version."));
    }

    #[test]
    fn overwrite_flag_replaces_existing_content() {
        let dir = tempfile::tempdir().expect("temp dir");
        let video_path = Path::new("intro.mp4");
        write_markdown_into(dir.path(), video_path, "First version.", false)
            .expect("first write succeeds");

        let result = write_markdown_into(dir.path(), video_path, "Second version.", true)
            .expect("overwrite should succeed");

        let content = std::fs::read_to_string(&result).expect("read file");
        assert!(content.contains("Second version."));
        assert!(!content.contains("First version."));
    }

    #[test]
    fn creates_output_directory_when_missing() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("video-from-text");
        assert!(!nested.exists());

        write_markdown_into(&nested, Path::new("talk.mp4"), "Body text.", false)
            .expect("write markdown");
        assert!(nested.is_dir());
    }

    #[test]
    fn does_not_leave_stray_temp_files_after_a_successful_write() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_markdown_into(dir.path(), Path::new("talk.mp4"), "Body text.", false)
            .expect("write markdown");

        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .map(|entry| entry.expect("dir entry").file_name())
            .collect();
        assert_eq!(entries.len(), 1, "expected exactly one file: {entries:?}");
        assert_eq!(entries[0].to_string_lossy(), "talk.md");
    }

    #[test]
    fn front_matter_includes_source_filename_and_non_verbatim_note() {
        let dir = tempfile::tempdir().expect("temp dir");
        let result = write_markdown_into(
            dir.path(),
            Path::new("session-one.mp4"),
            "Paragraph one.\n\nParagraph two.",
            false,
        )
        .expect("write markdown");

        let content = std::fs::read_to_string(&result).expect("read file");
        assert!(content.starts_with("---\n"));
        assert!(content.contains("source: \"session-one.mp4\""));
        assert!(content.contains("generated_at: \""));
        assert!(content.contains("Paragraph one."));
        assert!(content.contains("Paragraph two."));
        assert!(content.to_lowercase().contains("not a verbatim"));
        assert!(content.to_lowercase().contains("review"));
    }

    #[test]
    fn civil_from_days_matches_known_epoch_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 is a well-known reference date for this algorithm.
        let days_to_2000_03_01 = {
            // Compute via the same function's inverse expectation: just
            // check a date far enough out to exercise leap-year handling.
            11_017
        };
        assert_eq!(civil_from_days(days_to_2000_03_01), (2000, 3, 1));
    }

    #[test]
    fn format_utc_timestamp_produces_expected_shape() {
        let now = UNIX_EPOCH + std::time::Duration::from_secs(0);
        assert_eq!(format_utc_timestamp(now), "1970-01-01T00:00:00Z");
    }
}
