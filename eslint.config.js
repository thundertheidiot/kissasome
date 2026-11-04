// @ts-check

import { defineConfig } from "eslint/config";
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";

export default defineConfig([
  { ignores: ["**/dist"] },
  {
    extends: [js.configs.recommended, tseslint.configs.recommended],

    files: ["**/*.ts"],

    rules: {
      "@typescript-eslint/no-unused-vars": ["warn"],
    },

    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  eslintConfigPrettier,
]);
