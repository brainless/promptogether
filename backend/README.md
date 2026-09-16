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

Before risky migrations, copy the database file:

```sh
cp promptogether.db promptogether.db.backup-$(date +%Y%m%d)
```

## Environment variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite connection string. | `sqlite:promptogether.db?mode=rwc` |
| `BIND_ADDRESS` | Server bind address. | `127.0.0.1:3000` |
| `LOG_LEVEL` / `RUST_LOG` | Tracing filter. | `backend=info,tower_http=info` |
