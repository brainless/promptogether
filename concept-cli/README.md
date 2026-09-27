# Concept CLI

This internal CLI searches `webapp/content/posts/*.md` for a named concept, sends representative excerpts to Xiaomi's MiMo V2.6 Flash through `llm-sdk`, and saves a short definition plus linked post slugs to the committed `webapp/content/concepts.sqlite` database. It does not edit transcript Markdown or publish to Cloudflare.

From the repository root:

```sh
export XIAOMI_API_KEY="..."   # or put it in the repository-root .env
cargo run -p concept-cli -- "Claude Code" --overwrite
cd webapp && npm run build
```

The CLI requires the committed SQLite file. MiMo V2.6 Flash is the default model; use `--model MODEL_ID` to select another Xiaomi chat model supported by `llm-sdk`. The CLI refuses terms absent from post bodies and existing concepts unless `--overwrite` is given. `--overwrite` updates the definition and post links but keeps the concept's slug, URL, and aliases. A failed or incomplete LLM response is not saved. Review the generated definition and related posts before committing the database. The webapp build validates post links and generates the static sidebar and Concepts page directly from SQLite. Never commit the API key.
