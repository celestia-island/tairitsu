import tseslint from "typescript-eslint";
import js from "@eslint/js";

export default tseslint.config(
  {
    ignores: [
      "src/glue/**",
      "dist/**",
      "node_modules/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/runtime/**/*.ts", "src/async.ts", "src/handles.ts", "src/index.ts"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/ban-ts-comment": "warn",
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-non-null-assertion": "off",
      "no-empty": ["error", { allowEmptyCatch: true }],
    },
  },
);
