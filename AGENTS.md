# Prompt Together

Prompt Together (promptogether.com) is an inclusive platform for learning to build software with coding agents. Read `PRD.md` for the high-level product direction. Features will evolve; do not treat planned features as already implemented.

## Local reference sources

- SolidJS source: `/Users/brainless/Projects/solid` (`~/Projects/solid`).
- SolidJS documentation: `/Users/brainless/Projects/solid-docs` (`~/Projects/solid-docs`).

Consult these checkouts for Solid 2.0 APIs, examples, and conventions. They are reference repositories, not dependencies of this project. Their example package versions may differ from compatible published versions; check `webapp/package.json` and its lockfile before changing dependencies. Use Solid 2.0 conventions, including rendering through `@solidjs/web`.

## Project layout

- `PRD.md`: high-level product requirements.
- `epics/`: implementation plans and delivery tracking. Completed work currently
  covers the Rust foundation, shared API contracts, SQLite persistence, and
  durable background jobs; do not infer that later epics are implemented.
- `Cargo.toml`: Rust workspace containing `backend` and `api-types`.
- `backend/`: Axum/Tokio API service using SQLx with SQLite. See
  `backend/README.md` for commands, configuration, migration operations, and
  the background-worker walkthrough.
- `backend/migrations/`: embedded, forward-only SQLite migrations. Never edit a
  shipped migration; add the next sequential migration for schema changes.
- `api-types/`: source of truth for shared JSON request and response contracts,
  plus the `export-api-types` binary.
- `scripts/export-api-types.sh`: regenerates or checks the committed TypeScript
  bindings in `webapp/src/api/generated/`. Do not edit generated bindings by
  hand.
- `webapp/`: SolidJS 2.0 web application using TypeScript, Vite, and component-scoped CSS modules.
- `webapp/src/App.tsx`: homepage with separate, stacked sections and a “Start a Project” navigation link to the on-page starting section.
- `webapp/src/App.module.css`: component styles.
- `webapp/src/index.css`: global styles.
- `webapp/src/main.tsx`: application entry point.

Keep the interface minimal, mobile-first, accessible, and free to explore without registration. The visible web experience is currently homepage-only; the prompt gallery epics remain planned work.

## Backend conventions

- Keep transport contracts in `api-types`; SQL rows and internal job payloads
  remain private to `backend`.
- After changing `api-types`, run `scripts/export-api-types.sh` and commit the
  resulting files under `webapp/src/api/generated/`. Use
  `scripts/export-api-types.sh check` to detect stale generated contracts.
- Long-running application commands must validate the embedded migration set.
  `serve` and `worker` refuse missing, pending, unknown, incomplete, or
  checksum-mismatched migrations.
- Durable jobs use typed, versioned payloads and SQLite leases. Claims and
  state transitions must preserve lease ownership, bounded concurrency,
  maximum-attempt limits, graceful shutdown renewal, and the allowlisted,
  non-sensitive `last_error` policy documented in `backend/README.md`.
- `enqueue-fixture` and its `--fail` mode are development-only tools for
  manually exercising worker lifecycle behavior.

## Rust workspace commands

Run from the repository root:

```sh
cargo check --workspace
cargo build
cargo clippy --workspace -- -D warnings
cargo run -p backend -- bootstrap
cargo run -p backend -- migrate
cargo run -p backend -- serve
cargo run -p backend -- worker
cargo run -p backend -- enqueue-fixture [--fail]
scripts/export-api-types.sh
scripts/export-api-types.sh check
```

`bootstrap` creates a database and applies migrations. `migrate` applies
pending migrations. `serve` and `worker` require a compatible, fully migrated
database rather than applying migrations implicitly. The SQLite pool has five
connections, so worker concurrency is capped at four. Configuration and
defaults are documented in `backend/README.md`.

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
