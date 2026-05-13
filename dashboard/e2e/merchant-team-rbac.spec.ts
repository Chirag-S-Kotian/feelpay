import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('merchant team page supports required RBAC roles', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await page.goto('/team');

  await expect(page.getByRole('heading', { name: 'Team' })).toBeVisible();
  await expect(page.getByText('finance@example.com')).toBeVisible();
  await expect(page.getByLabel('Role').first()).toContainText('Developer');
  await expect(page.getByLabel('Role').first()).toContainText('Finance');
  await expect(page.getByLabel('Role').first()).toContainText('Support');
  await expect(page.getByLabel('Role').first()).toContainText('Read Only');

  await page.getByPlaceholder('Jane Doe').fill('Support User');
  await page.getByPlaceholder('jane@company.com').fill('support@example.com');
  await page.getByLabel('Role').first().selectOption('merchant_support');
  await page.getByRole('button', { name: 'Invite' }).click();
  await expect(page.getByText('support@example.com')).toBeVisible();

  await page.getByLabel('Role for support@example.com').selectOption('merchant_read_only');
  await expect(page.getByText('Team role updated.')).toBeVisible();
});
