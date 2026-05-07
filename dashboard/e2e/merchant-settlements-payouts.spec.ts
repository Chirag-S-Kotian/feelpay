import { expect, test, type Route } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('verified bank account gates payout and settlements show generated sandbox values', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  const emptyBankAccountHandler = async (route: Route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify([]) });
  };
  await page.route('**/{api/,}v1/payouts/bank-accounts', emptyBankAccountHandler);
  await page.goto('/payouts');
  await expect(page.getByRole('heading', { name: 'Payouts' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Create Payout/i })).toBeDisabled();
  await expect(page.getByText(/Add and verify bank account/i)).toBeVisible();
  await page.unroute('**/{api/,}v1/payouts/bank-accounts', emptyBankAccountHandler);

  await page.goto('/bank-accounts');
  await expect(page.getByText(/XXXXXX7890|verified/i).first()).toBeVisible();

  await page.goto('/payouts');
  await expect(page.getByRole('heading', { name: 'Payouts' })).toBeVisible();
  await expect(page.getByText(/verified|HDFC/i).first()).toBeVisible();
  await page.getByPlaceholder('25000').fill('1250');
  await page.getByRole('button', { name: /Create Payout/i }).click();
  await expect(page.getByText(/Payout queued/i)).toBeVisible();
  await expect(page.getByText(/IMPS-SANDBOX/i)).toBeVisible();

  await page.goto('/settlements');
  await expect(page.getByText(/Available Balance/i)).toBeVisible();
  await expect(page.getByText(/UTRTEST123/i)).toBeVisible();
});
