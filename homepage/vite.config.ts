import { defineConfig } from "vite";
import eslintPlugin from "@nabla/vite-plugin-eslint";

import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vitejs.dev/config/
export default defineConfig({
  // Refer https://vitejs.dev/guide/static-deploy#github-pages
  base: "/marc/",
  plugins: [react(), wasm(), topLevelAwait(), eslintPlugin()],
  assetsInclude: ["**/*.md"],
});
