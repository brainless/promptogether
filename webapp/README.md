# Prompt Together webapp

Mobile-first web application built with SolidJS 2.0, TypeScript, Vite, and component-scoped CSS modules. Includes the homepage and a read-only prompt gallery.

## Local development

The gallery requires the Rust backend to be running with seeded data.

```sh
# 1. Bootstrap the database and seed gallery data (from repo root)
cargo run -p backend -- bootstrap
cargo run -p backend -- seed-gallery

# 2. Start the backend API server
cargo run -p backend -- serve

# 3. Start the Vite dev server (from webapp/)
cd webapp
npm install
npm run dev
```

The Vite dev server proxies `/api` requests to `http://localhost:3000` by default. To use a different backend origin, set the `VITE_API_ORIGIN` environment variable:

```sh
VITE_API_ORIGIN=http://localhost:4000 npm run dev
```

## Build

`npm run build` checks types, reads `content/concepts.sqlite`, builds the SPA, and generates static HTML for posts and `/concepts`. Use `npm run preview` to serve the production build locally. Reading SQLite at build time requires Node.js 22.16 or newer for `node:sqlite`.

## Routes

| Path | Page |
|------|------|
| `/` | Homepage |
| `/gallery` | Gallery — browse all published projects |
| `/gallery/:slug` | Project detail — prompt, initial files, and result files |
| `/posts` | Posts — blog index |
| `/posts/:slug` | Post — a single post, statically pre-rendered for SEO |
| `/concepts` | Concepts — a static index of terms, definitions, and related posts |

## Posts

Posts are Markdown files with YAML frontmatter, dropped into `content/posts/` (a cleaned transcript from `video-to-markdown-cli`'s output in `video-from-text/` works as-is once you add frontmatter). Any `.md` file there becomes a post automatically — no code changes needed.

```markdown
---
title: "My post title"
date: "2026-09-20"
youtube_url: "https://www.youtube.com/watch?v=..."   # optional — renders a YouTube embed on the post
description: "Optional; falls back to an excerpt of the body"
slug: "custom-slug"                                    # optional; defaults to the filename
---

Post body in Markdown.
```

`npm run dev` picks up new/edited posts on file change (full reload). `npm run build` renders each post to `dist/posts/<slug>/index.html` with a post-specific `<title>`, meta description, and Open Graph/Twitter tags. `/concepts` also gets static HTML. The homepage and gallery remain a client-rendered `index.html`.

## Concepts

`content/concepts.sqlite` is the committed source of truth for the concept index. The tables are `concepts` (slug, display name, short definition, optional external URL), `concept_aliases` (alternate terms in transcripts), and `concept_posts` (explicit links to post slugs). Cloudflare Pages reads the file while building; only the generated site in `dist/` is deployed.

From the repository root, use the Rust CLI to find a concept in the Markdown posts, ask MiMo V2.6 Flash for a short definition, and save it locally:

```sh
export XIAOMI_API_KEY="..."
cargo run -p concept-cli -- "coding agent"
cd webapp && npm run build
```

Use `--overwrite` to regenerate an existing concept. You can also edit the database with `sqlite3`:

```sh
sqlite3 content/concepts.sqlite "INSERT INTO concepts (slug, name, definition) VALUES ('prompt', 'Prompt', 'An instruction or question given to an AI model.');"
sqlite3 content/concepts.sqlite "INSERT INTO concept_posts (concept_slug, post_slug) VALUES ('prompt', 'Introduction');"
```

The build rejects links to post slugs that do not exist. Post slugs use the Markdown filename unless frontmatter sets `slug`. The post page sidebar lists explicitly linked concepts. Its body links the first matching plain-text mention of each concept or alias to `/concepts`; existing links and code are left alone. A concept can still appear in the sidebar when its exact term is absent from the transcript. The dev server reloads after database changes. Commit the SQLite file with concept changes; no JSON export is needed.

Only generated HTML, CSS, and JavaScript go to `dist/`. Visitors make no concept API requests and do not download the SQLite file.

## Production redirects (Cloudflare Pages)

`public/_redirects` is copied into `dist/` on build. It rewrites clean `/posts`, `/posts/<slug>`, and `/concepts` URLs to their prerendered files and retains the SPA fallback for other routes. On Cloudflare Pages, set the project root to `webapp`, build command to `npm run build`, and output directory to `dist`. Use a build image with Node.js 22.16 or newer.

## Conventions

The app follows the local Solid repository's examples, using compatible published Solid 2.0 release candidates and a matching Vite plugin prerelease. Dependencies are installed from npm; the app does not require the local source checkouts.

"Start a Project" links to the homepage's introductory project section. Tours, community, Q&A, and AI support are described as upcoming features.
