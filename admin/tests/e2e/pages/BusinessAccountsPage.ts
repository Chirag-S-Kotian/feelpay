import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class BusinessAccountsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/business-accounts', navLabel: 'Business Accounts', heading: 'Business Accounts', primaryAction: /View detail/i });
  }

  async openFirstDetail() {
    await this.page.getByRole('button', { name: /View detail/i }).first().click();
    await expect(this.page.locator('.drawer, [role="dialog"]').first()).toBeVisible();
  }
}

