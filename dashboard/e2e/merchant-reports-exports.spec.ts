import { expect, test } from '@playwright/test';
import { installMerchantApi, seedMerchantSession } from './merchant-fixtures';

test('exports, settlement reports, and scheduled reports expose history states', async ({ page }) => {
  await installMerchantApi(page);
  await seedMerchantSession(page);

  await page.goto('/exports');
  await expect(page.getByRole('heading', { name: /Exports/i })).toBeVisible();
  await page.getByPlaceholder('Start date').fill('2026-05-01');
  await page.getByPlaceholder('End date').fill('2026-05-07');
  await page.getByRole('button', { name: /Export Transactions/i }).click();
  await expect(page.getByText(/Export job created/i)).toBeVisible();
  await page.getByRole('button', { name: /Export History/i }).click();
  await expect(page.getByText(/completed|transactions/i).first()).toBeVisible();
  const exportDownloadRequest = page.waitForRequest(/\/exports\/exp_test_1\/download/);
  await page.locator('button.btn-small').first().click();
  await exportDownloadRequest;

  await page.goto('/settlement-reports');
  await expect(page.getByRole('heading', { name: /Settlement Reports/i })).toBeVisible();
  await expect(page.getByText(/UTRTEST123|completed/i).first()).toBeVisible();
  await page.locator('button.btn-small').first().click();
  await expect(page.getByRole('heading', { name: /Report Preview/i })).toBeVisible();
  const pdfDownload = page.waitForEvent('download');
  await page.getByRole('button', { name: /^PDF$/i }).click();
  await page.getByRole('button', { name: /Download Report/i }).click();
  expect((await pdfDownload).suggestedFilename()).toMatch(/\.pdf$/);
  const csvDownload = page.waitForEvent('download');
  await page.getByRole('button', { name: /^CSV$/i }).click();
  await page.getByRole('button', { name: /Download Report/i }).click();
  expect((await csvDownload).suggestedFilename()).toMatch(/\.csv$/);

  await page.goto('/scheduled-reports');
  await expect(page.getByRole('heading', { name: /Scheduled Reports/i })).toBeVisible();
  await expect(page.getByText(/daily|active|merchant@example.com/i).first()).toBeVisible();
});
