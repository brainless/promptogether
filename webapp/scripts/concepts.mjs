// Shared build-time reader for the Vite virtual module and static HTML pages.
// Only the derived content enters dist/; the SQLite file stays in the repo.
import fs from "node:fs";
import { DatabaseSync } from "node:sqlite";

export function loadConcepts(dbPath, posts) {
  if (!fs.existsSync(dbPath)) {
    throw new Error(`Concept database missing at ${dbPath}`);
  }
  const bySlug = new Map(posts.map((post) => [post.slug, post]));
  const db = new DatabaseSync(dbPath, { readOnly: true });
  try {
    const rows = db.prepare(`
      SELECT c.slug, c.name, c.definition, c.url, cp.post_slug
      FROM concepts c LEFT JOIN concept_posts cp ON cp.concept_slug = c.slug
      ORDER BY c.name COLLATE NOCASE, cp.post_slug
    `).all();
    const concepts = [];
    for (const row of rows) {
      if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(row.slug)) {
        throw new Error(`Invalid concept slug: ${row.slug}`);
      }
      if (row.url && !/^https:\/\//.test(row.url)) {
        throw new Error(`Concept ${row.slug} needs an https URL`);
      }
      let concept = concepts.find((item) => item.slug === row.slug);
      if (!concept) {
        concept = { slug: row.slug, name: row.name, definition: row.definition, url: row.url, aliases: [], posts: [] };
        concepts.push(concept);
      }
      if (row.post_slug) {
        const post = bySlug.get(row.post_slug);
        if (!post) throw new Error(`Concept ${row.slug} links to missing post ${row.post_slug}`);
        concept.posts.push({ slug: post.slug, title: post.title, date: post.date });
      }
    }
    for (const concept of concepts) {
      concept.aliases = db.prepare("SELECT alias FROM concept_aliases WHERE concept_slug = ? ORDER BY alias")
        .all(concept.slug).map((row) => row.alias);
      concept.posts.sort((a, b) => b.date.localeCompare(a.date));
    }
    return concepts;
  } finally {
    db.close();
  }
}
