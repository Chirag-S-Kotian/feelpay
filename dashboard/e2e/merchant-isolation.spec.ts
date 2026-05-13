import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('merchant-scoped report APIs reject foreign merchant paths', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  await page.goto('/exports');
  await expect(page.getByRole('heading', { name: /Transaction Exports/i })).toBeVisible();

  await page.evaluate(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return;
    localStorage.setItem(
      'feelpay_merchant',
      JSON.stringify({
        ...JSON.parse(raw),
        id: '00000000-0000-0000-0000-000000000999',
      }),
    );
  });
  await expect
    .poll(() =>
      page.evaluate(() => {
        const raw = localStorage.getItem('feelpay_merchant');
        return raw ? JSON.parse(raw).id : null;
      }),
    )
    .toBe('00000000-0000-0000-0000-000000000999');

  await page.getByRole('link', { name: /^Reports$/i }).click();
  await expect(page.getByRole('heading', { name: /^Reports$/i })).toBeVisible();

  const status = await page.evaluate(async () => {
    const response = await fetch(
      '/api/v1/merchants/00000000-0000-0000-0000-000000000999/exports?limit=20&offset=0',
      {
        headers: {
          authorization: 'Bearer fake-token',
        },
      },
    );
    return statusFromResponse(response);

    function statusFromResponse(response: Response) {
      return response.status;
    }
  });

  expect(status).toBe(403);
});
