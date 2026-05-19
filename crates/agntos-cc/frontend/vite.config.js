import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    svelte(),
    {
      name: "inline-css",
      enforce: "post",
      closeBundle() {
        const outDir = path.resolve(__dirname, "dist");
        const htmlPath = path.resolve(outDir, "index.html");
        if (!fs.existsSync(htmlPath)) return;

        let html = fs.readFileSync(htmlPath, "utf-8");
        const assetsDir = path.resolve(outDir, "assets");

        html = html.replace(
          /<link rel="stylesheet"[^>]*href="([^"]+\.css)"[^>]*>/g,
          (_, href) => {
            const cssPath = path.resolve(outDir, href.replace(/^\//, ""));
            if (fs.existsSync(cssPath)) {
              const css = fs.readFileSync(cssPath, "utf-8");
              fs.unlinkSync(cssPath);
              return `<style>${css}</style>`;
            }
            return _;
          },
        );

        fs.writeFileSync(htmlPath, html);
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