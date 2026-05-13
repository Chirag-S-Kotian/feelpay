import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('merchant notifications show database-backed alerts and mark read', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/notifications');

  await expect(page.getByRole('heading', { name: 'Notifications' })).toBeVisible();
  await expect(page.getByText('KYC incomplete')).toBeVisible();
  await expect(page.getByText('Complete KYC/KYB before requesting live mode.')).toBeVisible();
  await page.getByRole('button', { name: 'Mark read' }).click();
  await expect(page.getByRole('button', { name: 'Mark read' })).toHaveCount(0);
});
