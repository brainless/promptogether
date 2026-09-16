# Epic 02: Shared API types and TypeScript export

## Outcome

Rust owns the public JSON contract, and the webapp consumes generated
TypeScript declarations from a separate `api-types` crate.

## Tasks

- [x] Keep `api-types` limited to serializable wire models with no Axum, SQLx,
      or backend-domain dependencies.
- [x] Define common API types, including health, an error envelope, gallery
      summaries, gallery detail, and project files.
- [x] Derive Serde and `ts-rs` traits, making JSON naming and optional fields
      explicit; use string wire representations for identifiers and timestamps.
- [x] Add an exporter binary or example that calls `ts-rs` programmatically and
      writes all bindings plus a barrel file to `webapp/src/api/generated/`.
- [x] Add a repository script for export and a check mode that fails when
      committed bindings are stale or nondeterministic.
- [x] Mark generated files clearly and document that they must not be edited by
      hand.

## Done when

- One documented command regenerates the complete TypeScript contract.
- Re-running the exporter without Rust type changes produces no diff.
- The backend serializes the same types that the webapp imports.

## Notes

`ts-rs` 12 supports programmatic `TS::export_all`/`TS::export` as well as
test-driven export. A dedicated exporter keeps ordinary tests free of file
system side effects and provides one deliberate synchronization command.

