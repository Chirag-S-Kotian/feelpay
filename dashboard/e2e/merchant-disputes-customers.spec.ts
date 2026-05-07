import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('disputes empty state is honest and customers derive from sandbox payments', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  await page.goto('/disputes');
  await expect(page.getByText('No disputes yet.')).toBeVisible();
  await expect(page.getByRole('button', { name: /Upload Evidence/i })).toBeDisabled();

  await page.goto('/customers');
  await expect(page.getByRole('heading', { name: 'Customers' })).toBeVisible();
  await expect(page.getByText('buyer@example.com')).toBeVisible();
});
