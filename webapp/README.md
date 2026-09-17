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

`npm run build` checks types and builds to `dist/`. Use `npm run preview` to serve the production build locally.

## Routes

| Path | Page |
|------|------|
| `/` | Homepage |
| `/gallery` | Gallery — browse all published projects |
| `/gallery/:slug` | Project detail — prompt, initial files, and result files |

## Production SPA fallback

The gallery uses client-side routing. Direct links to `/gallery` or `/gallery/:slug` require the production server to serve `index.html` for all paths that do not match a static asset. Configure your hosting provider with a rewrite rule that falls back to `/index.html` for unknown paths.

## Conventions

The app follows the local Solid repository's examples, using compatible published Solid 2.0 release candidates and a matching Vite plugin prerelease. Dependencies are installed from npm; the app does not require the local source checkouts.

"Start a Project" links to the homepage's introductory project section. Tours, community, Q&A, and AI support are described as upcoming features.
