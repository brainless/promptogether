# Prompt Together Backend

Rust API server built with Axum and SQLx (SQLite).

## Quick start

```sh
cargo run -- bootstrap
cargo run -- serve
```

`bootstrap` creates a fresh database and applies all migrations. `serve` starts the API server.

## Commands

| Command | Description |
|---------|-------------|
| `cargo run -- bootstrap` | Create a fresh database and apply all migrations. |
| `cargo run -- migrate` | Apply pending migrations to an existing database. |
| `cargo run -- serve` | Start the API server (fails if migrations are pending). |
| `cargo run -- worker` | Start the background worker (polls and processes jobs). |
| `cargo run -- enqueue-fixture` | Enqueue a dev-only fixture job. Pass `--fail` for a failing job. |

`serve` is the default when no subcommand is given.

## Creating new migrations

Migrations live in `backend/migrations/`.

**Naming convention:** `NNN_description.sql`

```
001_initial_schema.sql
002_add_user_table.sql
003_add_tags.sql
```

Numbers must be unique and sequential. Migrations are embedded at compile time via `sqlx::migrate!()`.

After adding a migration file, rebuild the binary so it is included.

## Forward-only policy

- All migrations must be forward-only. There are no down migrations.
- Never modify an already-applied migration file. Create a new migration instead.
- If a migration has a bug, create a corrective migration with the next sequential number.

## Backup expectations

The SQLite database file is `promptogether.db` (configurable via `DATABASE_URL`).

Before risky migrations, use SQLite's online backup command. It creates a consistent
snapshot even when the database is using WAL mode:

```sh
sqlite3 promptogether.db ".backup 'promptogether.db.backup-$(date +%Y%m%d-%H%M%S)'"
```

If `DATABASE_URL` points elsewhere, replace `promptogether.db` with its database
file path.

Alternatively, stop the server and every other process using the database, then
checkpoint the WAL before copying the main file:

```sh
sqlite3 promptogether.db "PRAGMA wal_checkpoint(TRUNCATE);"
cp promptogether.db promptogether.db.backup-$(date +%Y%m%d-%H%M%S)
```

The checkpoint result must report `0` in its first field (no busy connections)
before copying. Never copy only `promptogether.db` while the application is
running: committed data may still be in `promptogether.db-wal`.

## Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite connection string. | `sqlite:promptogether.db?mode=rwc` |
| `BIND_ADDRESS` | Server bind address. | `127.0.0.1:3000` |
| `LOG_LEVEL` / `RUST_LOG` | Tracing filter. | `backend=info,tower_http=info` |
| `WORKER_CONCURRENCY` | Max concurrent job handlers (1-4). | `2` |
| `WORKER_POLL_INTERVAL_MS` | Milliseconds between poll cycles (min 100). | `1000` |
| `WORKER_LEASE_DURATION_SECS` | Job lease duration in seconds (min 5). | `30` |
| `WORKER_SHUTDOWN_TIMEOUT_SECS` | Seconds to wait for handlers on shutdown (min 1). | `30` |

Concurrency is capped at 4 because the SQLite pool has 5 max connections.

## Worker command

```sh
cargo run -p backend -- worker
```

The worker polls the `jobs` table for pending work, claims jobs with a lease, and processes them concurrently up to `WORKER_CONCURRENCY`. It requires migrations to be applied first (same as `serve`).

The worker shuts down gracefully on SIGINT or SIGTERM: it stops claiming new jobs and waits up to `WORKER_SHUTDOWN_TIMEOUT_SECS` for in-flight handlers to finish.

## Enqueue fixture command

```sh
cargo run -p backend -- enqueue-fixture
cargo run -p backend -- enqueue-fixture --fail
```

Enqueues a development-only fixture job. With `--fail`, the job is enqueued with parameters that cause it to fail (useful for testing retry and terminal-failure paths).

This command is for local development and testing only. Do not use it in production.

## Job lifecycle

1. Jobs start as `pending`.
2. When claimed, a job becomes `running` with a lease token and its `attempts` counter is incremented.
3. The lease is renewed automatically every `lease_duration / 2` while the handler is active.
4. On success: the job is marked `completed`.
5. On failure with attempts remaining: the job returns to `pending` with exponential backoff.
6. On failure at max attempts: the job is marked `failed` (terminal).
7. An expired lease is atomically reclaimed when attempts remain. If its final
   allowed attempt expired, the job is marked `failed` instead of being run
   beyond `max_attempts`.

`jobs.last_error` contains only allowlisted, non-sensitive diagnostic categories
(for example, `malformed job payload`), never payload values, parser output, or
free-form handler errors. These messages are limited to 128 bytes. Add future
handler failures to the typed `JobFailure` taxonomy rather than persisting raw
error details.

**Backoff formula:** `min(base_backoff * 2^(attempts-1), max_backoff)` where `base_backoff` = 1s and `max_backoff` = 300s.

## Fixture job walkthrough

A step-by-step guide to exercise the worker locally.

```sh
# 1. Bootstrap the database
cargo run -p backend -- bootstrap

# 2. Enqueue a fixture job (success)
cargo run -p backend -- enqueue-fixture

# 3. Run the worker (it will claim and complete the job)
cargo run -p backend -- worker

# 4. Enqueue a failing fixture job
cargo run -p backend -- enqueue-fixture --fail

# 5. Run the worker again (it will retry and eventually mark as terminal failure)
cargo run -p backend -- worker
```
