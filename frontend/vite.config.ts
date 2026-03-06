import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = resolve(fileURLToPath(import.meta.url), "..");

export default defineConfig(({ mode }) => {
  // Load all env vars (incl. without VITE_ prefix) for proxy config
  const env = loadEnv(mode, __dirname, "");

  return {
    plugins: [react()],
    resolve: {
      alias: {
        "@": resolve(__dirname, "src"),
      },
      preserveSymlinks: true,
    },
    server: {
      port: 5173,
      proxy: {
        // Report generation → OpenAEC Reports API
        // Bearer token van de gebruiker wordt doorgegeven.
        // REPORTS_API_KEY is optionele fallback totdat OIDC volledig werkt.
        "/api/report": {
          target: env.REPORTS_API_URL || "https://report.open-aec.com",
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api\/report\/generate/, "/api/generate/v2"),
          configure: (proxy) => {
            const apiKey = env.REPORTS_API_KEY || "";
            if (apiKey) {
              proxy.on("proxyReq", (proxyReq) => {
                proxyReq.setHeader("X-API-Key", apiKey);
              });
            }
          },
        },
        // Other API calls → backend
        "/api": {
          target: "http://localhost:3001",
        },
      },
    },
    envPrefix: ["VITE_", "TAURI_"],
  };
});
