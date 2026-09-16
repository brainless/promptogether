# Epic 06: Prompt gallery webapp

## Outcome

A visitor can browse example prompts and enter a project view that makes the
agent's result understandable, without registering or knowing repository tools.

## Tasks

- [ ] Add a gallery entry point from the homepage and shareable gallery project
      URLs.
- [ ] Configure Vite to proxy `/api` locally and add a small typed fetch layer
      using the generated `api-types` output.
- [ ] Build a mobile-first gallery shell: prompt list first on narrow screens
      and a two-column prompt sidebar plus project pane on wider screens.
- [ ] Load summaries separately from the selected project detail, with visible
      loading, empty, retryable error, and not-found states.
- [ ] Show the full prompt and a navigable file list, clearly separating the
      initial project state from the agent result.
- [ ] Render file contents readably with paths, language labels, preserved
      whitespace, horizontal overflow, and copy controls; treat content as text.
- [ ] Support keyboard navigation, focus movement after project selection,
      semantic landmarks, and an accessible selected state.
- [ ] Keep the experience useful without JavaScript-only hover behavior and
      preserve the existing minimal visual language.

## Done when

- Selecting a prompt updates the URL and project pane, and a copied URL opens
  the same project.
- The layout works as a single-column flow on small screens and two columns on
  larger screens.
- All visible data comes from the Rust API and is typed through generated
  declarations; no gallery fixture remains in the webapp.

