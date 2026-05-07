import { expect, test } from '@playwright/test';
import { installMerchantApi, loginMerchant } from './merchant-fixtures';

test('sandbox payment journey updates payments, dashboard, webhook, customer, and settlement surfaces', async ({ page }) => {
  await installMerchantApi(page);
  const badResponses: string[] = [];
  const consoleErrors: string[] = [];
  page.on('response', (response) => {
    if ([401, 403, 404].includes(response.status())) {
      badResponses.push(`${response.status()} ${response.url()}`);
    }
  });
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });
  await loginMerchant(page);

  await page.goto('/onboarding');
  await expect(page.getByRole('heading', { name: 'Onboarding' })).toBeVisible();

  await page.goto('/bank-accounts');
  await expect(page.getByText(/verified/i).first()).toBeVisible();

  await page.goto('/payment-methods');
  await expect(page.getByRole('heading', { name: 'Payment Methods' })).toBeVisible();
  await page.getByText('Checkout Order').waitFor();

  await page.goto('/checkout-preview');
  await page.getByRole('button', { name: /success/i }).click();
  await page.getByRole('button', { name: /pay securely/i }).click();
  await expect(page.getByText(/Payment captured/i)).toBeVisible();

  await page.goto('/payments');
  await expect(page.getByText(/pay_test_/)).toBeVisible();

  await page.goto('/');
  await expect(page.getByText('Total Volume')).toBeVisible();
  await expect(page.getByText('Successful Txns')).toBeVisible();

  await page.goto('/analytics');
  await expect(page.getByRole('heading', { name: 'Analytics' })).toBeVisible();

  await page.goto('/webhooks');
  await expect(page.getByText(/payment.captured/i).first()).toBeVisible();

  await page.goto('/customers');
  await expect(page.getByText('buyer@example.com')).toBeVisible();

  await page.goto('/settlements');
  await expect(page.getByText(/UTRTEST123/i)).toBeVisible();

  await page.goto('/exports');
  await expect(page.getByText(/completed|transactions/i).first()).toBeVisible();
  expect(badResponses).toEqual([]);
  expect(consoleErrors).toEqual([]);
});
