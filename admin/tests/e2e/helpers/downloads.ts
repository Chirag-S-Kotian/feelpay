import { Download, expect, Page } from '@playwright/test';

export async function expectDownloadFromAction(
  page: Page,
  action: () => Promise<void>,
): Promise<Download> {
  const [download] = await Promise.all([
    page.waitForEvent('download'),
    action(),
  ]);

  expect(download.suggestedFilename()).not.toHaveLength(0);
  return download;
}

export async function expectReadyDownloadLink(page: Page, name: string | RegExp) {
  const link = page.getByRole('link', { name }).or(page.getByRole('button', { name }));
  await expect(link.first()).toBeVisible();
}

