use regex::Regex;
use std::fs;
use std::path::Path;

pub struct Match {
    pub slug: String,
    pub snippet: String,
}

pub fn find(posts_dir: &Path, term: &str) -> Result<Vec<Match>, Box<dyn std::error::Error>> {
    // Keep punctuation in names such as AGENTS.md and require word-like
    // boundaries, so "Codex" does not match a longer identifier.
    let pattern = Regex::new(&format!(
        r"(?i)(?:^|[^\p{{L}}\p{{N}}]){}(?:$|[^\p{{L}}\p{{N}}])",
        regex::escape(term)
    ))?;
    let mut paths = fs::read_dir(posts_dir)?
        .map(|entry| entry.map(|item| item.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut found = Vec::new();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let (frontmatter, body) = split_frontmatter(&raw)?;
        let slug = frontmatter_slug(frontmatter)
            .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().into_owned());
        for paragraph in body.split("\n\n") {
            let clean = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
            let Some(found_at) = pattern.find(&clean) else {
                continue;
            };
            // Save one representative excerpt per post, with enough context
            // for the definition prompt but a bounded request size.
            let start = clean[..found_at.start()]
                .char_indices()
                .rev()
                .nth(350)
                .map(|(index, _)| index)
                .unwrap_or(0);
            let end = clean[found_at.end()..]
                .char_indices()
                .nth(550)
                .map(|(index, _)| found_at.end() + index)
                .unwrap_or(clean.len());
            let snippet = clean[start..end].to_string();
            found.push(Match { slug, snippet });
            break;
        }
    }
    Ok(found)
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str), Box<dyn std::error::Error>> {
    let rest = raw
        .strip_prefix("---\n")
        .ok_or("post lacks YAML frontmatter")?;
    rest.split_once("\n---\n")
        .ok_or_else(|| "post has unterminated YAML frontmatter".into())
}

fn frontmatter_slug(frontmatter: &str) -> Option<String> {
    frontmatter
        .lines()
        .find_map(|line| {
            line.strip_prefix("slug:").map(|value| {
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
        })
        .filter(|slug| !slug.is_empty())
}
