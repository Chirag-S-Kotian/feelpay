import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('reorder, disable, save, and preview payment methods', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/payment-methods');

  await expect(page.getByRole('heading', { name: 'Payment Methods' })).toBeVisible();
  const cards = page.locator('article.method-card').filter({ hasText: 'Cards' });
  await cards.getByRole('checkbox').uncheck();
  await cards.getByRole('button', { name: /save method/i }).click();
  await expect(page.getByText(/Cards settings saved/i)).toBeVisible();

  const wallets = page.locator('article.method-card').filter({ hasText: 'Wallets' });
  await wallets.getByRole('spinbutton').fill('1');
  await wallets.getByRole('button', { name: /save method/i }).click();
  await expect(page.getByText(/Wallets settings saved/i)).toBeVisible();

  await page.goto('/checkout-preview');
  await expect(page.getByText(/Visible methods from merchant settings/i)).toBeVisible();
  await expect(page.locator('.preview-method-list button').first()).toHaveText(/Wallets/i);
  await expect(page.locator('.preview-method-list')).not.toContainText('Cards');
});
