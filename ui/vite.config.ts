/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { fileURLToPath } from "node:url";

// Tauri expects a fixed dev port and must not have its output cleared.
const DEV_PORT = 1420;

export default defineConfig({
  root: fileURLToPath(new URL(".", import.meta.url)),
  plugins: [solid()],
  clearScreen: false,
  resolve: { alias: { "~": fileURLToPath(new URL("./src", import.meta.url)) } },
  server: { port: DEV_PORT, strictPort: true, watch: { ignored: ["**/app/**", "**/crates/**", "**/target/**"] } },
  build: {
    target: "chrome120",
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        overlay: fileURLToPath(new URL("./overlay.html", import.meta.url)),
      },
    },
  },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
