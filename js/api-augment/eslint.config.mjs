// @ts-check

import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import mochaPlugin from "eslint-plugin-mocha";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  mochaPlugin.configs.flat.recommended,
  {
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.eslint.json"],
      },
    },
  },
  {
    rules: {
      "@typescript-eslint/no-empty-interface": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/no-explicit-any": "off",
      semi: ["error", "always"],
      "mocha/no-setup-in-describe": "off",
      "no-use-before-define": "off",
      "no-unused-vars": "off",
      "no-var": "error",
      "id-length": [
        "error",
        {
          exceptionPatterns: ["[i-k]", "e", "c", "x", "y", "r", "s", "v"],
          properties: "never",
        },
      ],
      "allow-namespace": "off",
    },
  },
);
