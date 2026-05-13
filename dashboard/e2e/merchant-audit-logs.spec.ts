import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('merchant audit logs are listed and reveal sanitized event details', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/audit-logs');

  await expect(page.getByRole('heading', { name: 'Audit Logs' })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'team member invited' })).toBeVisible();
  await expect(page.getByText('team_member').first()).toBeVisible();
  await page.getByRole('cell', { name: 'team member invited' }).click();
  await expect(page.getByText('finance@example.com')).toBeVisible();
  await expect(page.locator('.detail-item').filter({ hasText: 'IP' }).getByText('127.0.0.1')).toBeVisible();
});
