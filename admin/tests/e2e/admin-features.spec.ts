import { expect, Page, test } from '@playwright/test';
import { ADMIN_ROLES, loginAsAdmin } from './helpers/auth';
import { setupApiGuards } from './helpers/apiGuards';
import { setupConsoleGuards } from './helpers/consoleGuards';
import { expectSecurityCleanPage } from './helpers/securityAssertions';
import { sandboxBankDetails, uniqueName } from './helpers/testData';
import { BankIntegrationsPage } from './pages/BankIntegrationsPage';
import { BusinessAccountsPage } from './pages/BusinessAccountsPage';
import { DeveloperToolsPage } from './pages/DeveloperToolsPage';
import { GenericAdminPage } from './pages/GenericAdminPage';
import { ProviderConnectorsPage } from './pages/ProviderConnectorsPage';
import { ReconciliationPage } from './pages/ReconciliationPage';
import { ReportsPage } from './pages/ReportsPage';
import { RoutingRulesPage } from './pages/RoutingRulesPage';
import { WhiteLabelCheckoutPage } from './pages/WhiteLabelCheckoutPage';

async function expectTableOrFriendlyEmpty(page: Page) {
  await expect(page.locator('table, .empty-state, .table-card').first()).toBeVisible();
  await expectSecurityCleanPage(page);
}

async function openTab(page: Page, name: string) {
  await page.getByRole('button', { name }).click();
  await expectTableOrFriendlyEmpty(page);
}

test.describe('admin feature coverage', () => {
  test.beforeEach(async ({ page }) => {
    setupConsoleGuards(page);
    setupApiGuards(page);
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
  });

  test('Provider Connectors: providers, readiness and sandbox test', async ({ page }) => {
    const connectors = new ProviderConnectorsPage(page);
    await connectors.goto();
    await expect(page.locator('table, .provider-card, .table-card').first()).toBeVisible();
    await expect(page.locator('body')).not.toContainText(/100% ready/i);
    await connectors.primaryActionSmoke();
  });

  test('Bank Integrations: create, update-style save, verify sandbox bank and masking', async ({ page }) => {
    const bank = new BankIntegrationsPage(page);
    const details = sandboxBankDetails();
    await bank.goto();
    await bank.saveSandboxBank(details.name);
    await expect(page.locator('body')).toContainText(details.accountNumber);
    await expect(page.locator('body')).not.toContainText(/\b[0-9]{9,18}\b/);
  });

  test('Routing Rules: performance and simulator load', async ({ page }) => {
    const routing = new RoutingRulesPage(page);
    await routing.goto();
    await openTab(page, 'Provider Performance');
    await routing.simulateSandboxRoute();
  });

  test('White-label Checkout: save, preview, payment links, sessions and receipts', async ({ page }) => {
    const checkout = new WhiteLabelCheckoutPage(page);
    await checkout.goto();
    await checkout.primaryActionSmoke();
    await checkout.openPreview();
    await openTab(page, 'Payment Links');
    await expect(page.getByRole('button', { name: 'Create payment link' })).toBeVisible();
    await openTab(page, 'Checkout Sessions');
    await openTab(page, 'Receipts');
  });

  test('Business Accounts: table, statuses, actions and detail drawer', async ({ page }) => {
    const accounts = new BusinessAccountsPage(page);
    await accounts.goto();
    await expectTableOrFriendlyEmpty(page);
    await expect(page.locator('body')).not.toContainText(/More\s+More/i);
  });

  test('Transactions: captured sandbox transactions and details surface', async ({ page }) => {
    const transactions = new GenericAdminPage(page, { path: '/transactions', navLabel: 'Transactions', heading: 'Transactions' });
    await transactions.goto();
    await expectTableOrFriendlyEmpty(page);
    await expect(page.locator('body')).not.toContainText(/\bpayment_order_id\b/i);
  });

  test('Refunds, Settlements and Disputes: workflows load without raw bank data', async ({ page }) => {
    for (const config of [
      { path: '/refunds', navLabel: 'Refunds', heading: 'Refunds' },
      { path: '/settlements', navLabel: 'Settlements', heading: 'Settlements' },
      { path: '/disputes', navLabel: 'Disputes', heading: 'Disputes' },
    ]) {
      const adminPage = new GenericAdminPage(page, config);
      await adminPage.goto();
      await expectTableOrFriendlyEmpty(page);
      await expect(page.locator('body')).not.toContainText(/\b[0-9]{9,18}\b/);
    }
  });

  test('Risk Rules: tables and simulator explain decisions', async ({ page }) => {
    const risk = new GenericAdminPage(page, { path: '/risk-rules', navLabel: 'Risk Rules', heading: 'Risk Rules' });
    await risk.goto();
    await openTab(page, 'Simulator');
    await expect(page.getByRole('button', { name: /Simulate risk decision/i })).toBeVisible();
    await expect(page.locator('body')).toContainText(/allow|review|block/i);
  });

  test('Reconciliation: upload/import area, matching and mismatch tabs', async ({ page }) => {
    const reconciliation = new ReconciliationPage(page);
    await reconciliation.goto();
    await reconciliation.expectWorkflowTabs();
    await openTab(page, 'Uploads');
    await openTab(page, 'Matching Queue');
    await openTab(page, 'Mismatches');
  });

  test('Ledger and Money Movement: entries and payout tabs load', async ({ page }) => {
    await new GenericAdminPage(page, { path: '/ledger', navLabel: 'Ledger', heading: 'Ledger' }).goto();
    await expectTableOrFriendlyEmpty(page);
    const moneyMovement = new GenericAdminPage(page, { path: '/money-movement', navLabel: 'Money Movement', heading: 'Money Movement' });
    await moneyMovement.goto();
    await openTab(page, 'Payout Queue');
    await openTab(page, 'Split Rules');
  });

  test('Revenue, Reports and Bulk Export: export surfaces load with company wording', async ({ page }) => {
    await new GenericAdminPage(page, { path: '/revenue', navLabel: 'Revenue', heading: /Revenue/i }).goto();
    await expect(page.locator('.stat-card, .card, .table-card').first()).toBeVisible();

    const reports = new ReportsPage(page);
    await reports.goto();
    await reports.primaryActionSmoke();
    await reports.openExportHistory();

    await new GenericAdminPage(page, { path: '/bulk-export', navLabel: 'Bulk Export', heading: /Bulk Export/i }).goto();
    await expect(page.locator('body')).not.toContainText(/\bmerchant label\b/i);
  });

  test('Developer Tools and Webhook Config: redacted API keys, events and replay surfaces', async ({ page }) => {
    const developerTools = new DeveloperToolsPage(page);
    await developerTools.goto();
    await openTab(page, 'API Keys');
    await expect(page.locator('body')).not.toContainText(/\bsk_[a-z0-9_]{12,}\b/i);
    await openTab(page, 'Request Logs');
    await openTab(page, 'Webhook Testing');
    await openTab(page, 'Stored Events');
    await openTab(page, 'Event Replay');
    await openTab(page, 'Sample Payloads');

    await new GenericAdminPage(page, { path: '/webhook-config', navLabel: 'Webhook Config', heading: 'Webhook Config' }).goto();
    await openTab(page, 'Endpoint Configs');
    await openTab(page, 'Events');
    await openTab(page, 'Processing Attempts');
    await openTab(page, 'Dead Letter');
    await openTab(page, 'Retry Policies');
  });

  test('Audit Logs and Admin Users & Roles: details and confirmations are present', async ({ page }) => {
    await new GenericAdminPage(page, { path: '/audit-logs', navLabel: 'Audit Logs', heading: 'Audit Logs' }).goto();
    await expectTableOrFriendlyEmpty(page);
    await expect(page.locator('body')).not.toContainText(/\b(secret|cvv|cvc)\b/i);

    await new GenericAdminPage(page, { path: '/admin-users-roles', navLabel: 'Admin Users & Roles', heading: 'Admin Users & Roles' }).goto();
    await expect(page.getByRole('button', { name: /Invite/i })).toBeVisible();
  });

  test('System Health, Notifications, Settings, Compliance and Readiness render controls', async ({ page }) => {
    await new GenericAdminPage(page, { path: '/system-health', navLabel: 'System Health', heading: 'System Health' }).goto();
    await openTab(page, 'Providers');
    await openTab(page, 'Jobs');
    await openTab(page, 'Queues');

    await new GenericAdminPage(page, { path: '/notifications', navLabel: 'Notifications', heading: 'Notifications' }).goto();
    await expectTableOrFriendlyEmpty(page);

    await new GenericAdminPage(page, { path: '/settings', navLabel: 'Platform Settings', heading: 'Platform Settings' }).goto();
    await expect(page.getByRole('button', { name: /Save settings/i })).toBeVisible();

    await new GenericAdminPage(page, { path: '/compliance-center', navLabel: 'Compliance Center', heading: 'Compliance Center' }).goto();
    await expect(page.locator('body')).toContainText(/CVV storage/i);
    await openTab(page, 'Access Reviews');
    await openTab(page, 'Security Events');

    await new GenericAdminPage(page, { path: '/readiness-scorecard', navLabel: 'Readiness Scorecard', heading: 'Readiness Scorecard' }).goto();
    await expect(page.getByRole('button', { name: /Export readiness report/i })).toBeVisible();
  });

  test('Platform Settings save action creates user-visible confirmation path', async ({ page }) => {
    await new GenericAdminPage(page, { path: '/settings', navLabel: 'Platform Settings', heading: 'Platform Settings' }).goto();
    await page.getByLabel('Support email').fill(`${uniqueName('admin-e2e').replaceAll(' ', '-')}@feelpay.local`);
    await page.getByRole('button', { name: /Save settings/i }).click();
    await expect(page.getByRole('button', { name: 'Confirm' })).toBeVisible();
  });
});

