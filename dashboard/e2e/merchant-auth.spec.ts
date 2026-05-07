import { expect, test } from '@playwright/test';
import { installMerchantApi, loginMerchant, seedMerchantSession } from './merchant-fixtures';

test.beforeEach(async ({ page }) => {
  await installMerchantApi(page);
});

test('valid token loads dashboard', async ({ page }) => {
  await loginMerchant(page);
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByText('Total Volume')).toBeVisible();
});

test('logout redirects and blocks protected dashboard', async ({ page }) => {
  await loginMerchant(page);
  await page.getByRole('button', { name: /logout/i }).click();
  await expect(page).toHaveURL(/\/login\?reason=logout/);
  await page.goto('/');
  await expect(page).toHaveURL(/\/login\?reason=invalid/);
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toHaveCount(0);
});

test('refresh without token redirects to login', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveURL(/\/login\?reason=invalid/);
  await expect(page.getByText('Session expired. Please sign in again.')).toBeVisible();
});

test('protected API 401 clears session and does not render stale dashboard', async ({ page }) => {
  await seedMerchantSession(page);
  await page.route('**/{api/,}v1/analytics/summary', async (route) => {
    await route.fulfill({ status: 401, contentType: 'application/json', body: JSON.stringify({ message: 'Authentication required. Provide valid x-api-key or Bearer token' }) });
  });
  await page.goto('/');
  await expect(page).toHaveURL(/\/login\?reason=invalid/);
  await expect(page.getByText('Authentication required')).toHaveCount(0);
});
