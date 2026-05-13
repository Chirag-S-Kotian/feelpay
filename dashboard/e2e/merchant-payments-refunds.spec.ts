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
  await page.getByText('pay_seed_success').click();
  await expect(page.getByRole('heading', { name: /Refund/i })).toBeVisible();
  await page.locator('.refund-field textarea').fill('Duplicate order');
  await page.locator('.refund-field input').fill('10.00');
  await page.getByRole('button', { name: /Submit Refund/i }).click();
  await expect(page.getByText(/Refund submitted for ₹10.00/i)).toBeVisible();
});

test('refunds module lists real refund records and opens detail drawer', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/refunds');

  await expect(page.getByRole('heading', { name: 'Refunds' })).toBeVisible();
  await expect(page.getByText('ref_seed_1')).toBeVisible();
  await expect(page.getByText('order_seed_1')).toBeVisible();
  await page.getByText('ref_seed_1').click();
  await expect(page.getByText('Customer requested refund')).toBeVisible();
  await expect(page.getByText('gw_refund_seed_1')).toBeVisible();
});
