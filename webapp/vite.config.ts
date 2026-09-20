import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import solid from "@solidjs/vite-plugin";
import { loadPosts } from "./scripts/posts.mjs";

declare const process: { env: Record<string, string | undefined> };
const apiOrigin = process.env.VITE_API_ORIGIN || "http://localhost:3000";

const contentDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "content/posts");

/**
 * Exposes the parsed Markdown posts under `content/posts/` to app code as
 * `import { posts } from "virtual:posts"`. Posts are parsed once at
 * dev-server start / build time; editing a post file triggers a full page
 * reload in dev so the virtual module is re-evaluated.
 */
function postsPlugin(): Plugin {
  const virtualModuleId = "virtual:posts";
  const resolvedVirtualModuleId = "\0" + virtualModuleId;

  return {
    name: "posts-content",
    resolveId(id) {
      if (id === virtualModuleId) return resolvedVirtualModuleId;
    },
    load(id) {
      if (id === resolvedVirtualModuleId) {
        const posts = loadPosts(contentDir);
        return `export const posts = ${JSON.stringify(posts)};`;
      }
    },
    configureServer(server) {
      server.watcher.add(contentDir);
      server.watcher.on("all", (_event, file) => {
        if (path.resolve(file).startsWith(contentDir)) {
          server.ws.send({ type: "full-reload" });
        }
      });
    },
  };
}

export default defineConfig({
  plugins: [solid(), postsPlugin()],
  server: {
    proxy: {
      "/api": {
        target: apiOrigin,
        changeOrigin: true,
      },
    },
  },
});
