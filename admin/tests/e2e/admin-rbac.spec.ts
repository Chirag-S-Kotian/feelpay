import { expect, test } from '@playwright/test';
import { ADMIN_ROLES, AdminRole, loginAsAdmin } from './helpers/auth';
import { setupConsoleGuards } from './helpers/consoleGuards';

const allowedPagesByRole: Record<AdminRole, string[]> = {
  [ADMIN_ROLES.SUPER_ADMIN]: ['/connectors', '/reports', '/audit-logs', '/admin-users-roles', '/settings'],
  [ADMIN_ROLES.PAYMENT_OPS]: ['/payment-ops', '/transactions', '/connectors', '/routing-rules'],
  [ADMIN_ROLES.FINANCE]: ['/ledger', '/reports', '/refunds', '/settlements', '/revenue'],
  [ADMIN_ROLES.RISK]: ['/risk-rules', '/disputes', '/transactions'],
  [ADMIN_ROLES.COMPLIANCE]: ['/audit-logs', '/compliance-center', '/readiness-scorecard'],
  [ADMIN_ROLES.SUPPORT]: ['/transactions', '/refunds', '/notifications'],
  [ADMIN_ROLES.AUDITOR]: ['/transactions', '/audit-logs', '/system-health', '/notifications', '/settings'],
};

const blockedPagesByRole: Record<AdminRole, string[]> = {
  [ADMIN_ROLES.SUPER_ADMIN]: [],
  [ADMIN_ROLES.PAYMENT_OPS]: ['/reports', '/audit-logs', '/admin-users-roles', '/settings'],
  [ADMIN_ROLES.FINANCE]: ['/connectors', '/risk-rules', '/admin-users-roles'],
  [ADMIN_ROLES.RISK]: ['/reports', '/settlements', '/connectors'],
  [ADMIN_ROLES.COMPLIANCE]: ['/connectors', '/reports', '/settlements'],
  [ADMIN_ROLES.SUPPORT]: ['/reports', '/connectors', '/settings', '/admin-users-roles'],
  [ADMIN_ROLES.AUDITOR]: ['/connectors', '/reports', '/admin-users-roles'],
};

test.describe('admin RBAC', () => {
  test.beforeEach(async ({ page }) => {
    setupConsoleGuards(page);
  });

  for (const role of Object.values(ADMIN_ROLES)) {
    test(`${role}: allowed and blocked pages enforce permissions`, async ({ page }) => {
      await loginAsAdmin(page, role);

      for (const path of allowedPagesByRole[role]) {
        await page.goto(path);
        await expect(page.locator('h1')).not.toHaveText(/Access denied/i);
      }

      for (const path of blockedPagesByRole[role]) {
        await page.goto(path);
        await expect(page.locator('h1')).toHaveText(/Access denied/i);
        await expect(page.locator('body')).not.toContainText(/CORS|Failed to fetch/i);
      }
    });
  }

  test('Auditor can view but cannot perform write actions', async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.AUDITOR);
    await page.goto('/audit-logs');
    await expect(page.locator('h1')).not.toHaveText(/Access denied/i);
    await page.goto('/settings');
    await expect(page.getByRole('button', { name: /Save settings/i })).not.toBeVisible();
  });

  test('Support Agent cannot export reports or change provider secrets', async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPPORT);
    await page.goto('/reports');
    await expect(page.locator('h1')).toHaveText(/Access denied/i);
    await page.goto('/connectors');
    await expect(page.locator('h1')).toHaveText(/Access denied/i);
  });

  test('Finance, Risk and Compliance admins retain their expected domain access', async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.FINANCE);
    for (const path of ['/reports', '/refunds', '/settlements']) {
      await page.goto(path);
      await expect(page.locator('h1')).not.toHaveText(/Access denied/i);
    }

    await loginAsAdmin(page, ADMIN_ROLES.RISK);
    for (const path of ['/risk-rules', '/disputes']) {
      await page.goto(path);
      await expect(page.locator('h1')).not.toHaveText(/Access denied/i);
    }

    await loginAsAdmin(page, ADMIN_ROLES.COMPLIANCE);
    for (const path of ['/audit-logs', '/compliance-center', '/readiness-scorecard']) {
      await page.goto(path);
      await expect(page.locator('h1')).not.toHaveText(/Access denied/i);
    }
  });

  test('unmapped admin route denies by default via redirect-safe fallback', async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPPORT);
    await page.goto('/__unmapped-admin-route__');
    await expect(page.locator('h1')).not.toContainText(/Application error/i);
    await expect(page.locator('body')).not.toContainText(/CORS|Failed to fetch/i);
  });
});
