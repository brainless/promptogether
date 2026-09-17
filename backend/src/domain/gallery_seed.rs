use sqlx::SqlitePool;

struct SeedProject {
    slug: &'static str,
    title: &'static str,
    summary: &'static str,
    prompt: &'static str,
    display_order: i64,
    files: Vec<SeedFile>,
}

struct SeedFile {
    phase: &'static str,
    path: &'static str,
    language: Option<&'static str>,
    content: &'static str,
    display_order: i64,
}

fn curated_projects() -> Vec<SeedProject> {
    vec![
        SeedProject {
            slug: "random-password-gen",
            title: "Random Password Generator",
            summary: "Create a Rust CLI that generates cryptographically random passwords of a configurable length.",
            prompt: "Create a Rust command-line tool that generates a random password. Accept an optional length argument (default 16) and print the password to stdout. Use only the standard library and the rand crate. The password should contain a mix of uppercase, lowercase, digits, and symbols.",
            display_order: 1,
            files: vec![
                SeedFile {
                    phase: "result",
                    path: "Cargo.toml",
                    language: Some("toml"),
                    content: "[package]\nname = \"random-password-gen\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nrand = \"0.8\"\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "result",
                    path: "src/main.rs",
                    language: Some("rust"),
                    content: "use rand::seq::SliceRandom;\nuse std::{env, process};\n\nconst LOWERCASE: &[u8] = b\"abcdefghijklmnopqrstuvwxyz\";\nconst UPPERCASE: &[u8] = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ\";\nconst DIGITS: &[u8] = b\"0123456789\";\nconst SYMBOLS: &[u8] = b\"!@#$%^&*\";\nconst ALL: &[u8] = b\"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*\";\n\nfn main() {\n    let len: usize = env::args()\n        .nth(1)\n        .and_then(|a| a.parse().ok())\n        .unwrap_or(16);\n\n    if len < 4 {\n        eprintln!(\"Password length must be at least 4.\");\n        process::exit(2);\n    }\n\n    let mut rng = rand::thread_rng();\n    let mut password = vec![\n        *LOWERCASE.choose(&mut rng).unwrap(),\n        *UPPERCASE.choose(&mut rng).unwrap(),\n        *DIGITS.choose(&mut rng).unwrap(),\n        *SYMBOLS.choose(&mut rng).unwrap(),\n    ];\n\n    password.extend((password.len()..len).map(|_| *ALL.choose(&mut rng).unwrap()));\n    password.shuffle(&mut rng);\n\n    let password: String = password.into_iter().map(char::from).collect();\n    println!(\"{password}\");\n}\n",
                    display_order: 1,
                },
            ],
        },
        SeedProject {
            slug: "hello-rust-cli",
            title: "Hello Rust CLI",
            summary: "A minimal command-line program that prints a greeting using clap for argument parsing.",
            prompt: "Create a minimal Rust CLI application that accepts an optional name argument and prints \"Hello, <name>!\" defaulting to \"Hello, world!\" when no argument is provided. Use clap for argument parsing.",
            display_order: 2,
            files: vec![
                SeedFile {
                    phase: "initial",
                    path: "Cargo.toml",
                    language: Some("toml"),
                    content: "[package]\nname = \"hello-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "initial",
                    path: "src/main.rs",
                    language: Some("rust"),
                    content: "fn main() {\n    // TODO: implement CLI\n}\n",
                    display_order: 2,
                },
                SeedFile {
                    phase: "result",
                    path: "Cargo.toml",
                    language: Some("toml"),
                    content: "[package]\nname = \"hello-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nclap = { version = \"4\", features = [\"derive\"] }\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "result",
                    path: "src/main.rs",
                    language: Some("rust"),
                    content: "use clap::Parser;\n\n#[derive(Parser)]\n#[command(name = \"hello-cli\")]\nstruct Args {\n    /// Name to greet\n    #[arg(default_value = \"world\")]\n    name: String,\n}\n\nfn main() {\n    let args = Args::parse();\n    println!(\"Hello, {}!\", args.name);\n}\n",
                    display_order: 2,
                },
            ],
        },
        SeedProject {
            slug: "html-landing-page",
            title: "HTML Landing Page",
            summary: "Build a responsive landing page from scratch with semantic HTML and modern CSS.",
            prompt: "Create a responsive landing page for a fictional open-source project called \"Prompt Together\". Use only HTML and CSS with no JavaScript frameworks. The page should include a hero section, a features grid, and a footer. Make it mobile-first and accessible.",
            display_order: 3,
            files: vec![
                SeedFile {
                    phase: "initial",
                    path: "index.html",
                    language: Some("html"),
                    content: "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"UTF-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    <title>Prompt Together</title>\n    <link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n    <!-- TODO: build the page -->\n</body>\n</html>\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "initial",
                    path: "style.css",
                    language: Some("css"),
                    content: "/* TODO: add styles */\n",
                    display_order: 2,
                },
                SeedFile {
                    phase: "result",
                    path: "index.html",
                    language: Some("html"),
                    content: "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"UTF-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    <title>Prompt Together</title>\n    <link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n    <header class=\"hero\">\n        <h1>Prompt Together</h1>\n        <p>An inclusive platform for learning to build software with coding agents.</p>\n        <a href=\"#features\" class=\"cta\">Explore Features</a>\n    </header>\n    <main id=\"features\" class=\"features\">\n        <h2>Features</h2>\n        <div class=\"grid\">\n            <article>\n                <h3>Curated Prompts</h3>\n                <p>Browse a gallery of prompt examples for common tasks.</p>\n            </article>\n            <article>\n                <h3>Side-by-Side View</h3>\n                <p>Compare starting files with the agent-generated result.</p>\n            </article>\n            <article>\n                <h3>Open Source</h3>\n                <p>Explore, learn from, and contribute to the codebase.</p>\n            </article>\n        </div>\n    </main>\n    <footer>\n        <p>&copy; 2026 Prompt Together. Open source under the MIT license.</p>\n    </footer>\n</body>\n</html>\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "result",
                    path: "style.css",
                    language: Some("css"),
                    content: "* { box-sizing: border-box; margin: 0; padding: 0; }\nbody { font-family: system-ui, sans-serif; line-height: 1.6; color: #222; }\n.hero { text-align: center; padding: 4rem 1rem; background: #0d1117; color: #fff; }\n.hero h1 { font-size: 2.5rem; }\n.hero p { margin: 1rem 0 2rem; font-size: 1.125rem; }\n.cta { display: inline-block; padding: 0.75rem 1.5rem; background: #2563eb; color: #fff; text-decoration: none; border-radius: 0.375rem; }\n.features { padding: 3rem 1rem; max-width: 64rem; margin: 0 auto; }\n.features h2 { text-align: center; margin-bottom: 2rem; }\n.grid { display: grid; gap: 1.5rem; grid-template-columns: 1fr; }\n@media (min-width: 48rem) { .grid { grid-template-columns: repeat(3, 1fr); } }\narticle { padding: 1.5rem; border: 1px solid #e5e7eb; border-radius: 0.5rem; }\narticle h3 { margin-bottom: 0.5rem; }\nfooter { text-align: center; padding: 2rem 1rem; font-size: 0.875rem; color: #6b7280; }\n",
                    display_order: 2,
                },
            ],
        },
        SeedProject {
            slug: "python-word-counter",
            title: "Python Word Counter",
            summary: "A small Python script that reads text and counts word frequencies, sorted by count.",
            prompt: "Write a Python script that reads a block of text, counts how often each word appears (case-insensitive, ignoring punctuation), and prints the results sorted by frequency descending. Include a sample text inline so the script works standalone.",
            display_order: 4,
            files: vec![
                SeedFile {
                    phase: "initial",
                    path: "word_counter.py",
                    language: Some("python"),
                    content: "# TODO: implement word counter\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "result",
                    path: "word_counter.py",
                    language: Some("python"),
                    content: "import re\nfrom collections import Counter\n\nSAMPLE_TEXT = \"\"\"\nPrompt Together is an inclusive platform for learning to build software\nwith coding agents. Together we explore prompts, compare outputs, and\nlearn best practices for working with AI coding tools.\n\"\"\"\n\ndef count_words(text: str) -> list[tuple[str, int]]:\n    words = re.findall(r\"[a-z0-9]+\", text.lower())\n    return Counter(words).most_common()\n\nif __name__ == \"__main__\":\n    for word, count in count_words(SAMPLE_TEXT):\n        print(f\"{word:>12}  {count}\")\n",
                    display_order: 1,
                },
            ],
        },
        SeedProject {
            slug: "readme-template",
            title: "README Template",
            summary: "Transform a bare-bones README into a polished open-source project README with badges and sections.",
            prompt: "Given a minimal README with just a project title, expand it into a complete open-source README with a description, installation instructions, usage examples, a contributing section, and a license badge. Keep it concise and suitable for a small CLI tool.",
            display_order: 5,
            files: vec![
                SeedFile {
                    phase: "initial",
                    path: "README.md",
                    language: Some("markdown"),
                    content: "# hello-cli\n",
                    display_order: 1,
                },
                SeedFile {
                    phase: "result",
                    path: "README.md",
                    language: Some("markdown"),
                    content: "# hello-cli\n\n[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)\n\nA minimal command-line tool that prints a friendly greeting.\n\n## Installation\n\n```sh\ncargo install --path .\n```\n\n## Usage\n\n```sh\n# Default greeting\nhello-cli\n# Output: Hello, world!\n\n# Custom greeting\nhello-cli Alice\n# Output: Hello, Alice!\n```\n\n## Contributing\n\nContributions are welcome! Please open an issue or pull request.\n\n## License\n\nThis project is licensed under the MIT License.\n",
                    display_order: 1,
                },
            ],
        },
    ]
}

pub async fn seed(pool: &SqlitePool) -> Result<SeedSummary, sqlx::Error> {
    let projects = curated_projects();
    let mut tx = pool.begin().await?;

    let mut projects_upserted = 0u64;
    let mut files_upserted = 0u64;
    let mut files_removed = 0u64;

    for project in &projects {
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM gallery_projects WHERE slug = ?1",
        )
        .bind(project.slug)
        .fetch_optional(&mut *tx)
        .await?;

        let project_id = if let Some(id) = existing {
            let current = sqlx::query_as::<_, (String, String, String, i64, String)>(
                "SELECT title, summary, prompt, display_order, publication_state FROM gallery_projects WHERE id = ?1",
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            let needs_update = current.0 != project.title
                || current.1 != project.summary
                || current.2 != project.prompt
                || current.3 != project.display_order
                || current.4 != "published";

            if needs_update {
                sqlx::query(
                    "UPDATE gallery_projects SET title = ?1, summary = ?2, prompt = ?3, display_order = ?4, publication_state = 'published', updated_at = datetime('now') WHERE id = ?5",
                )
                .bind(project.title)
                .bind(project.summary)
                .bind(project.prompt)
                .bind(project.display_order)
                .bind(id)
                .execute(&mut *tx)
                .await?;
                projects_upserted += 1;
            }
            id
        } else {
            sqlx::query(
                "INSERT INTO gallery_projects (slug, title, summary, prompt, display_order, publication_state) VALUES (?1, ?2, ?3, ?4, ?5, 'published')",
            )
            .bind(project.slug)
            .bind(project.title)
            .bind(project.summary)
            .bind(project.prompt)
            .bind(project.display_order)
            .execute(&mut *tx)
            .await?;
            projects_upserted += 1;
            sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
                .fetch_one(&mut *tx)
                .await?
        };

        let curated_keys: Vec<(&str, &str)> = project
            .files
            .iter()
            .map(|f| (f.phase, f.path))
            .collect();

        let existing_files: Vec<(String, String)> = sqlx::query_as::<_, (String, String)>(
            "SELECT phase, path FROM gallery_files WHERE project_id = ?1",
        )
        .bind(project_id)
        .fetch_all(&mut *tx)
        .await?;

        for (phase, path) in &existing_files {
            if !curated_keys
                .iter()
                .any(|(p, pat)| *p == phase.as_str() && *pat == path.as_str())
            {
                sqlx::query(
                    "DELETE FROM gallery_files WHERE project_id = ?1 AND phase = ?2 AND path = ?3",
                )
                .bind(project_id)
                .bind(phase)
                .bind(path)
                .execute(&mut *tx)
                .await?;
                files_removed += 1;
            }
        }

        for file in &project.files {
            let existing_file = sqlx::query_as::<_, (i64, String, Option<String>, i64)>(
                "SELECT id, content, language, display_order FROM gallery_files WHERE project_id = ?1 AND phase = ?2 AND path = ?3",
            )
            .bind(project_id)
            .bind(file.phase)
            .bind(file.path)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some((file_id, cur_content, cur_language, cur_display_order)) = existing_file {
                let needs_update = cur_content != file.content
                    || cur_language.as_deref() != file.language
                    || cur_display_order != file.display_order;

                if needs_update {
                    sqlx::query(
                        "UPDATE gallery_files SET content = ?1, language = ?2, display_order = ?3 WHERE id = ?4",
                    )
                    .bind(file.content)
                    .bind(file.language)
                    .bind(file.display_order)
                    .bind(file_id)
                    .execute(&mut *tx)
                    .await?;
                    files_upserted += 1;
                }
            } else {
                sqlx::query(
                    "INSERT INTO gallery_files (project_id, phase, path, language, content, display_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .bind(project_id)
                .bind(file.phase)
                .bind(file.path)
                .bind(file.language)
                .bind(file.content)
                .bind(file.display_order)
                .execute(&mut *tx)
                .await?;
                files_upserted += 1;
            }
        }
    }

    tx.commit().await?;

    Ok(SeedSummary {
        projects_upserted,
        files_upserted,
        files_removed,
    })
}

pub struct SeedSummary {
    pub projects_upserted: u64,
    pub files_upserted: u64,
    pub files_removed: u64,
}
