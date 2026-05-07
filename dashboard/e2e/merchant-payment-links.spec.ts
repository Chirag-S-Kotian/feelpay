import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('create, copy, open, and expire payment link flow is wired', async ({ page, context }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);
  await context.route('https://checkout.feelpay.test/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'text/html',
      body: `
        <main>
          <h1>FeelPay test checkout</h1>
          <button id="pay">Pay success</button>
          <p id="status">Ready</p>
          <script>
            document.getElementById('pay').addEventListener('click', async () => {
              const session = await fetch('http://localhost:3000/api/v1/checkout/session', {
                method: 'POST',
                headers: { 'content-type': 'application/json', authorization: 'Bearer fake-token' },
                body: JSON.stringify({ amount: 250000, currency: 'INR', customer_email: 'buyer@example.com' })
              }).then((response) => response.json());
              await fetch('http://localhost:3000/api/v1/checkout/session/' + session.session_id + '/pay', {
                method: 'POST',
                headers: { 'content-type': 'application/json', authorization: 'Bearer fake-token' },
                body: JSON.stringify({ simulation_action: 'success', selected_payment_method: 'upi' })
              });
              document.getElementById('status').textContent = 'Payment captured';
            });
          </script>
        </main>
      `,
    });
  });
  await page.goto('/payment-links');

  await page.getByRole('button', { name: /create link/i }).first().click();
  await page.getByPlaceholder('Enter amount').fill('2500');
  await page.getByPlaceholder('What is this payment for?').fill('Sandbox invoice');
  await page.getByPlaceholder('customer@example.com').fill('buyer@example.com');
  await page.locator('.payment-link-form-actions button').filter({ hasText: /^Create Link$/ }).click();

  await expect(page.getByText(/Sandbox invoice/i)).toBeVisible();
  await page.locator('button[title="Copy link"]').first().click();
  await expect(page.getByText(/copied/i)).toBeVisible();

  const popupPromise = context.waitForEvent('page');
  await page.locator('a[title="Open link"]').first().click();
  const popup = await popupPromise;
  expect(popup.url()).toContain('checkout.feelpay.test');
  await popup.getByRole('button', { name: /pay success/i }).click();
  await expect(popup.getByText(/Payment captured/i)).toBeVisible();
  await popup.close();

  await page.goto('/payments');
  await expect(page.getByText(/pay_test_/)).toBeVisible();
  await page.goto('/');
  await expect(page.getByText('Successful Txns')).toBeVisible();

  await page.goto('/payment-links');
  page.on('dialog', (dialog) => dialog.accept());
  await page.locator('button[title="Cancel link"]').first().click();
});
