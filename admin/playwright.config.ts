import { defineConfig, devices } from '@playwright/test';
import dotenv from 'dotenv';
import path from 'path';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
dotenv.config({ path: path.resolve('../.env') });

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  timeout: 60000,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : 1,
  reporter: [['html'], ['list']],
  use: {
    baseURL: process.env.ADMIN_E2E_BASE_URL || 'http://localhost:3000',
    extraHTTPHeaders: {
      'x-admin-api-base-url': process.env.ADMIN_API_BASE_URL || 'http://127.0.0.1:8081',
    },
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'smoke',
      testMatch: /admin-smoke\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'full-admin',
      testMatch: /admin-(all-pages-navigation|features|sandbox-payment-flow|rbac)\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'api',
      testMatch: /admin-api\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'a11y',
      testMatch: /admin-(quality|a11y)\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'visual',
      testMatch: /admin-visual\.spec\.ts/,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
