# Epic 06: Prompt gallery webapp

## Outcome

A visitor can browse example prompts and open a shareable project view that
makes the agent's result understandable, without registering or knowing
repository tools. The existing homepage remains intact and gains a clear entry
point to the gallery.

## Existing foundation

- The webapp uses SolidJS 2.0, TypeScript, Vite, and component-scoped CSS. Use
  Solid 2.0 conventions and continue rendering through `@solidjs/web`.
- Use Solid 2.0 async computations with narrowly scoped `Loading` and `Errored`
  boundaries for API-backed views; do not introduce the legacy 1.x
  `createResource` model.
- Rust wire types are generated into `webapp/src/api/generated/`. Import from
  the generated barrel and do not copy, loosen, or hand-edit those types.
- Before adding a routing or highlighting dependency, check compatibility with
  the exact versions in `webapp/package.json` and its lockfile. Keep the first
  gallery implementation small; use either a compatible focused router or the
  History API with explicit cleanup and link semantics.

## Tasks

- [x] Define canonical routes for the gallery index and project detail (for
      example `/gallery` and `/gallery/:slug`) and add a gallery entry point to
      the homepage. Direct navigation, refresh, same-origin links, and browser
      back/forward must all derive selection from the URL; document the
      production SPA-fallback requirement.
- [x] Configure Vite to proxy `/api` to a configurable local backend origin,
      with a documented default. Keep browser requests relative so development
      and same-origin production use the same code path.
- [x] Add a small typed API client that imports generated response types,
      checks `response.ok`, parses the shared `ErrorResponse`, distinguishes
      not-found from other failures, and handles invalid or non-JSON responses
      without presenting raw server internals to visitors.
- [x] Build a mobile-first gallery shell: prompt list first on narrow screens,
      and a two-column prompt navigation plus project pane on wider screens.
      Preserve the homepage's minimal visual language rather than replacing
      its existing sections or global styles wholesale.
- [x] Fetch summaries independently from selected-project detail. Prevent a
      slow earlier request from replacing a newer selection, and avoid
      refetching the unchanged summary list on every project navigation.
- [x] Provide distinct, visible states for initial loading, empty gallery,
      retryable list failure, detail loading, retryable detail failure, and
      project not found. A bad detail URL must not make the whole gallery
      unusable, and retry controls must repeat the failed operation.
- [x] Show the complete prompt and a navigable file list, clearly separating
      the initial project from the agent result. Give empty initial states an
      explicit explanation rather than rendering an unexplained blank area.
- [x] Render file paths, language labels, and contents readably with preserved
      whitespace and horizontal overflow. Treat all API content as text; do not
      inject HTML. Implement copy controls with success/failure feedback and a
      usable fallback when the Clipboard API is unavailable.
- [x] Use semantic landmarks and headings, label the project navigation,
      expose the selected item with `aria-current`, keep all controls keyboard
      reachable, and move focus to the project heading after an intentional
      in-app selection without stealing focus on initial load or browser
      history navigation. Announce asynchronous errors and copy feedback.
- [x] Keep essential information and controls independent of hover, respect
      reduced-motion preferences, and preserve readable focus, contrast, and
      touch-target sizes at narrow and wide breakpoints.
- [x] Document local startup (`bootstrap`/`seed-gallery`, backend, then Vite),
      the proxy override, canonical URLs, and any production fallback needed
      for direct gallery links.

## Done when

- Selecting a prompt updates the canonical URL, back/forward restores the
  corresponding selection, and opening or refreshing a copied project URL
  loads the same project when the host serves the documented SPA fallback.
- The layout is a coherent single-column flow on small screens and a
  two-column experience on wider screens, including loading, empty, error, and
  not-found cases.
- All visible gallery data comes from the Rust API and is typed through the
  generated declarations; no gallery fixture or duplicated API interface
  remains in the webapp.
- The gallery can be explored with a keyboard and common screen-reader
  navigation without registration.

## Scope boundary

This epic is a read-only gallery. Client-side code editing, diffs, syntax
highlighting, running projects, authentication, and invoking a coding agent are
future work.
