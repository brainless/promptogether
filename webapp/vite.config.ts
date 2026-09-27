import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import solid from "@solidjs/vite-plugin";
import { loadPosts } from "./scripts/posts.mjs";
import { linkConcepts } from "./scripts/concept-links.mjs";
import { loadConcepts } from "./scripts/concepts.mjs";

declare const process: { env: Record<string, string | undefined> };
const apiOrigin = process.env.VITE_API_ORIGIN || "http://localhost:3000";

const contentDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "content/posts");
const conceptsDb = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "content/concepts.sqlite");

/**
 * Exposes Markdown posts and SQLite concepts to app code as virtual modules.
 * Both are loaded during dev/build; editing a post or the database reloads
 * the modules in dev.
 */
function contentPlugin(): Plugin {
  const postsId = "\0virtual:posts";
  const conceptsId = "\0virtual:concepts";

  return {
    name: "site-content",
    resolveId(id) {
      if (id === "virtual:posts") return postsId;
      if (id === "virtual:concepts") return conceptsId;
    },
    load(id) {
      if (id === postsId || id === conceptsId) {
        const posts = loadPosts(contentDir);
        const concepts = loadConcepts(conceptsDb, posts);
        if (id === conceptsId) return `export const concepts = ${JSON.stringify(concepts)};`;
        const linkedPosts = posts.map((post) => ({
          ...post,
          html: linkConcepts(post.html, concepts.filter((concept) =>
            concept.posts.some((related) => related.slug === post.slug))),
        }));
        return `export const posts = ${JSON.stringify(linkedPosts)};`;
      }
    },
    configureServer(server) {
      server.watcher.add([contentDir, conceptsDb]);
      server.watcher.on("all", (_event, file) => {
        const changed = path.resolve(file);
        if (changed.startsWith(contentDir + path.sep) || changed === conceptsDb) {
          for (const id of [postsId, conceptsId]) {
            const module = server.moduleGraph.getModuleById(id);
            if (module) server.moduleGraph.invalidateModule(module);
          }
          server.ws.send({ type: "full-reload" });
        }
      });
    },
  };
}

export default defineConfig({
  plugins: [solid(), contentPlugin()],
  server: {
    proxy: {
      "/api": {
        target: apiOrigin,
        changeOrigin: true,
      },
    },
  },
});
