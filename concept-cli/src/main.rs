mod definition;
mod posts;
mod storage;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "concept-cli",
    about = "Find a concept in posts and draft its definition with MiMo V2.6 Flash"
)]
struct Cli {
    /// Concept name to search for, such as "Claude Code" or "AGENTS.md".
    concept: String,
    /// Replace an existing definition and its post links.
    #[arg(long)]
    overwrite: bool,
    /// Xiaomi chat model ID.
    #[arg(long, default_value = "mimo-v2.6-flash")]
    model: String,
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

#[tokio::main]
async fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let _ = dotenvy::from_path(repo_root.join(".env"));
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    if let Err(err) = run(cli, repo_root).await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli, repo_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = repo_root.join("webapp/content/concepts.sqlite");
    let name = cli.concept.trim();
    if name.is_empty() {
        return Err("concept name cannot be empty".into());
    }
    let slug = slugify(name);
    if slug.is_empty() {
        return Err("concept name needs an ASCII letter or digit for its URL slug".into());
    }
    let posts = posts::find(&repo_root.join("webapp/content/posts"), name)?;
    if posts.is_empty() {
        return Err(format!("no posts mention {name:?}; nothing was saved").into());
    }

    let mut db = storage::open(&db_path).await?;
    let existing = storage::existing_slug(&mut db, &slug, name).await?;
    if existing.is_some() && !cli.overwrite {
        return Err(format!(
            "concept already exists; use --overwrite to replace its definition and post links"
        )
        .into());
    }
    let slug = existing.unwrap_or(slug);
    let api_key = std::env::var("XIAOMI_API_KEY")
        .map_err(|_| "XIAOMI_API_KEY is not set; set it in the shell or repository .env")?;
    if api_key.trim().is_empty() {
        return Err("XIAOMI_API_KEY is empty".into());
    }
    eprintln!(
        "Found {name:?} in {} post(s). Generating definition with {}...",
        posts.len(),
        cli.model
    );
    let definition = definition::generate(name, &posts, &api_key, &cli.model).await?;
    let post_slugs = posts
        .iter()
        .map(|item| item.slug.clone())
        .collect::<Vec<_>>();
    storage::save(
        &mut db,
        &slug,
        name,
        &definition,
        &post_slugs,
        cli.overwrite,
    )
    .await?;
    println!("{name}: {definition}");
    println!("Posts: {}", post_slugs.join(", "));
    println!(
        "Saved to {}. Rebuild the webapp to update the static site.",
        db_path.display()
    );
    Ok(())
}
