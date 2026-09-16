# Prompt Together

An inclusive platform for learning to build software with coding agents. Free to explore with no registration required. See `PRD.md` for product direction.

## Repository layout

- `webapp/` -- SolidJS 2.0 frontend (TypeScript, Vite, CSS modules). See `webapp/README.md`.
- `backend/` -- Rust API service (Axum, Tokio, tracing).
- `api-types/` -- Shared request/response types used by `backend`, plus the binary that generates the TypeScript bindings consumed by `webapp`; the source of truth for the JSON contract.
- `epics/` -- Planning and task-tracking documents.

The root `Cargo.toml` defines a workspace containing `backend` and `api-types`.

## Rust workspace

### Prerequisites

- Rust (the repo pins the nightly toolchain via `rust-toolchain.toml`).
- `cargo`, `clippy`, and `rustfmt` are included with rustup.

### Commands

Run from the repository root.

```sh
cargo check --workspace
cargo build
cargo clippy --workspace -- -D warnings
```

To run the server:

```sh
cargo run -p backend -- serve
```

The server binds to `127.0.0.1:3000` by default and logs startup information.

### Configuration

All settings are read from environment variables at startup. Invalid configuration causes a fast exit with a clear error message.

| Variable | Default | Description |
| --- | --- | --- |
| `BIND_ADDRESS` | `127.0.0.1:3000` | Socket address to bind (e.g. `0.0.0.0:3000`). |
| `DATABASE_URL` | `postgres://localhost:5432/promptogether` | Postgres connection string. Not yet used. |
| `LOG_LEVEL` / `RUST_LOG` | `backend=info,tower_http=info` | tracing-filter directive. `LOG_LEVEL` takes precedence; falls back to `RUST_LOG`. |

### Endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/api/health` | Returns `{"status":"ok","app":"promptogether","version":"0.1.0"}`. |
| `GET` | `/api/ping` | Dev placeholder that includes the bind address in the response. |

The server shuts down gracefully on SIGINT or SIGTERM.

## Generated TypeScript API bindings

`webapp/src/api/generated/` holds the TypeScript declarations for the shared
API types, produced by the `export-api-types` binary in the `api-types` crate.
They are generated and must **not be edited by hand**; change the types in
`api-types`, then regenerate.

One documented command (from the repository root) regenerates the complete
TypeScript contract:

```sh
scripts/export-api-types.sh             # regenerate webapp/src/api/generated/
scripts/export-api-types.sh check       # fail unless committed bindings are current
```

The `check` variant regenerates into a temporary directory and fails if the
committed bindings are stale or nondeterministic, so it is safe for CI. Each
generated file carries a header noting it is generated and must not be edited
manually.
