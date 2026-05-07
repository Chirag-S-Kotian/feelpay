import { expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class SettlementsPage extends BasePage {
  readonly url = '/'; // It's a SPA sub-page, navigate via sidebar

  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Settlements' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async expectSummaryMetric(label: string) {
    const metric = this.page.locator('.stat-card', { hasText: label });
    await expect(metric).toBeVisible();
    await expect(metric.locator('.stat-value')).not.toBeEmpty();
  }

  async getFirstSettlementRow() {
    return this.page.locator('table tbody tr').first();
  }

  async initiateSettlement(row: any) {
    await row.getByRole('button', { name: 'Actions' }).click();
    await this.page.getByRole('option', { name: 'Initiate payout' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForLoadingFinished();
  }
}
