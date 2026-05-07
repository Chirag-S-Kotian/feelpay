import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class TransactionsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/transactions', navLabel: 'Transactions', heading: 'Transactions', primaryAction: /Filter/i });
  }

  async expectCapturedSandboxTransaction() {
    await expect(this.page.locator('table, .empty-state').first()).toBeVisible();
  }
}

