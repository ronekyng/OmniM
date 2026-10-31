import { defineConfig, devices } from '@playwright/test';

const PROJECTS_DIR = '/tmp/opencode/omnim-e2e';

export default defineConfig({
  testDir: './e2e',
  globalSetup: './e2e/global-setup.ts',
  timeout: 45_000,
  expect: { timeout: 8_000 },
  fullyParallel: false,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:5174',
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: [
    {
      command: 'cargo run --quiet --bin omnim',
      cwd: '..',
      url: 'http://127.0.0.1:3100/api/health',
      // Never reuse a developer's server: it may point at the repo root and
      // write sample projects into the working tree.
      reuseExistingServer: false,
      timeout: 240_000,
      env: {
        OMNIM_PORT: '3100',
        OMNIM_EPHEMERAL: '1',
        OMNIM_PROJECTS_DIR: PROJECTS_DIR,
      },
    },
    {
      command: 'npm run dev -- --host 127.0.0.1 --port 5174',
      url: 'http://127.0.0.1:5174',
      reuseExistingServer: false,
      timeout: 90_000,
      env: {
        OMNIM_API_URL: 'http://127.0.0.1:3100',
      },
    },
  ],
});
