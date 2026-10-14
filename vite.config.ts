import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  build: { target: "safari17" },
  clearScreen: false,
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  test: { environment: "jsdom" },
});
