import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class DashboardPage extends AdminShellPage {
  readonly statsGrid = this.page.locator('.stats-grid');
  readonly metrics = this.page.locator('.stat-card');

  async goto() {
    await this.page.goto('/');
    await this.waitForLoadingFinished();
  }

  async getMetricValue(label: string) {
    const card = this.metrics.filter({ hasText: label });
    return card.locator('.stat-value').innerText();
  }

  async expectMetricVisible(label: string) {
    await expect(this.metrics.filter({ hasText: label })).toBeVisible();
  }
}
