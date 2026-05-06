import { test, expect } from '@playwright/test';

/**
 * FeelPay Admin Smoke Test Suite
 * Covers critical flows to ensure platform health.
 */

test.describe('Admin Dashboard Smoke Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Inject a mock token for UI-only smoke tests to bypass login
    await page.goto('/login');
    await page.evaluate(() => {
      localStorage.setItem('feelpay_token', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJyb2xlIjoiYWRtaW4iLCJwZXJtaXNzaW9ucyI6WyJwYXltZW50X29wcy52aWV3IiwidHJhbnNhY3Rpb25zLnZpZXciLCJsZWRnZXIudmlldyIsInByb3ZpZGVycy5tYW5hZ2UiLCJiYW5rX2ludGVncmF0aW9ucy5tYW5hZ2UiLCJ3ZWJob29rX2NvbmZpZy5tYW5hZ2UiLCJkZXZlbG9wZXJfdG9vbHMudmlldyIsImF1ZGl0X2xvZ3MudmlldyJdfQ.mock_signature');
    });
    await page.goto('/');
  });

  test('should load the shell and dashboard overview', async ({ page }) => {
    await expect(page.locator('.sidebar')).toBeVisible();
    await expect(page.getByText('FeelPay', { exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();
  });

  test('should navigate to Provider Connectors', async ({ page }) => {
    await page.getByRole('link', { name: 'Provider Connectors' }).click();
    await expect(page.getByRole('heading', { name: 'Provider Connectors' })).toBeVisible();
  });

  test('should navigate to Bank Integrations', async ({ page }) => {
    await page.getByRole('link', { name: 'Bank Integrations' }).click();
    await expect(page.getByRole('heading', { name: 'Bank Integrations' })).toBeVisible();
  });

  test('should navigate to Developer Tools and check Webhook Events', async ({ page }) => {
    await page.getByRole('link', { name: 'Developer Tools' }).click();
    await expect(page.getByRole('heading', { name: 'Developer Tools' })).toBeVisible();
    
    const webhookTab = page.getByRole('tab', { name: /Webhook Events/i });
    if (await webhookTab.isVisible()) {
      await webhookTab.click();
      await expect(page.getByText(/Webhook Events/i)).toBeVisible();
    }
  });

  test('should navigate to Audit Logs', async ({ page }) => {
    await page.getByRole('link', { name: 'Audit Logs' }).click();
    await expect(page.getByRole('heading', { name: 'Audit Logs' })).toBeVisible();
  });
  
  test('should navigate to Platform Settings', async ({ page }) => {
    await page.getByRole('link', { name: 'Platform Settings' }).click();
    await expect(page.getByRole('heading', { name: 'Platform Settings' })).toBeVisible();
  });
});
