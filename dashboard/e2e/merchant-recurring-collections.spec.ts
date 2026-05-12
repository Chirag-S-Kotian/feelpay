import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('recurring billing, mandates, and virtual accounts are wired', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  await page.goto('/subscriptions');
  await expect(page.getByRole('heading', { name: 'Subscriptions', exact: true })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'Gold Monthly' }).first()).toBeVisible();
  await page.getByPlaceholder('Plan name').first().fill('Silver Monthly');
  await page.getByPlaceholder('Amount INR').first().fill('499');
  await page.getByRole('button', { name: /Create Plan/i }).click();
  await expect(page.getByText('Plan created.')).toBeVisible();
  await page.getByPlaceholder('Customer UUID').fill('33333333-3333-3333-3333-333333333333');
  await page.locator('select').nth(1).selectOption({ index: 1 });
  await page.getByRole('button', { name: /Create Subscription/i }).click();
  await expect(page.getByText('Subscription created.')).toBeVisible();
  await page.getByRole('button', { name: /Pause/i }).first().click();
  await expect(page.getByText('Subscription paused.')).toBeVisible();

  await page.goto('/mandates');
  await expect(page.getByRole('heading', { name: 'Mandates' })).toBeVisible();
  await expect(page.getByText('feelpay.seed@bank')).toBeVisible();
  await page.getByPlaceholder('Limit INR').fill('1500');
  await page.getByPlaceholder('payer@upi').fill('newbuyer@upi');
  await page.getByPlaceholder('Payer name').fill('New Buyer');
  await page.getByRole('button', { name: /Create Mandate/i }).click();
  await expect(page.getByText('Mandate initiated.')).toBeVisible();
  const mandateRow = page.getByRole('row').filter({ hasText: 'feelpay.seed@bank' });
  await mandateRow.getByPlaceholder('INR').fill('100');
  await mandateRow.getByRole('button', { name: /Execute/i }).click();
  await expect(page.getByText('Mandate executed.')).toBeVisible();

  await page.goto('/virtual-accounts');
  await expect(page.getByRole('heading', { name: 'Virtual Accounts' })).toBeVisible();
  await expect(page.getByText('Demo Collections')).toBeVisible();
  await page.getByPlaceholder('Beneficiary name').fill('New Collections');
  await page.getByRole('button', { name: /Create Virtual Account/i }).click();
  await expect(page.getByText('Virtual account created.')).toBeVisible();
  await expect(page.getByText('New Collections')).toBeVisible();
});
