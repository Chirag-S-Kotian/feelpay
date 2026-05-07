import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('checkout settings are readable on mobile and update preview immediately', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/checkout-preview');

  await expect(page.getByRole('heading', { name: 'Checkout Preview' })).toBeVisible();
  await page.getByLabel('Brand color').fill('#236E3E');
  await page.getByLabel('Button text').fill('Pay test order');
  await expect(page.getByLabel('Brand color')).toHaveCSS('min-height', '42px');
  await page.getByRole('button', { name: /mobile/i }).click();
  await page.getByRole('button', { name: /Payment Configuration/i }).click();

  await expect(page.getByRole('button', { name: /Pay test order/i })).toBeVisible();
  await expect(page.locator('.settings-chip-list').first()).toBeVisible();
  const clippedControls = await page.locator('.preview-controls').evaluate((node) => node.scrollWidth > node.clientWidth + 2);
  expect(clippedControls).toBe(false);
  await expect(page.locator('body')).not.toHaveCSS('overflow-x', 'scroll');
});
