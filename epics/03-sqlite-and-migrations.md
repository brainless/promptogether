# Epic 03: SQLite persistence and migrations

## Outcome

The backend has one async SQLite connection layer and a repeatable way to create
and upgrade databases in development and deployment.

## Tasks

- [ ] Configure a SQLx `SqlitePool` for Tokio with foreign keys enabled, WAL
      mode, a busy timeout, and bounded connections suitable for SQLite.
- [ ] Add numbered SQL migrations under `backend/migrations/` and embed them in
      the backend binary with SQLx.
- [ ] Add `backend migrate` for explicit migration runs and make `serve` fail
      clearly when the database schema is behind.
- [ ] Create the initial gallery and job tables, indexes, foreign keys, and
      constraints needed by Epics 4 and 5.
- [ ] Add a local database bootstrap command and document migration creation,
      forward-only changes, and backup expectations.
- [ ] Keep SQL rows internal to `backend`; map them to `api-types` at the API
      boundary.

## Initial schema

- `gallery_projects`: stable slug, title, summary, prompt text, publication
  state, display order, and timestamps.
- `gallery_files`: project reference, phase (`initial` or `result`), path,
  optional language, content, and display order; unique by project, phase, and
  path.
- `jobs`: kind, versioned JSON payload, status, attempt counts, availability and
  lease times, last error, and timestamps.

## Done when

- A blank database reaches the latest schema with one command.
- Re-running migrations is safe, and an outdated database cannot silently run
  the API.
- SQLite connection settings are applied consistently by the server, worker,
  and migration command.

