use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct Match {
    pub slug: String,
    pub snippet: String,
}

pub struct SearchResult {
    pub posts: Vec<Match>,
    pub matched_terms: HashSet<String>,
}

pub fn find(
    posts_dir: &Path,
    terms: &[String],
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    // Keep punctuation in names such as AGENTS.md and require word-like
    // boundaries, so "Codex" does not match a longer identifier.
    let patterns = terms
        .iter()
        .map(|term| {
            Regex::new(&format!(
                r"(?i)(?:^|[^\p{{L}}\p{{N}}]){}(?:$|[^\p{{L}}\p{{N}}])",
                regex::escape(term)
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut paths = fs::read_dir(posts_dir)?
        .map(|entry| entry.map(|item| item.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut found = Vec::new();
    let mut matched_terms = HashSet::new();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let (frontmatter, body) = split_frontmatter(&raw)?;
        let slug = frontmatter_slug(frontmatter)
            .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().into_owned());
        let mut first_match = None;
        for paragraph in body.split("\n\n") {
            let clean = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
            let matches = patterns
                .iter()
                .enumerate()
                .filter_map(|(index, pattern)| {
                    pattern
                        .find_iter(&clean)
                        .find(|found| {
                            // An ordinary word in AGENTS.md is a filename mention,
                            // not a mention of the word "agents" by itself.
                            !(!terms[index].contains('.')
                                && found.as_str().ends_with('.')
                                && clean[found.end()..]
                                    .get(..2)
                                    .is_some_and(|suffix| suffix.eq_ignore_ascii_case("md")))
                        })
                        .map(|found| (index, found))
                })
                .collect::<Vec<_>>();
            for (index, _) in &matches {
                matched_terms.insert(terms[*index].to_lowercase());
            }
            let Some((_, found_at)) = matches.into_iter().min_by_key(|(_, found)| found.start())
            else {
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
            if first_match.is_none() {
                first_match = Some(snippet);
            }
        }
        if let Some(snippet) = first_match {
            found.push(Match { slug, snippet });
        }
    }
    Ok(SearchResult {
        posts: found,
        matched_terms,
    })
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
