// Runs after prerender-posts.mjs (see the "build" script in package.json).
// Emits dist/sitemap.xml listing the homepage, /posts, every post, and
// /concepts. Gallery routes are intentionally left out until that content
// is ready and linked from the site.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadPosts } from "./posts.mjs";

const rootDir = path.dirname(fileURLToPath(import.meta.url)) + "/..";
const distDir = path.resolve(rootDir, "dist");
const contentDir = path.resolve(rootDir, "content/posts");
const siteUrl = (process.env.SITE_URL || "https://promptogether.com").replace(/\/$/, "");

function isoDate(value) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString().slice(0, 10);
}

function urlEntry(loc, lastmod) {
  return `  <url>\n    <loc>${loc}</loc>\n${lastmod ? `    <lastmod>${lastmod}</lastmod>\n` : ""}  </url>`;
}

function main() {
  if (!fs.existsSync(distDir)) {
    throw new Error(`dist/ not found at ${distDir} — run "vite build" before this script`);
  }

  const posts = loadPosts(contentDir);
  const latestPostDate = posts.reduce((latest, post) => {
    const d = isoDate(post.date);
    return d && (!latest || d > latest) ? d : latest;
  }, undefined);

  const entries = [
    urlEntry(`${siteUrl}/`, latestPostDate),
    urlEntry(`${siteUrl}/posts`, latestPostDate),
    ...posts.map((post) => urlEntry(`${siteUrl}/posts/${post.slug}`, isoDate(post.date))),
    urlEntry(`${siteUrl}/concepts`, latestPostDate),
  ];

  const xml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${entries.join("\n")}\n</urlset>\n`;

  fs.writeFileSync(path.join(distDir, "sitemap.xml"), xml);
  console.log(`Wrote sitemap.xml with ${entries.length} URL(s)`);
}

main();
