import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:3001',
    trace: 'on-first-retry',
    launchOptions: {
      args: ['--no-sandbox', '--disable-gpu', '--use-gl=swiftshader'],
    },
  },
  projects: [
    {
      name: 'chromium',
      use: {
        browserType: 'chromium',
      },
    },
  ],
});
