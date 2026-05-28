import { defineConfig } from "eslint/config";
import react from "eslint-plugin-react";

export default defineConfig([
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