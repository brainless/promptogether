// Shared Markdown-post parsing used by both the Vite dev/build virtual
// module (vite.config.ts) and the post-build static HTML generator
// (scripts/prerender-posts.mjs). Plain Node ESM (not TypeScript) because it
// runs standalone in scripts/prerender-posts.mjs, outside Vite's toolchain.

import fs from "node:fs";
import path from "node:path";
import matter from "gray-matter";
import { marked } from "marked";

marked.use({ gfm: true, breaks: false });

/**
 * Reads every `*.md` file in `contentDir`, parses its YAML frontmatter and
 * Markdown body, and returns posts sorted newest-first by `date`.
 *
 * Required frontmatter: `title`, `date` (any string `Date` can parse, e.g.
 * `2026-09-20`). Optional: `description` (falls back to a plain-text
 * excerpt of the body), `youtube_url`, `slug` (falls back to the filename
 * without its `.md` extension).
 */
export function loadPosts(contentDir) {
  if (!fs.existsSync(contentDir)) return [];

  const files = fs.readdirSync(contentDir).filter((file) => file.endsWith(".md"));

  const posts = files.map((file) => {
    const filePath = path.join(contentDir, file);
    const raw = fs.readFileSync(filePath, "utf-8");
    const { data, content } = matter(raw);

    if (!data.title) {
      throw new Error(`Post "${file}" is missing the required frontmatter field "title"`);
    }
    if (!data.date) {
      throw new Error(`Post "${file}" is missing the required frontmatter field "date"`);
    }

    const slug = data.slug ? String(data.slug) : file.replace(/\.md$/, "");
    const body = content.trim();

    return {
      slug,
      title: String(data.title),
      date: String(data.date),
      description: String(data.description || excerpt(body)),
      youtubeUrl: data.youtube_url ? String(data.youtube_url) : null,
      html: marked.parse(body, { async: false }),
    };
  });

  const slugs = new Set();
  for (const post of posts) {
    if (slugs.has(post.slug)) {
      throw new Error(`Duplicate post slug "${post.slug}" — rename one of the source files or set a distinct "slug" in its frontmatter`);
    }
    slugs.add(post.slug);
  }

  posts.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0));
  return posts;
}

function excerpt(markdownBody) {
  const text = markdownBody.replace(/[#*_`>[\]]/g, "").replace(/\s+/g, " ").trim();
  return text.length > 160 ? `${text.slice(0, 157)}...` : text;
}
