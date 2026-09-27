# Epic 08: Concepts from video posts

## Outcome

Readers can follow unfamiliar terms from a post to a plain-language Concepts
index and find the posts that discuss each term. Maintainers curate concepts in
one Git-versioned SQLite file. A Rust CLI can search the Markdown posts, draft a
definition with an LLM, and save the definition and post links. The webapp reads
the database during its build and publishes static pages without a concepts API.

**Status:** Implemented locally. The web build and Rust compile check passed;
manual browser review, a live LLM call, and a Cloudflare Pages deployment have
not been performed as part of this work.

## Context and decisions

- Video transcripts produced with Epic 07 can become Markdown posts under
  `webapp/content/posts/`. Five posts are included with this work.
- `webapp/content/concepts.sqlite` is the sole source of truth for concepts. It
  is separate from the backend application database and is versioned with the
  site in Git. There is no generated JSON data file to synchronize.
- Cloudflare Pages runs the webapp build against the checked-out repository.
  Node's `node:sqlite` reads the database at build time; the database is not
  copied to `dist/` or queried by visitors.
- The CLI uses the same pinned `llm-sdk` revision as the video-to-Markdown CLI
  and defaults to Xiaomi's `mimo-v2.6-flash` chat model. The `--model` option
  allows another Xiaomi chat model. Credentials come from `XIAOMI_API_KEY` in
  the environment or the repository-root `.env` file.
- Explicit concept-to-post rows determine the sidebar and related-post lists.
  Matching words in transcript text alone do not create a relationship.

## Delivered workflow

1. Run `cargo run -p concept-cli -- "TERM"` from the repository root. The CLI
   searches the body of each Markdown post for the term and collects one
   representative excerpt per matching post. It refuses a term with no matches.
2. The CLI sends up to eight excerpts as user content with a dedicated system
   prompt to the selected Xiaomi model through `llm-sdk`. It rejects empty,
   incomplete, overly long, or multiline responses before writing.
3. A successful run saves the definition and matched post slugs in one SQLite
   transaction. An existing concept is protected unless `--overwrite` is
   supplied; overwrite keeps its slug, external URL, and aliases while
   replacing its definition and post links. The CLI uses SQLite's delete
   journal mode so committed changes reside in the main database file rather
   than a WAL sidecar.
4. Run `npm run build` from `webapp/`. Vite reads the Markdown posts and SQLite
   concepts into virtual modules for the Solid app. The post-build generator
   reads the same sources and writes static HTML for `/posts`, each post, and
   `/concepts`. The build rejects concept links to missing post slugs.
5. Review the generated definition and site, then commit the changed SQLite
   file alongside any new posts it references. The Vite dev server reloads
   its content modules when a post or the database changes.

## Data and reading experience

The database contains `concepts` (slug, name, definition, optional external
URL), `concept_aliases` (alternate words in transcripts), and `concept_posts`
(explicit post slugs). The entries cover Claude Code, Codex, `AGENTS.md`, LLM,
context window, and harness. Post slugs come from Markdown filenames
unless frontmatter supplies a `slug`.

On a related post, the first plain-text mention of a concept name or alias
links to `/concepts#<slug>`. Existing links and code are left alone. A sidebar
lists every concept explicitly linked to that post, even when the exact term
does not appear in the transcript. The Concepts page lists definitions,
optional external references, and links back to related posts. Both the Solid
pages and the pre-rendered HTML use the same build-time database reader.

## Verification and limits

- `cargo check -p concept-cli --offline` passed with the configured `sccache`
  wrapper disabled.
- `npm run build` passed, including TypeScript checking, Vite bundling, and
  pre-rendering of five post pages, the posts index, and the Concepts page.
- SQLite reported `integrity_check = ok`. The database was confirmed to use
  delete journal mode. No SQLite file appeared in `dist/`.
- Automated tests and browser checks were not run, following the project's
  manual verification preference. The LLM request and Cloudflare deployment
  still need manual review.
- New post files referenced by the database must be committed with it for a
  clean build from Git.

## Relevant files

- `concept-cli/`: argument handling, post search, LLM definition drafting,
  and transactional SQLite updates.
- `webapp/content/concepts.sqlite`: versioned authoring database.
- `webapp/scripts/concepts.mjs`: shared read-only build-time database loader.
- `webapp/vite.config.ts` and `webapp/scripts/prerender-posts.mjs`: browser
  bundle and static page generation.
- `webapp/src/pages/Concepts.tsx` and `webapp/src/pages/Post.tsx`: the concept
  index and post sidebar.
- `concept-cli/README.md` and `webapp/README.md`: maintainer commands and
  deployment setup.
