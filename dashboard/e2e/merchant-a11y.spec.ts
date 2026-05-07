import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('core merchant pages have no critical accessibility violations or console errors', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  const consoleErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });

  for (const path of ['/', '/payments', '/checkout-preview', '/webhooks']) {
    await page.goto(path);
    await expect(page.locator('h1').first()).toBeVisible();
    const scan = await new AxeBuilder({ page }).analyze();
    expect(scan.violations.filter((violation) => violation.impact === 'critical')).toEqual([]);
  }

  expect(consoleErrors).toEqual([]);
});
