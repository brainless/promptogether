# Epic 05: Prompt gallery API and sample data

## Outcome

Visitors can list curated prompt examples and open one project to compare its
starting files with the files materialized by a coding agent.

## Tasks

- [ ] Finalize shared contracts for `GalleryProjectSummary`,
      `GalleryProjectDetail`, and `GalleryProjectFile`, then regenerate the
      TypeScript bindings.
- [ ] Implement `GET /api/gallery` for published projects in display order with
      summary fields only.
- [ ] Implement `GET /api/gallery/{slug}` with the prompt, initial files, and
      result files; return the shared error envelope for invalid or missing
      slugs.
- [ ] Add repository queries with deterministic file ordering and no
      per-project query loop.
- [ ] Add an idempotent seed command containing several
      approachable, license-safe examples for a general audience.
- [ ] Include at least one new project and one modification of an existing
      project so both empty and non-empty initial states are represented.
- [ ] Document how the future dataset generator should upsert projects and
      files without coupling it to internal numeric keys.

## Done when

- A fresh local database can be migrated, seeded, listed, and queried through
  the public endpoints.
- Only published examples appear, slugs are stable, and repeated seeding does
  not duplicate data.
- Responses contain no machine-specific paths, private content, or provenance
  from the future personal dataset.
