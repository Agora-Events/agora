import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    include: ["**/__tests__/**/*.ts", "**/__tests__/**/*.tsx"],
    coverage: {
      provider: "istanbul",
      reporter: ["text", "json", "html"],
      exclude: [
        "node_modules/",
        "**/*.test.ts",
        "**/*.test.tsx",
        "**/vitest.config.ts",
        "**/vitest.setup.ts",
      ],
    },
  },
});