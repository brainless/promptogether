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

#[tokio::main]
async fn main() {
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
