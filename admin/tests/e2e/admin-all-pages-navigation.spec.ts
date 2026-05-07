import { Page, test } from '@playwright/test';
import { ADMIN_ROLES, loginAsAdmin } from './helpers/auth';
import { expectSecurityCleanPage } from './helpers/securityAssertions';
import { GenericAdminPage } from './pages/GenericAdminPage';

type PageObjectFactory = (page: Page) => GenericAdminPage;

const adminPages: { area: string; name: string; create: PageObjectFactory }[] = [
  { area: 'Operations', name: 'Overview', create: page => new GenericAdminPage(page, { path: '/', navLabel: 'Overview', heading: /Global Overview|Overview|Dashboard/i }) },
  { area: 'Operations', name: 'Payment Ops', create: page => new GenericAdminPage(page, { path: '/payment-ops', navLabel: 'Payment Ops', heading: /Payment Ops|Payment Operations/i }) },
  { area: 'Operations', name: 'Transactions', create: page => new GenericAdminPage(page, { path: '/transactions', navLabel: 'Transactions', heading: 'Transactions' }) },
  { area: 'Operations', name: 'Refunds', create: page => new GenericAdminPage(page, { path: '/refunds', navLabel: 'Refunds', heading: 'Refunds' }) },
  { area: 'Operations', name: 'Settlements', create: page => new GenericAdminPage(page, { path: '/settlements', navLabel: 'Settlements', heading: 'Settlements' }) },
  { area: 'Operations', name: 'Disputes', create: page => new GenericAdminPage(page, { path: '/disputes', navLabel: 'Disputes', heading: 'Disputes' }) },
  { area: 'Operations', name: 'Risk Rules', create: page => new GenericAdminPage(page, { path: '/risk-rules', navLabel: 'Risk Rules', heading: 'Risk Rules' }) },
  { area: 'Operations', name: 'Reconciliation', create: page => new GenericAdminPage(page, { path: '/reconciliation', navLabel: 'Reconciliation', heading: 'Reconciliation' }) },
  { area: 'Accounts', name: 'Business Accounts', create: page => new GenericAdminPage(page, { path: '/business-accounts', navLabel: 'Business Accounts', heading: 'Business Accounts' }) },
  { area: 'Accounts', name: 'Bank Integrations', create: page => new GenericAdminPage(page, { path: '/bank-integrations', navLabel: 'Bank Integrations', heading: 'Bank Integrations' }) },
  { area: 'Accounts', name: 'Money Movement', create: page => new GenericAdminPage(page, { path: '/money-movement', navLabel: 'Money Movement', heading: 'Money Movement' }) },
  { area: 'Accounts', name: 'Ledger', create: page => new GenericAdminPage(page, { path: '/ledger', navLabel: 'Ledger', heading: 'Ledger' }) },
  { area: 'Configuration', name: 'Provider Connectors', create: page => new GenericAdminPage(page, { path: '/connectors', navLabel: 'Provider Connectors', heading: 'Provider Connectors' }) },
  { area: 'Configuration', name: 'Routing Rules', create: page => new GenericAdminPage(page, { path: '/routing-rules', navLabel: 'Routing Rules', heading: 'Routing Rules' }) },
  { area: 'Configuration', name: 'White-label Checkout', create: page => new GenericAdminPage(page, { path: '/white-label-checkout', navLabel: 'White-label Checkout', heading: 'White-label Checkout' }) },
  { area: 'Configuration', name: 'Webhook Config', create: page => new GenericAdminPage(page, { path: '/webhook-config', navLabel: 'Webhook Config', heading: 'Webhook Config' }) },
  { area: 'Configuration', name: 'Developer Tools', create: page => new GenericAdminPage(page, { path: '/developer-tools', navLabel: 'Developer Tools', heading: 'Developer Tools' }) },
  { area: 'Configuration', name: 'Platform Settings', create: page => new GenericAdminPage(page, { path: '/settings', navLabel: 'Platform Settings', heading: 'Platform Settings' }) },
  { area: 'Reports', name: 'Revenue', create: page => new GenericAdminPage(page, { path: '/revenue', navLabel: 'Revenue', heading: /Revenue/i }) },
  { area: 'Reports', name: 'Reports', create: page => new GenericAdminPage(page, { path: '/reports', navLabel: 'Reports', heading: 'Reports' }) },
  { area: 'Reports', name: 'Bulk Export', create: page => new GenericAdminPage(page, { path: '/bulk-export', navLabel: 'Bulk Export', heading: /Bulk Export/i }) },
  { area: 'Admin', name: 'Audit Logs', create: page => new GenericAdminPage(page, { path: '/audit-logs', navLabel: 'Audit Logs', heading: 'Audit Logs' }) },
  { area: 'Admin', name: 'Admin Users & Roles', create: page => new GenericAdminPage(page, { path: '/admin-users-roles', navLabel: 'Admin Users & Roles', heading: 'Admin Users & Roles' }) },
  { area: 'Admin', name: 'System Health', create: page => new GenericAdminPage(page, { path: '/system-health', navLabel: 'System Health', heading: 'System Health' }) },
  { area: 'Admin', name: 'Notifications', create: page => new GenericAdminPage(page, { path: '/notifications', navLabel: 'Notifications', heading: 'Notifications' }) },
  { area: 'Admin', name: 'Compliance Center', create: page => new GenericAdminPage(page, { path: '/compliance-center', navLabel: 'Compliance Center', heading: 'Compliance Center' }) },
  { area: 'Admin', name: 'Readiness Scorecard', create: page => new GenericAdminPage(page, { path: '/readiness-scorecard', navLabel: 'Readiness Scorecard', heading: 'Readiness Scorecard' }) },
];

test.describe('all admin pages navigation', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
  });

  for (const adminPage of adminPages) {
    test(`${adminPage.area}: ${adminPage.name} loads cleanly`, async ({ page }) => {
      const pageObject = adminPage.create(page);
      await pageObject.goto();
      await pageObject.expectNoBrokenEmptyState();
      await expectSecurityCleanPage(page);
    });
  }
});
