//! `video-to-markdown`: turn a local video into a reviewed Markdown
//! transcript under `video-from-text/`.
//!
//! This binary only parses arguments and reports the outcome. Stage logic
//! lives in `audio`, `transcription`, `cleanup`, and `output`; sequencing
//! lives in `workflow`. See `README.md` for usage.

mod audio;
mod cleanup;
mod cli;
mod output;
mod transcription;
mod workflow;

use clap::Parser;

/// Load `XIAOMI_API_KEY` (and any other variables) from a `.env` file at the
/// repository root, if one is present. Variables already set in the process
/// environment take precedence and are never overwritten. Missing or
/// unreadable `.env` files are silently ignored — this is an optional
/// convenience, not a requirement (the key may already be exported).
fn load_dotenv() {
    let repo_root_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if dotenvy::from_path(&repo_root_env).is_ok() {
        return;
    }
    // Fall back to searching the current directory and its ancestors, in
    // case the binary is run from somewhere other than this workspace
    // checkout (e.g. after `cargo install`).
    let _ = dotenvy::dotenv();
}

#[tokio::main]
async fn main() {
    load_dotenv();
    let cli = cli::Cli::parse();

    match workflow::run(&cli.video_path, cli.overwrite).await {
        Ok(markdown_path) => {
            // Only the final Markdown path goes to stdout, so the command
            // is composable in scripts; all progress and errors go to
            // stderr.
            println!("{}", markdown_path.display());
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}
