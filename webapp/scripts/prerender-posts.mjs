// Runs after `vite build` (see the "build" script in package.json). Vite
// itself only ever emits one `dist/index.html` — the app's routing is
// entirely client-side — so this script generates a static HTML file per
// `/posts` route on top of that build: one per post plus the `/posts`
// index. Each gets its own <title>/description/Open Graph tags (crucial
// for search engines and link-preview crawlers, which don't run the SPA's
// JS) and a plain-HTML snapshot of the post inside <div id="root">, which
// Solid replaces with the live app once it mounts. The homepage and
// gallery are unaffected: they stay a single client-rendered index.html.
//
// Static hosting requirement: this relies on the host serving
// `/posts/<slug>/index.html` for a request to `/posts/<slug>` (and
// `/posts/index.html` for `/posts`) — Cloudflare Pages does this
// automatically for directory-style static output.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadPosts } from "./posts.mjs";
import { extractYouTubeId } from "./youtube.mjs";

const rootDir = path.dirname(fileURLToPath(import.meta.url)) + "/..";
const distDir = path.resolve(rootDir, "dist");
const contentDir = path.resolve(rootDir, "content/posts");
const siteUrl = (process.env.SITE_URL || "https://promptogether.com").replace(/\/$/, "");

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function youtubeEmbedHtml(youtubeUrl, title) {
  const videoId = extractYouTubeId(youtubeUrl);
  if (!videoId) return "";
  return `<div class="pt-video"><iframe src="https://www.youtube-nocookie.com/embed/${escapeHtml(videoId)}" title="${escapeHtml(title)}" loading="lazy" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe></div>`;
}

function headerHtml() {
  return `<header class="pt-header">
    <a class="pt-brand" href="/">prompt<span>ogether</span>.</a>
    <nav class="pt-nav" aria-label="Main navigation">
      <a href="/posts">Posts</a>
      <a href="/gallery">Gallery</a>
    </nav>
  </header>`;
}

function footerHtml() {
  return `<footer class="pt-footer"><span>Prompt Together</span><span>Build what you need.</span></footer>`;
}

function formatDate(iso) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
}

function renderHead(baseHead, { title, description, path: pagePath, ogType, youtubeUrl }) {
  let head = baseHead
    .replace(/<title>.*?<\/title>/s, `<title>${escapeHtml(title)}</title>`)
    .replace(
      /<meta name="description" content="[^"]*"\s*\/?>/,
      `<meta name="description" content="${escapeHtml(description)}" />`,
    );

  const url = `${siteUrl}${pagePath}`;
  const ogTags = [
    `<link rel="canonical" href="${escapeHtml(url)}">`,
    `<meta property="og:type" content="${ogType}">`,
    `<meta property="og:title" content="${escapeHtml(title)}">`,
    `<meta property="og:description" content="${escapeHtml(description)}">`,
    `<meta property="og:url" content="${escapeHtml(url)}">`,
    `<meta name="twitter:card" content="${youtubeUrl ? "player" : "summary"}">`,
    `<meta name="twitter:title" content="${escapeHtml(title)}">`,
    `<meta name="twitter:description" content="${escapeHtml(description)}">`,
  ];
  if (youtubeUrl) {
    const videoId = extractYouTubeId(youtubeUrl);
    if (videoId) {
      const embedUrl = `https://www.youtube-nocookie.com/embed/${videoId}`;
      ogTags.push(
        `<meta property="og:video" content="${escapeHtml(embedUrl)}">`,
        `<meta name="twitter:player" content="${escapeHtml(embedUrl)}">`,
      );
    }
  }
  ogTags.push('<link rel="stylesheet" href="/posts-static.css">');

  return head.replace("</head>", `  ${ogTags.join("\n  ")}\n  </head>`);
}

function writePage(relativeOutPath, baseTemplate, headOptions, bodyHtml) {
  const [head, rest] = baseTemplate.split("</head>");
  const renderedHead = renderHead(`${head}</head>`, headOptions);
  const page = `${renderedHead}${rest.replace('<div id="root"></div>', `<div id="root">${bodyHtml}</div>`)}`;

  const outPath = path.join(distDir, relativeOutPath);
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, page);
}

function main() {
  if (!fs.existsSync(distDir)) {
    throw new Error(`dist/ not found at ${distDir} — run "vite build" before this script`);
  }

  const baseTemplate = fs.readFileSync(path.join(distDir, "index.html"), "utf-8");
  const posts = loadPosts(contentDir);

  for (const post of posts) {
    const bodyHtml = `${headerHtml()}<main class="pt-page pt-main">
      <a class="pt-back" href="/posts">&larr; All posts</a>
      <p class="pt-date">${escapeHtml(formatDate(post.date))}</p>
      <h1 class="pt-title">${escapeHtml(post.title)}</h1>
      ${post.youtubeUrl ? youtubeEmbedHtml(post.youtubeUrl, post.title) : ""}
      <div class="pt-body">${post.html}</div>
    </main>${footerHtml()}`;

    writePage(
      `posts/${post.slug}/index.html`,
      baseTemplate,
      {
        title: `${post.title} — Prompt Together`,
        description: post.description,
        path: `/posts/${post.slug}`,
        ogType: "article",
        youtubeUrl: post.youtubeUrl,
      },
      bodyHtml,
    );
  }

  const listItems = posts
    .map(
      (post) => `<li><a href="/posts/${post.slug}">
        <span class="pt-date">${escapeHtml(formatDate(post.date))}</span>
        <h2>${escapeHtml(post.title)}</h2>
        <p>${escapeHtml(post.description)}</p>
      </a></li>`,
    )
    .join("\n");

  const indexBodyHtml = `${headerHtml()}<main class="pt-page pt-main">
    <h1 class="pt-title">Posts</h1>
    <ul class="pt-list">${listItems || "<li>No posts yet. Check back soon.</li>"}</ul>
  </main>${footerHtml()}`;

  writePage(
    "posts/index.html",
    baseTemplate,
    {
      title: "Posts — Prompt Together",
      description: "Notes, updates, and videos from building Prompt Together.",
      path: "/posts",
      ogType: "website",
      youtubeUrl: null,
    },
    indexBodyHtml,
  );

  console.log(`Prerendered ${posts.length} post page(s) and the /posts index into dist/posts/`);
}

main();
