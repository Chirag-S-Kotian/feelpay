import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('success, failed, pending filters and payment details surface load', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/payments');

  await expect(page.getByRole('heading', { name: 'Payments' })).toBeVisible();
  await expect(page.getByText('pay_seed_success')).toBeVisible();
  await page.getByRole('button', { name: /captured/i }).click();
  await expect(page.getByText(/captured/i).first()).toBeVisible();
  await page.getByPlaceholder(/Search by ID/i).fill('order_seed_1');
  await expect(page.getByText('order_seed_1')).toBeVisible();
});
