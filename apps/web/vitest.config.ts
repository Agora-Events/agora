import { defineConfig } from 'vitest/config';
import path from 'path';

// Force test environment — NODE_ENV=production (inherited from the shell)
// loads React production builds where React.act is stripped, crashing
// @testing-library/react with "React.act is not a function".
process.env.NODE_ENV = 'test';

export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./vitest.setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['components/**/*.{ts,tsx}'],
      exclude: [
        'node_modules/',
        '__tests__/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/mockups.ts',
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './'),
    },
  },
  esbuild: {
    jsx: 'automatic',
  },
});
