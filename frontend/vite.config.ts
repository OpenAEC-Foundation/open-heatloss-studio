import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { oidcSpa } from "oidc-spa/vite-plugin";
import { resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = resolve(fileURLToPath(import.meta.url), "..");

export default defineConfig(({ mode }) => {
  // Load all env vars (incl. without VITE_ prefix) for proxy config
  const env = loadEnv(mode, __dirname, "");

  return {
    plugins: [
      react(),
      oidcSpa({ sessionRestorationMethod: "full page redirect" }),
    ],
    resolve: {
      alias: {
        "@": resolve(__dirname, "src"),
      },
      preserveSymlinks: true,
    },
    server: {
      port: 3023,
      proxy: {
        // Report generation → OpenAEC Reports API
        // Pad matcht /api/v1/report/* zodat het consistent is met de Rust backend.
        // In dev: Vite proxy stuurt direct naar Reports API.
        // In prod: Rust backend proxied met X-API-Key.
        "/api/v1/report": {
          target: env.REPORTS_API_URL || "https://report.open-aec.com",
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api\/v1\/report\/generate/, "/api/generate/v2"),
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
