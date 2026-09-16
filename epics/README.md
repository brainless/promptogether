# Backend and Prompt Gallery Plan

These epics introduce the first end-to-end feature without treating the broader
product direction in `PRD.md` as implemented.

## Architecture decisions

- Use a root Cargo workspace with `backend` and `api-types` crates.
- Run the HTTP API with Axum on Tokio.
- Use SQLx's Tokio SQLite driver and versioned, embedded migrations.
- Keep durable background jobs in SQLite and run workers separately from the
  HTTP server, while allowing both processes to use the same backend crate.
- Define JSON request and response types in `api-types`; keep database and Axum
  types out of that crate.
- Generate deterministic TypeScript into `webapp/src/api/generated/` with a
  dedicated exporter command and commit the generated files.
- Serve the webapp and API under one origin in production. Proxy `/api` to the
  backend during Vite development.

## Delivery order

1. [Rust workspace and backend service](01-rust-backend-foundation.md)
2. [Shared API types and TypeScript exporter](02-api-types-and-typescript-export.md)
3. [SQLite persistence and migrations](03-sqlite-and-migrations.md)
4. [Durable background jobs](04-background-jobs.md)
5. [Prompt gallery API and sample data](05-prompt-gallery-api.md)
6. [Prompt gallery webapp](06-prompt-gallery-webapp.md)

Epics 2 and 3 can proceed once the workspace exists. Epic 4 establishes the
worker foundation for later AI helpers but is not required to fake asynchronous
work in the gallery. Epics 5 and 6 deliver the first visible vertical slice.

## External references

- [`ts-rs` documentation](https://docs.rs/ts-rs/latest/ts_rs/)
- [SQLx migration macro](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)
- [Axum documentation](https://docs.rs/axum/latest/axum/)

