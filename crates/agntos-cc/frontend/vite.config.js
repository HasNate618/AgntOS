import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    tailwindcss(),
    svelte(),
    {
      name: "remove-css-crossorigin",
      transformIndexHtml(html) {
        return html.replace(
          /<link rel="stylesheet"[^>]*crossorigin[^>]*>/g,
          (match) => match.replace(/\s*crossorigin\s*/g, ""),
        );
      },
    },
  ],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 5174,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});