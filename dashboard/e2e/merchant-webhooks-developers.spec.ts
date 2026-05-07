import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('webhook test/replay and developer logs are visible without exposing secrets', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  await page.goto('/webhooks');
  await expect(page.getByRole('heading', { name: /Webhook Testing/i })).toBeVisible();
  await expect(page.getByText(/payment.captured/i).first()).toBeVisible();
  await page.getByRole('button', { name: /Send Test Event/i }).click();
  await expect(page.getByText('Success', { exact: true })).toBeVisible();
  await page.reload();
  await expect(page.locator('.webhook-events-card strong').filter({ hasText: /payment.success/i }).first()).toBeVisible();
  const replay = await page.evaluate(async () => {
    const response = await fetch('/api/v1/merchants/00000000-0000-0000-0000-000000000001/webhooks/replay', {
      method: 'POST',
      headers: { authorization: 'Bearer fake-token', 'content-type': 'application/json' },
      body: JSON.stringify({ event_id: 'evt_test_1' }),
    });
    return response.json();
  });
  expect(replay.status).toBe('delivered');

  await page.goto('/developers');
  await expect(page.getByRole('heading', { name: /Build, test, ship payments fast/i })).toBeVisible();
  await expect(page.getByText(/fp_test_demo_key|API Logs|req_test_1/i).first()).toBeVisible();
  await expect(page.getByText(/4111 1111 1111 1111/)).toHaveCount(0);
});
