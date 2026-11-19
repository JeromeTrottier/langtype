import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The dev server proxies the API to the local Mnemo server (127.0.0.1:3000 unless MNEMO_API_URL is set).
const api = process.env.MNEMO_API_URL ?? "http://127.0.0.1:3000";

export default defineConfig({
  plugins: [react()],
  server: {
    // Overridable so several worktrees (e.g. screenshot runs) can run side by side.
    port: Number(process.env.VITE_PORT ?? 5173),
    strictPort: true,
    proxy: {
      "/api": { target: api, changeOrigin: true },
      "/mcp": { target: api, changeOrigin: true },
    },
  },
});
