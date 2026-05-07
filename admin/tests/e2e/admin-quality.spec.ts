import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { ADMIN_ROLES, loginAsAdmin } from './helpers/auth';
import { setupConsoleGuards } from './helpers/consoleGuards';

const pagesToScan = [
  { name: 'Overview', path: '/' },
  { name: 'Provider Connectors', path: '/connectors' },
  { name: 'Bank Integrations', path: '/bank-integrations' },
  { name: 'White-label Checkout', path: '/white-label-checkout' },
  { name: 'Reports', path: '/reports' },
  { name: 'Business Accounts', path: '/business-accounts' },
  { name: 'Audit Logs', path: '/audit-logs' },
  { name: 'Admin Users & Roles', path: '/admin-users-roles' },
];

test.describe('admin accessibility quality', () => {
  test.beforeEach(async ({ page }) => {
    setupConsoleGuards(page);
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
  });

  for (const adminPage of pagesToScan) {
    test(`${adminPage.name} has no critical or serious axe violations`, async ({ page }) => {
      await page.goto(adminPage.path);
      await expect(page.locator('h1')).toBeVisible();

      const results = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();

      const blockingViolations = results.violations.filter(violation =>
        violation.impact === 'critical' || violation.impact === 'serious'
      );

      expect(blockingViolations).toEqual([]);
    });
  }
});
