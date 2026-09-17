# Epic 05: Prompt gallery API and sample data

## Outcome

Visitors can list curated prompt examples and open one project to compare its
starting files with the files materialized by a coding agent. Draft projects
and internal database identifiers never cross the public API boundary.

## Existing foundation

- Epic 02 already defines `GalleryProjectSummary`, `GalleryProjectDetail`,
  `GalleryProjectFile`, `GalleryProjectFilePhase`, and `ErrorResponse` in
  `api-types`, with committed TypeScript output under
  `webapp/src/api/generated/`.
- Epic 03 created the gallery tables, internal SQL row types, and initial
  row-to-wire mappings. Treat the migration as shipped: use a new forward-only
  migration if the schema must change.
- The backend already rejects missing, unknown, failed, or checksum-mismatched
  migrations before serving. The seed command must apply the same compatibility
  rule rather than operating against an unknown schema.

## Tasks

- [x] Review the existing gallery wire types against the endpoint responses.
      Keep identifiers and timestamps as strings, keep Serde and `ts-rs`
      optionality/naming aligned, and add types only when they describe a real
      public response. Regenerate with `scripts/export-api-types.sh`; never edit
      generated TypeScript by hand.
- [x] Implement `GET /api/gallery` for published projects only. Return summary
      fields in deterministic order by `display_order`, with a stable slug
      tie-breaker; do not fetch prompts or file contents for the list response.
- [x] Implement `GET /api/gallery/{slug}` with the prompt and separately
      ordered `initialFiles` and `resultFiles`. Query the project once and its
      files once—no per-file or per-project query loop—and order files by
      `display_order`, path, and a final stable key.
- [x] Define and document a bounded lowercase ASCII kebab-case slug format, and
      validate it before querying. Return `400` / `invalid_gallery_slug` for a
      malformed slug and `404` / `gallery_project_not_found` for a missing
      project, using the shared error envelope. Treat an existing draft slug as
      not found so publication state is not exposed.
- [x] Convert unexpected database or row-decoding failures into the shared
      error envelope without exposing SQL, filesystem paths, configuration, or
      other internal details. Do not silently map an unknown stored enum value
      to a valid public variant.
- [x] Add `cargo run -p backend -- seed-gallery` using curated data stored in a
      clear, reviewable backend source file. Run the seed in a transaction and
      upsert by stable project slug plus file phase/path, without depending on
      internal numeric IDs.
- [x] Make reseeding convergent, not merely duplicate-free: update changed
      curated fields, remove files that were removed or renamed in the curated
      definition for those projects, and leave unrelated projects untouched.
- [x] Include several concise, original or clearly license-safe examples for a
      general audience. Include at least one project created from an empty
      initial state and one that modifies existing files; use relative,
      normalized project paths only and no private or machine-specific content.
- [x] Document seed ownership, slug stability, publication-state behavior, and
      how a future dataset generator can perform the same transactional upsert
      without coupling to numeric keys.

## Done when

- A fresh local database can be bootstrapped, seeded, listed, and queried using
  the documented commands and public endpoints.
- Only published projects are observable; list and file order are stable even
  when display-order values tie; repeated seeding reaches the same database
  state.
- Every non-success gallery response uses `ErrorResponse`, and the Rust
  responses match the regenerated, committed TypeScript declarations.
- Responses and seed data contain no machine-specific paths, private content,
  internal numeric IDs, or provenance from a future personal dataset.

## Scope boundary

The gallery is curated and read-only over HTTP. Public creation, editing,
uploads, authentication, dataset generation, and invoking a coding agent are
not part of this epic.
