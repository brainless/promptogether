import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";

declare const process: { env: Record<string, string | undefined> };
const apiOrigin = process.env.VITE_API_ORIGIN || "http://localhost:3000";

export default defineConfig({
  plugins: [solid()],
  server: {
    proxy: {
      "/api": {
        target: apiOrigin,
        changeOrigin: true,
      },
    },
  },
});
