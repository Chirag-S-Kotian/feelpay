import { defineConfig, devices } from '@playwright/test';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
const adminDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(adminDir, '..');

dotenv.config({ path: path.resolve(repoRoot, '.env') });

const frontendUrl = process.env.ADMIN_E2E_BASE_URL || 'http://127.0.0.1:3000';
const backendUrl = process.env.ADMIN_API_BASE_URL || 'http://127.0.0.1:8081';
const backendPort = new URL(backendUrl).port || '8081';
const backendHost = new URL(backendUrl).hostname || '127.0.0.1';
const corsAllowedOrigins = [
  frontendUrl,
  'http://localhost:3000',
  'http://127.0.0.1:3000',
].join(',');

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  timeout: 60000,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : 1,
  reporter: [['html'], ['list']],
  use: {
    baseURL: frontendUrl,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  webServer: [
    {
      command: 'cargo run',
      cwd: repoRoot,
      url: `${backendUrl}/health`,
      reuseExistingServer: true,
      timeout: 120000,
      env: {
        ADMIN_HOST: backendHost,
        ADMIN_PORT: backendPort,
        HOST: backendHost,
        PORT: backendPort,
        ADMIN_CORS_ALLOWED_ORIGINS: corsAllowedOrigins,
        CORS_ALLOWED_ORIGINS: corsAllowedOrigins,
        ADMIN_API_BASE_URL: backendUrl,
        JWT_SECRET: process.env.JWT_SECRET || 'feelpay_jwt_secret',
        VAULT_ENCRYPTION_KEY: process.env.VAULT_ENCRYPTION_KEY || '0000000000000000000000000000000000000000000000000000000000000000',
        DATABASE_URL: process.env.DATABASE_URL || '',
        RUST_LOG: process.env.RUST_LOG || 'info',
      },
    },
    {
      command: 'pnpm dev --host 127.0.0.1 --port 3000',
      cwd: adminDir,
      url: frontendUrl,
      reuseExistingServer: true,
      timeout: 120000,
      env: {
        ADMIN_E2E_BASE_URL: frontendUrl,
        ADMIN_API_BASE_URL: backendUrl,
        VITE_ADMIN_API_BASE_URL: backendUrl,
      },
    },
  ],
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
