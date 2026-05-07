import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class AuditLogsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/audit-logs', navLabel: 'Audit Logs', heading: 'Audit Logs', primaryAction: /View detail/i });
  }

  async openFirstDetailIfPresent() {
    const detail = this.page.getByRole('button', { name: /View detail/i }).first();
    if (await detail.isVisible()) {
      await detail.click();
      await expect(this.page.locator('.drawer, [role="dialog"]').first()).toBeVisible();
    }
  }
}

