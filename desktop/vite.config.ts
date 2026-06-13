import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri очікує фіксований порт і ігнорує src-tauri зі watcher'а
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "chrome105",
    minify: "esbuild",
    sourcemap: false,
  },
});