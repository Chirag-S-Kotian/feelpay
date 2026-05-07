import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class SystemHealthPage extends AdminShellPage {
  readonly pageTitle = 'System Health';

  async goto() {
    await this.navigateTo('System Health');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  async expectAllServicesHealthy() {
    const healthBadges = this.page.locator('.health-badge');
    const count = await healthBadges.count();
    for (let i = 0; i < count; i++) {
      await expect(healthBadges.nth(i)).toContainText(/Healthy|Up|Online/i);
    }
  }

  async runDiagnostics() {
    await this.page.getByRole('button', { name: /Run Diagnostics/i }).click();
    await this.waitForLoadingFinished();
    await expect(this.page.locator('.diagnostics-results')).toBeVisible();
  }
}
