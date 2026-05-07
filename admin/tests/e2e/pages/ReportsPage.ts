import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class ReportsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/reports', navLabel: 'Reports', heading: 'Reports', primaryAction: /Generate CSV/i });
  }

  async openExportHistory() {
    await this.page.getByRole('button', { name: 'Export History' }).click();
    await expect(this.page.locator('table, .empty-state').first()).toBeVisible();
  }
}

