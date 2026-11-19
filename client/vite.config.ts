import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The dev server proxies the API to the local Mnemo server (127.0.0.1:3000).
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": { target: "http://127.0.0.1:3000", changeOrigin: true },
      "/mcp": { target: "http://127.0.0.1:3000", changeOrigin: true },
    },
  },
});
