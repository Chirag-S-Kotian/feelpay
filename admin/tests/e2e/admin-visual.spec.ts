import { expect, Page, test } from '@playwright/test';
import { ADMIN_ROLES, loginAsAdmin } from './helpers/auth';

const dynamicMasks = (page: Page) => [
  page.locator('.stat-value'),
  page.locator('td').filter({ hasText: /^[0-9a-f]{8}-[0-9a-f-]{27,}$/i }),
  page.locator('td').filter({ hasText: /\d{4}-\d{2}-\d{2}|ago|AM|PM/i }),
  page.locator('.chart-container, canvas, svg.recharts-surface'),
];

test.describe('admin visual regression', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
  });

  test('Provider Connectors overview', async ({ page }) => {
    await page.goto('/connectors');
    await expect(page.getByRole('heading', { name: 'Provider Connectors' })).toBeVisible();
    await expect(page).toHaveScreenshot('provider-connectors-overview.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('White-label Checkout preview', async ({ page }) => {
    await page.goto('/white-label-checkout');
    await page.getByRole('button', { name: 'Preview' }).click();
    await expect(page.getByRole('heading', { name: /Checkout preview/i })).toBeVisible();
    await expect(page).toHaveScreenshot('checkout-preview.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('Business Accounts table', async ({ page }) => {
    await page.goto('/business-accounts');
    await expect(page.getByRole('heading', { name: 'Business Accounts' })).toBeVisible();
    await expect(page).toHaveScreenshot('business-accounts-table.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('Reports export history', async ({ page }) => {
    await page.goto('/reports');
    await page.getByRole('button', { name: 'Export History' }).click();
    await expect(page).toHaveScreenshot('reports-history.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('Readiness Scorecard', async ({ page }) => {
    await page.goto('/readiness-scorecard');
    await expect(page.getByRole('heading', { name: 'Readiness Scorecard' })).toBeVisible();
    await expect(page).toHaveScreenshot('readiness-scorecard.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('Compliance Center', async ({ page }) => {
    await page.goto('/compliance-center');
    await expect(page.getByRole('heading', { name: 'Compliance Center' })).toBeVisible();
    await expect(page).toHaveScreenshot('compliance-center.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });

  test('Audit Log detail modal', async ({ page }) => {
    await page.goto('/audit-logs');
    const detail = page.getByRole('button', { name: /View detail/i }).first();
    if (await detail.isVisible()) {
      await detail.click();
    }
    await expect(page).toHaveScreenshot('audit-log-detail.png', { fullPage: true, mask: dynamicMasks(page), animations: 'disabled' });
  });
});

