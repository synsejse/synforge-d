import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  // Generated output and build artifacts are never linted.
  {
    ignores: ["dist", "src/generated/**", "src/routeTree.gen.ts"],
  },
  {
    files: ["**/*.{ts,tsx}"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      // Opinionated React-Compiler-era rule (plugin v7) that flags the
      // common "initialize state from loaded data" effect pattern. Left off
      // for now to keep a clean baseline; adopt gradually if desired.
      "react-hooks/set-state-in-effect": "off",
    },
  },
  // TanStack Router file routes must export `Route` alongside the component,
  // which the react-refresh heuristic can't model. Not a real fast-refresh
  // hazard here, so silence it for route files only.
  {
    files: ["src/routes/**/*.{ts,tsx}"],
    rules: {
      "react-refresh/only-export-components": "off",
    },
  },
  // Config files run in Node, not the browser.
  {
    files: ["*.{js,ts}", "vite.config.ts"],
    languageOptions: {
      globals: globals.node,
    },
  },
);
