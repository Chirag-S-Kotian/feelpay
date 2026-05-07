import { test, expect } from '@playwright/test';
import { loginAsAdmin, ADMIN_ROLES } from './helpers/auth';
import { AdminShellPage } from './pom/AdminShellPage';
import AxeBuilder from '@axe-core/playwright';

test.describe('Admin Accessibility Suite', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
    const adminShell = new AdminShellPage(page);
    await adminShell.setupGuards();
  });

  const corePages = [
    { label: 'Overview', path: '/' },
    { label: 'Transactions', path: '/transactions' },
    { label: 'Provider Connectors', path: '/connectors' },
    { label: 'Bank Integrations', path: '/bank-integrations' },
    { label: 'Developer Tools', path: '/developer-tools' },
  ];

  for (const p of corePages) {
    test(`accessibility check for ${p.label}`, async ({ page }) => {
      await page.goto(p.path);
      // Wait for any animations/loading
      await page.waitForTimeout(1000); 
      
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();
      const blockingViolations = accessibilityScanResults.violations.filter(violation =>
        violation.impact === 'critical' || violation.impact === 'serious'
      );

      expect(blockingViolations).toEqual([]);
    });
  }
});
