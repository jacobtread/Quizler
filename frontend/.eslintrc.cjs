/* eslint-env node */
module.exports = {
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:svelte/recommended"
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    extraFileExtensions: [".svelte"]
  },
  plugins: ["@typescript-eslint"],
  root: true,
  overrides: [
    {
      files: ["*.svelte"],
      parser: "svelte-eslint-parser",
      // Parse the `<script>` in `.svelte` as TypeScript by adding the following configuration.
      parserOptions: {
        parser: "@typescript-eslint/parser"
      }
    }
  ],
  env: {
    browser: true
  },
  rules: {
    "@typescript-eslint/no-inferrable-types": "off"
  }
};
