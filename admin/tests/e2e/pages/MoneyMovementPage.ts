import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class MoneyMovementPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/money-movement', navLabel: 'Money Movement', heading: 'Money Movement', primaryAction: /Payout Queue|Split Rules/i });
  }

  async expectCoreTabs() {
    await expect(this.page.getByRole('button', { name: 'Payout Queue' })).toBeVisible();
    await expect(this.page.getByRole('button', { name: 'Split Rules' })).toBeVisible();
  }
}

