import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, "./src/lib"),
      $pages: path.resolve(__dirname, "./src/lib/pages"),
      $stores: path.resolve(__dirname, "./src/lib/stores"),
      $components: path.resolve(__dirname, "./src/lib/components"),
      $assets: path.resolve(__dirname, "./src/lib/assets"),
      $api: path.resolve(__dirname, "./src/lib/api")
    }
  }
});
