import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactNative from "eslint-plugin-react-native";
import reactHooks from "eslint-plugin-react-hooks";

const eslintConfig = defineConfig([
  globalIgnores([
    "node_modules/**",
    ".expo/**",
    "dist/**",
    "build/**",
    "web-build/**",
    "scripts/**",
    "metro.config.js",
    "**/__tests__/**",
  ]),
  ...tseslint.configs.recommended,
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
    plugins: {
      react,
      "react-native": reactNative,
      "react-hooks": reactHooks,
    },
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    rules: {
      "no-restricted-syntax": "off",
      "react/display-name": "off",
      "react/jsx-no-comment-textnodes": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      // Honour the leading-underscore convention already used in the codebase
      // for deliberately-discarded bindings — e.g. ThemeContext's
      // `const { light: _light, dark: _dark, ...palette } = Colors`, which
      // destructures purely to omit those keys. Without this the rule flags
      // the omission itself as an unused variable.
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          ignoreRestSiblings: true,
        },
      ],
    },
  },
]);

export default eslintConfig;
