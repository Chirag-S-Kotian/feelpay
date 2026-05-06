import { test, expect } from '@playwright/test';
import jwt from 'jsonwebtoken';

/**
 * FeelPay Admin Sandbox Payment Flow E2E Test
 * Tests the end-to-end sandbox lifecycle: provider health, bank integration, and transaction management.
 */

test.describe('Admin Sandbox Payment Flow', () => {

  test.beforeEach(async ({ page }) => {
    // Fail immediately on browser console errors
    page.on('console', msg => {
      const text = msg.text();
      const type = msg.type();
      
      const forbiddenPatterns = [
        'CORS',
        'Failed to fetch',
        '404',
        '500',
        'Auth failed',
        'PDF generator not configured',
        'value prop on input should not be null',
        'controlled input to be uncontrolled',
        'uncontrolled input to be controlled'
      ];

      // Ignore normal Vite/HMR logs
      if (text.includes('[vite]') || text.includes('connected.')) return;

      for (const pattern of forbiddenPatterns) {
        if (text.includes(pattern)) {
          throw new Error(`FORBIDDEN CONSOLE ERROR DETECTED: [${type.toUpperCase()}] ${text}`);
        }
      }
    });

    page.on('pageerror', err => {
      throw new Error(`PAGE ERROR DETECTED: ${err.message}`);
    });

    // Generate valid JWT token for authentication
    const secret = process.env.JWT_SECRET || 'feelpay_jwt_secret';
    const merchant_id = 'cd317088-a9ef-495c-ab3e-2a88603c54ac'; // Valid Test Merchant ID
    const payload = {
      sub: merchant_id,
      merchant_id: merchant_id,
      role: 'Super Admin',
      permissions: [
        'payment_ops.view',
        'transactions.view',
        'transactions.action',
        'refunds.request',
        'refunds.approve',
        'refunds.initiate',
        'settlements.manage',
        'disputes.manage',
        'risk_rules.manage',
        'reconciliation.manage',
        'providers.manage',
        'credentials.rotate',
        'bank_integrations.manage',
        'routing.manage',
        'webhook_config.manage',
        'reports.export',
        'audit_logs.view',
        'admin_users.view',
        'admin_users.manage',
        'roles.view',
        'roles.manage',
        'system_health.view',
        'system_health.manage',
        'notifications.view',
        'notifications.manage',
        'platform_settings.view',
        'platform_settings.manage',
        'ledger.view',
        'ledger.export',
        'developer_tools.view',
        'api_keys.manage',
        'api_logs.view',
        'webhook_replay.manage',
        'compliance.view',
        'compliance.manage'
      ],
      iat: Math.floor(Date.now() / 1000),
      exp: Math.floor(Date.now() / 1000) + (60 * 60) // 1 hour
    };
    
    const token = jwt.sign(payload, secret);

    // Inject mock token for authentication
    await page.goto('/login');
    await page.evaluate((t) => {
      localStorage.setItem('feelpay_token', t);
    }, token);
    await page.goto('/');
    
    // Ensure dashboard loads by checking for a core element
    await expect(page.getByRole('heading', { name: /Overview|Dashboard/i })).toBeVisible();
  });

  test('should complete the full sandbox admin flow', async ({ page }) => {
    
    // 1. Provider Sandbox Test
    await test.step('Provider Connectors: Run sandbox test', async () => {
      await page.getByRole('link', { name: 'Provider Connectors' }).click();
      await expect(page.getByRole('heading', { name: 'Provider Connectors' })).toBeVisible();

      const providerCard = page.locator('.card', { hasText: /Mock Provider|Simulator/i }).first();
      await expect(providerCard).toBeVisible();
      
      await providerCard.getByRole('button', { name: 'Run sandbox test' }).click();
      
      // Assert success message and health status
      await expect(page.locator('.alert')).toContainText(/sandbox test passed/i);
      await expect(providerCard).toContainText(/100%/);
      await expect(providerCard).toContainText(/HEALTHY|Ready|Sandbox only/i);
    });

    // 2. Bank Integration
    await test.step('Bank Integrations: Create and verify sandbox bank', async () => {
      await page.getByRole('link', { name: 'Bank Integrations' }).click();
      await expect(page.getByRole('heading', { name: 'Bank Integrations' })).toBeVisible();

      const bankName = `Sandbox Settlement Bank ${Date.now()}`;
      
      // Fill form
      await page.getByLabel('Acquiring bank name').fill(bankName);
      await page.getByLabel('Integration type').selectOption({ label: 'API / sandbox test' });
      await page.getByLabel('Currency').fill('INR');
      await page.getByLabel('Masked account display').fill('****5678');
      
      // Save
      await page.getByRole('button', { name: 'Save bank integration' }).click();
      await page.getByRole('button', { name: 'Confirm' }).click();
      
      await expect(page.locator('.alert')).toContainText(/Bank integration saved/i);

      // Verify row exists and verify bank
      const bankRow = page.getByRole('row', { name: bankName });
      await expect(bankRow).toBeVisible();
      
      await bankRow.getByRole('button', { name: 'Verify sandbox bank setup' }).click();
      await expect(page.locator('.alert')).toContainText(/Sandbox bank setup verified/i);
      await expect(bankRow).toContainText(/active/i);
    });

    // 3. Transaction Lifecycle
    await test.step('Transactions: Capture, Refund, and Cancel lifecycle', async () => {
      await page.getByRole('link', { name: 'Transactions' }).click();
      await expect(page.getByRole('heading', { name: 'Transactions' })).toBeVisible();

      // Find an authorized transaction to work with
      const transactionRow = page.getByRole('row', { name: /authorized|pending/i }).first();
      
      if (await transactionRow.isVisible()) {
        // Step: Capture
        await transactionRow.getByRole('button', { name: 'Capture' }).click();
        await page.getByRole('button', { name: 'Confirm' }).click();
        await expect(page.locator('.alert')).toContainText(/Capture requested/i);
        
        // Step: Refund (assuming it moves to captured status)
        await transactionRow.getByRole('button', { name: 'Request refund' }).click();
        await page.getByRole('button', { name: 'Confirm' }).click();
        await expect(page.locator('.alert')).toContainText(/Refund requested/i);

        // Step: Cancel (find another or test cancellation if status allows)
        const cancelableRow = page.getByRole('row', { name: /authorized|pending/i }).last();
        if (await cancelableRow.isVisible() && cancelableRow !== transactionRow) {
          await cancelableRow.getByRole('button', { name: 'Cancel' }).click();
          await page.getByRole('button', { name: 'Confirm' }).click();
          await expect(page.locator('.alert')).toContainText(/Cancel requested/i);
        }
      } else {
        console.warn('Skipping transaction lifecycle tests: No pending/authorized transactions found.');
      }
    });

    // 4. Readiness Scorecard
    await test.step('Readiness Scorecard: Verify system health', async () => {
      await page.getByRole('link', { name: 'Readiness Scorecard' }).click();
      await expect(page.getByRole('heading', { name: 'Readiness Scorecard' })).toBeVisible();
      
      await page.getByRole('button', { name: 'Run readiness check' }).click();
      await expect(page.getByText('Total readiness score')).toBeVisible();
      
      // Ensure score is present (even if 0, but usually should be > 0 in sandbox if provider is up)
      const scoreElement = page.locator('.stat-value', { hasText: /\d+/ }).first();
      await expect(scoreElement).toBeVisible();
    });
  });

});
