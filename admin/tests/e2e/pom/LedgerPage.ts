import { expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class LedgerPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Ledger' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async expectLedgerBalance(label: string) {
    const metric = this.page.locator('.stat-card', { hasText: label });
    await expect(metric).toBeVisible();
  }
}
