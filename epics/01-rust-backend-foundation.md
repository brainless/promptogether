# Epic 01: Rust workspace and backend service

## Outcome

A small production-shaped Rust service can start, report its health, share
state safely, and shut down cleanly. It creates the seam for persistence,
workers, and the first public API without adding authentication.

## Tasks

- [x] Create a root Cargo workspace containing `backend` and `api-types`.
- [x] Scaffold `backend` with Axum, Tokio, Serde, tracing, and structured error
      responses.
- [x] Add configuration for bind address, database URL, log level, and local
      development defaults; validate configuration at startup.
- [x] Build an application-state type and route composition that keeps HTTP,
      persistence, and domain modules separate.
- [x] Add `GET /api/health` and graceful shutdown on process signals.
- [x] Document backend setup and commands in the repository README or a
      backend README.

## Done when

- `cargo run -p backend -- serve` starts the API and the health endpoint returns
  a stable JSON response.
- Startup failures identify the invalid or missing setting.
- `cargo check --workspace` succeeds.

