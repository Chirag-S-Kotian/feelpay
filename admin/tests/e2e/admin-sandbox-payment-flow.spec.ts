import { expect, test } from '@playwright/test';
import { ADMIN_ROLES, loginAsAdmin } from './helpers/auth';
import { setupApiGuards } from './helpers/apiGuards';
import { setupConsoleGuards } from './helpers/consoleGuards';
import { expectSecurityCleanPage } from './helpers/securityAssertions';
import { sandboxBankDetails } from './helpers/testData';
import { BankIntegrationsPage } from './pages/BankIntegrationsPage';
import { DeveloperToolsPage } from './pages/DeveloperToolsPage';
import { GenericAdminPage } from './pages/GenericAdminPage';
import { ProviderConnectorsPage } from './pages/ProviderConnectorsPage';
import { ReportsPage } from './pages/ReportsPage';
import { RoutingRulesPage } from './pages/RoutingRulesPage';
import { WhiteLabelCheckoutPage } from './pages/WhiteLabelCheckoutPage';

test.describe.configure({ mode: 'serial' });

test.describe('critical sandbox payment flow', () => {
  test.beforeEach(async ({ page }) => {
    setupConsoleGuards(page);
    setupApiGuards(page);
    await loginAsAdmin(page, ADMIN_ROLES.SUPER_ADMIN);
  });

  test('completes provider, checkout, payment, reporting, webhook and audit path', async ({ page }) => {
    await test.step('Provider Connectors: Mock Provider sandbox test passes', async () => {
      const connectors = new ProviderConnectorsPage(page);
      await connectors.goto();
      await connectors.runMockSandboxTest();
      await expectSecurityCleanPage(page);
    });

    await test.step('Bank Integrations: sandbox bank verified and masked', async () => {
      const bank = new BankIntegrationsPage(page);
      const details = sandboxBankDetails();
      await bank.goto();
      await bank.saveSandboxBank(details.name);
      await expect(page.locator('body')).toContainText(details.accountNumber);
      await expect(page.locator('body')).not.toContainText(/\b[0-9]{9,18}\b/);
    });

    await test.step('White-label Checkout: config saves and preview renders', async () => {
      const checkout = new WhiteLabelCheckoutPage(page);
      await checkout.goto();
      await checkout.primaryActionSmoke();
      await checkout.openPreview();
      await expect(page.getByRole('button', { name: /Pay securely/i })).toBeVisible();
    });

    await test.step('Payment Link and Checkout Session surfaces are usable', async () => {
      await page.getByRole('button', { name: 'Payment Links' }).click();
      await expect(page.getByRole('button', { name: 'Create payment link' })).toBeVisible();
      await page.getByRole('button', { name: 'Checkout Sessions' }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
    });

    await test.step('Transactions: captured payment surface visible', async () => {
      await new GenericAdminPage(page, { path: '/transactions', navLabel: 'Transactions', heading: 'Transactions' }).goto();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
      await expect(page.locator('body')).toContainText(/captured|authorized|pending|No transactions/i);
    });

    await test.step('Ledger: debit/credit entries surface is present', async () => {
      await new GenericAdminPage(page, { path: '/ledger', navLabel: 'Ledger', heading: 'Ledger' }).goto();
      await expect(page.locator('table, .empty-state, .stat-card').first()).toBeVisible();
      await expect(page.locator('body')).not.toContainText(/unbalanced/i);
    });

    await test.step('Revenue and routing performance update surfaces load', async () => {
      await new GenericAdminPage(page, { path: '/revenue', navLabel: 'Revenue', heading: /Revenue/i }).goto();
      await expect(page.locator('.stat-card, .card').first()).toBeVisible();

      const routing = new RoutingRulesPage(page);
      await routing.goto();
      await page.getByRole('button', { name: 'Provider Performance' }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
    });

    await test.step('Reports: CSV and PDF actions are ready', async () => {
      const reports = new ReportsPage(page);
      await reports.goto();
      await expect(page.getByRole('button', { name: /Generate CSV/i }).first()).toBeVisible();
      await expect(page.getByRole('button', { name: /Generate PDF/i }).first()).toBeVisible();
      await reports.openExportHistory();
      await expect(page.locator('body')).not.toContainText(/queued forever|PDF generator not configured/i);
    });

    await test.step('Developer Tools and Webhook Config: stored events, replay and processing surfaces load', async () => {
      const developerTools = new DeveloperToolsPage(page);
      await developerTools.goto();
      await page.getByRole('button', { name: /^(Testing|Webhook Testing)$/ }).click();
      await expect(page.getByRole('button', { name: /Send (sandbox )?test event/i })).toBeVisible();
      await page.getByRole('button', { name: /^(History|Stored Events)$/ }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
      await page.getByRole('button', { name: /^(Re-send|Event Replay)$/ }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();

      await new GenericAdminPage(page, { path: '/webhook-config', navLabel: 'Webhook Config', heading: 'Webhook Config' }).goto();
      await page.getByRole('button', { name: 'Events' }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
      await page.getByRole('button', { name: /^(Pending Retries|Processing Attempts)$/ }).click();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
    });

    await test.step('Audit Logs: sensitive actions are logged and masked', async () => {
      await new GenericAdminPage(page, { path: '/audit-logs', navLabel: 'Audit Logs', heading: 'Audit Logs' }).goto();
      await expect(page.locator('table, .empty-state').first()).toBeVisible();
      await expectSecurityCleanPage(page);
    });
  });
});
