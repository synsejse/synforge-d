import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";

// Avoid pulling in @types/node here just for `process`; the config runs
// under Node at build time, so `process.cwd()` is always available.
declare const process: { cwd(): string };

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), ["VITE_", "PUBLIC_"]);
  const proxyTarget =
    env.VITE_DEV_PROXY_TARGET || env.PUBLIC_API_URL || "http://localhost:8080";
  return {
    envPrefix: ["VITE_", "PUBLIC_"],
    plugins: [
      TanStackRouterVite({ target: "react", autoCodeSplitting: true }),
      react(),
      tailwindcss(),
    ],
    server: {
      proxy: {
        "/api": { target: proxyTarget, changeOrigin: true },
        "/repo": { target: proxyTarget, changeOrigin: true },
      },
    },
  };
});
