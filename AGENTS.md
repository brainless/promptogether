# Prompt Together

Prompt Together (promptogether.com) is an inclusive platform for learning to build software with coding agents. Read `PRD.md` for the high-level product direction. Features will evolve; do not treat planned features as already implemented.

## Local reference sources

- SolidJS source: `/Users/brainless/Projects/solid` (`~/Projects/solid`).
- SolidJS documentation: `/Users/brainless/Projects/solid-docs` (`~/Projects/solid-docs`).

Consult these checkouts for Solid 2.0 APIs, examples, and conventions. They are reference repositories, not dependencies of this project. Their example package versions may differ from compatible published versions; check `webapp/package.json` and its lockfile before changing dependencies. Use Solid 2.0 conventions, including rendering through `@solidjs/web`.

## Project layout

- `PRD.md`: high-level product requirements.
- `webapp/`: SolidJS 2.0 web application using TypeScript, Vite, and component-scoped CSS modules.
- `webapp/src/App.tsx`: homepage with separate, stacked sections and a “Start a Project” navigation link to the on-page starting section.
- `webapp/src/App.module.css`: component styles.
- `webapp/src/index.css`: global styles.
- `webapp/src/main.tsx`: application entry point.

Keep the interface minimal, mobile-first, accessible, and free to explore without registration. The current implementation is homepage-only. The planned backend is Rust; no backend has been scaffolded yet.

## Webapp commands

Run from `webapp/`:

```sh
npm install
npm run dev
npm run build
npm run preview
```

`build` checks TypeScript and creates the production output in `dist/`. After dependency changes, `npm run dev -- --force` refreshes Vite's dependency cache. See `webapp/README.md` for setup details.

## Verification preference

The user prefers to test manually. Do not run automated tests or browser checks unless requested. Report changes and any verification limitations clearly.
