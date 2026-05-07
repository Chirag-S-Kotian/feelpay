import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('merchant pages remain usable on mobile without horizontal overflow', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.setViewportSize({ width: 375, height: 812 });

  for (const path of ['/', '/payments', '/checkout-preview', '/payment-links', '/settings', '/onboarding']) {
    await page.goto(path);
    await expect(page.locator('h1').first()).toBeVisible();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 2);
    expect(overflow, `${path} should not horizontally overflow`).toBe(false);
  }
});
