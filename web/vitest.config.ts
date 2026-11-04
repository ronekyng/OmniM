import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // Playwright specs live in e2e/ and must not be collected by Vitest.
    exclude: ['e2e/**', 'node_modules/**', 'dist/**'],
    // Per-file browser env: tests that need DOM (window/document) declare it
    // with the `@vitest-environment happy-dom` header; the default keeps the
    // node env for pure-logic suites.
    environment: 'node',
  },
});
