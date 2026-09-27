mod definition;
mod posts;
mod storage;

use clap::Parser;
use std::collections::HashSet;
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
    /// Additional spelling to search and save; repeat for multiple aliases.
    #[arg(long)]
    alias: Vec<String>,
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

fn plural_form(name: &str) -> Option<String> {
    if !name.bytes().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with("ch")
        || lower.ends_with("sh")
        || lower.ends_with("ss")
        || lower.ends_with('x')
        || lower.ends_with('z')
    {
        Some(format!("{name}es"))
    } else if lower.ends_with('s') {
        None
    } else if lower.len() > 1
        && lower.ends_with('y')
        && !matches!(
            lower.as_bytes().get(lower.len() - 2),
            Some(b'a' | b'e' | b'i' | b'o' | b'u')
        )
    {
        Some(format!("{}ies", &name[..name.len() - 1]))
    } else {
        Some(format!("{name}s"))
    }
}

fn add_term(terms: &mut Vec<String>, seen: &mut HashSet<String>, term: String) {
    if seen.insert(term.to_lowercase()) {
        terms.push(term);
    }
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
    let mut db = storage::open(&db_path).await?;
    let existing = storage::existing_slug(&mut db, &slug, name).await?;
    if existing.is_none() {
        if let Some(owner) = storage::term_owner(&mut db, name).await? {
            return Err(format!("{name:?} is already an alias of concept {owner:?}").into());
        }
    }
    if existing.is_some() && !cli.overwrite {
        return Err(format!(
            "concept already exists; use --overwrite to replace its definition and post links"
        )
        .into());
    }
    let slug = existing.unwrap_or(slug);
    let saved_aliases = storage::aliases(&mut db, &slug).await?;
    let mut terms = Vec::new();
    let mut seen = HashSet::new();
    add_term(&mut terms, &mut seen, name.to_string());
    for alias in &saved_aliases {
        add_term(&mut terms, &mut seen, alias.clone());
    }
    let mut new_aliases = Vec::new();
    for alias in &cli.alias {
        let alias = alias.trim();
        if alias.is_empty() || alias.contains('\n') || alias.contains('\r') {
            return Err("aliases must be nonempty single-line terms".into());
        }
        if seen.contains(&alias.to_lowercase()) {
            continue;
        }
        if let Some(owner) = storage::term_owner(&mut db, alias).await? {
            if owner != slug {
                return Err(format!("alias {alias:?} already belongs to concept {owner:?}").into());
            }
        }
        add_term(&mut terms, &mut seen, alias.to_string());
        new_aliases.push(alias.to_string());
    }
    let generated = plural_form(name);
    if let Some(plural) = &generated {
        if !seen.contains(&plural.to_lowercase())
            && storage::term_owner(&mut db, plural).await?.is_none()
        {
            add_term(&mut terms, &mut seen, plural.clone());
        }
    }
    let search = posts::find(&repo_root.join("webapp/content/posts"), &terms)?;
    if search.posts.is_empty() {
        return Err(format!("no posts mention {name:?} or its aliases; nothing was saved").into());
    }
    if let Some(plural) = generated {
        if search.matched_terms.contains(&plural.to_lowercase())
            && !new_aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&plural))
            && !saved_aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&plural))
        {
            new_aliases.push(plural);
        }
    }
    let api_key = std::env::var("XIAOMI_API_KEY")
        .map_err(|_| "XIAOMI_API_KEY is not set; set it in the shell or repository .env")?;
    if api_key.trim().is_empty() {
        return Err("XIAOMI_API_KEY is empty".into());
    }
    eprintln!(
        "Found {name:?} in {} post(s). Generating definition with {}...",
        search.posts.len(),
        cli.model
    );
    let definition = definition::generate(name, &search.posts, &api_key, &cli.model).await?;
    let post_slugs = search
        .posts
        .iter()
        .map(|item| item.slug.clone())
        .collect::<Vec<_>>();
    storage::save(
        &mut db,
        &slug,
        name,
        &definition,
        &post_slugs,
        &new_aliases,
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
