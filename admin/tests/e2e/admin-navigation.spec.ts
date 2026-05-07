import { test } from '@playwright/test';
import { loginAsAdmin, ADMIN_ROLES } from './helpers/auth';
import { AdminShellPage } from './pom/AdminShellPage';

test.describe('Admin Navigation & Smoke Suite', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
    const adminShell = new AdminShellPage(page);
    await adminShell.setupGuards();
  });

  const pages = [
    { label: 'Overview', title: 'Dashboard' },
    { label: 'Payment Ops', title: 'Payment Operations' },
    { label: 'Transactions', title: 'Transactions' },
    { label: 'Ledger', title: 'Ledger' },
    { label: 'Money Movement', title: 'Money Movement' },
    { label: 'Refunds', title: 'Refunds' },
    { label: 'Settlements', title: 'Settlements' },
    { label: 'Disputes', title: 'Disputes' },
    { label: 'Risk Rules', title: 'Risk Rules' },
    { label: 'Reconciliation', title: 'Reconciliation' },
    { label: 'Business Accounts', title: 'Business Accounts' },
    { label: 'Provider Connectors', title: 'Provider Connectors' },
    { label: 'Bank Integrations', title: 'Bank Integrations' },
    { label: 'Routing Rules', title: 'Routing Rules' },
    { label: 'White-label Checkout', title: 'White-label Checkout' },
    { label: 'Webhook Config', title: 'Webhook Config' },
    { label: 'Developer Tools', title: 'Developer Tools' },
    { label: 'Revenue', title: 'Revenue' },
    { label: 'Bulk Export', title: 'Bulk Export' },
    { label: 'Reports', title: 'Reports' },
    { label: 'Audit Logs', title: 'Audit Logs' },
    { label: 'Admin Users & Roles', title: 'Admin Users & Roles' },
    { label: 'System Health', title: 'System Health' },
    { label: 'Notifications', title: 'Notifications' },
    { label: 'Compliance Center', title: 'Compliance Center' },
    { label: 'Readiness Scorecard', title: 'Readiness Scorecard' },
    { label: 'Platform Settings', title: 'Platform Settings' },
  ];

  for (const p of pages) {
    test(`navigate to ${p.label}`, async ({ page }) => {
      const adminShell = new AdminShellPage(page);
      await adminShell.navigateTo(p.label);
      await adminShell.expectPageTitle(new RegExp(p.title, 'i'));
      await adminShell.verifyNoLeaks();
    });
  }
});
