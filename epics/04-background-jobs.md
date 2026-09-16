# Epic 04: Durable background jobs

## Outcome

Long-running work can be queued without holding an HTTP request open. Jobs
survive restarts and provide the basic reliability needed for later AI helpers.

## Tasks

- [ ] Define a versioned job envelope and typed job-kind registry inside the
      backend; reject unknown kinds or payload versions visibly.
- [ ] Implement enqueue, atomic claim, lease renewal, completion, retry with
      backoff, and terminal failure against the `jobs` table.
- [ ] Add `cargo run -p backend -- worker` with configurable concurrency and
      polling intervals.
- [ ] Recover jobs whose worker lease expired, cap retries, and retain useful
      error details without storing secrets in logs.
- [ ] Add graceful shutdown that stops claiming jobs and lets active handlers
      finish within a configured timeout.
- [ ] Add a small no-op or fixture job handler to exercise the whole lifecycle
      before an AI provider is introduced.
- [ ] Emit structured logs with job ID, kind, attempt, duration, and outcome.

## Done when

- A queued fixture job is completed by a separate worker process.
- A failed job retries and eventually reaches a terminal state.
- Restarting a worker does not permanently strand a claimed job.

## Scope boundary

This epic supplies execution infrastructure only. AI prompts, providers,
credentials, quotas, and user-facing progress are separate future work.

