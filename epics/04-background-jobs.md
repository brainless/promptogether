# Epic 04: Durable background jobs

## Outcome

Long-running work can be queued without holding an HTTP request open. Jobs
survive process restarts, and a worker can retry or recover them without a
stale worker completing work it no longer owns. This provides the execution
foundation for later AI helpers without introducing an AI provider.

## Existing foundation

- Epic 03 created the `jobs` table and the shared SQLite pool configuration.
- SQL rows stay internal to `backend`; job payloads are backend-domain types,
  not public `api-types` contracts.
- Every long-running command or command that reads application data must use
  the embedded-migration compatibility check. Add a new forward-only migration
  for any schema change; do not edit `001_initial_schema.sql` after it has
  shipped.

## Tasks

- [x] Define a versioned job envelope and a closed, typed job-kind registry in
      `backend`. Enqueue typed payloads only, and report unknown kinds, unknown
      payload versions, and malformed JSON explicitly when reading existing
      rows.
- [x] Add a forward-only migration with a unique lease token (and any other
      lease-ownership data required) so renewal, completion, and failure are
      compare-and-set operations. Generate a new token on every claim and clear
      it when leaving the running state. A worker whose lease has expired or
      been reclaimed must not be able to update the new owner's job.
- [x] Implement a repository layer for enqueue, single-statement or
      transactionally atomic claim, lease renewal, completion, retry, and
      terminal failure. Use SQLite/database time consistently, deterministic
      claim order (`available_at`, then `id`), and never hold a transaction
      open while a handler runs.
- [x] Define lifecycle semantics in code: the statuses allowed, when
      `attempts` increments, the lease duration, capped exponential backoff,
      and the transition to terminal failure at `max_attempts`. Expired leases
      must become claimable without a separate manual repair step.
- [x] Add `cargo run -p backend -- worker`. Load and validate worker settings
      such as concurrency, polling interval, lease duration, and shutdown
      timeout; reject zero, contradictory, non-numeric, or non-Unicode values
      with the setting name in the startup error.
- [x] Keep worker concurrency bounded for the existing five-connection SQLite
      pool. Each concurrent handler gets its own lease-renewal loop, and one
      failed claim or handler must not stop the worker process.
- [x] On SIGINT or SIGTERM, stop polling and claiming first, allow active
      handlers to finish for the configured timeout, then stop renewal tasks
      and exit. Jobs left in progress must be recoverable after their leases
      expire.
- [x] Add an internal fixture job with deterministic success and failure modes,
      plus a clearly development-only CLI path to enqueue it, so claim,
      renewal, retry, completion, terminal failure, and restart recovery can be
      exercised without network access or secrets.
- [x] Emit structured logs containing job ID, kind, payload version, attempt,
      duration, and outcome. Do not log payload bodies or error details that
      may contain prompts, credentials, or other secrets; persist a bounded,
      useful `last_error`.
- [x] Document the worker command, every setting and default, lifecycle and
      retry behavior, and a manual fixture-job walkthrough in `backend/README.md`.

## Done when

- A migrated database can enqueue a fixture job and a separate worker process
  completes it.
- A deterministic failure is retried with backoff and becomes terminal after
  exactly the configured maximum attempts.
- Killing a worker after claim does not permanently strand the job, and a
  superseded worker cannot renew or complete a lease reclaimed by another
  worker.
- The worker refuses an incompatible database schema and invalid worker
  configuration with actionable errors.

## Scope boundary

This epic supplies internal execution infrastructure only. It adds no public
job endpoint or public job type. AI prompts, providers, credentials, quotas,
and user-facing progress are separate future work.
