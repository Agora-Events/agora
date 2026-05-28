import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

import react from "eslint-plugin-react";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  // Override default ignores of eslint-config-next.
  globalIgnores([
    // Default ignores of eslint-config-next:
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
  ]),
  {
    plugins: {
      react
    },
    rules: {
      // Disallow inline SVG elements - encourage using pre-saved SVG files from public/icons
      "no-restricted-syntax": [
        "error",
        {
          "selector": "JSXOpeningElement[name.name=\"svg\"]",
          "message": "Inline SVGs are not allowed. Please use pre-saved SVG files from public/icons instead."
        }
      ]
    }
  }
]);

export default eslintConfig;
