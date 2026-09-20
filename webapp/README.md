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

`npm run build` checks types, builds the SPA to `dist/`, and then generates a static HTML page per post under `dist/posts/` (see Posts below). Use `npm run preview` to serve the production build locally.

## Routes

| Path | Page |
|------|------|
| `/` | Homepage |
| `/gallery` | Gallery — browse all published projects |
| `/gallery/:slug` | Project detail — prompt, initial files, and result files |
| `/posts` | Posts — blog index |
| `/posts/:slug` | Post — a single post, statically pre-rendered for SEO |

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

`npm run dev` picks up new/edited posts on file change (full reload). `npm run build` renders each post to `dist/posts/<slug>/index.html` with a post-specific `<title>`, meta description, and Open Graph/Twitter tags — important because the rest of the app is a client-rendered SPA that search engines and link-preview crawlers can't reliably read, but these pages are plain static HTML per post. Only `/posts` and `/posts/:slug` get this treatment; the homepage and gallery remain a single client-rendered `index.html`.

## Production redirects (Cloudflare Pages)

`public/_redirects` is copied into `dist/` on build and covers two things: explicit rewrites so a clean URL like `/posts/<slug>` resolves to its prerendered `dist/posts/<slug>/index.html`, and the SPA fallback (`/* /index.html 200`) for client-side routing on `/gallery` and other in-app routes. No dashboard configuration should be needed on Cloudflare Pages beyond a default static deploy.

## Conventions

The app follows the local Solid repository's examples, using compatible published Solid 2.0 release candidates and a matching Vite plugin prerelease. Dependencies are installed from npm; the app does not require the local source checkouts.

"Start a Project" links to the homepage's introductory project section. Tours, community, Q&A, and AI support are described as upcoming features.
