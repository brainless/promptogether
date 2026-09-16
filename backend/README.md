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
