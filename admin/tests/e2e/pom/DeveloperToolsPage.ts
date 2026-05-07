import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class DeveloperToolsPage extends AdminShellPage {
  readonly pageTitle = 'Developer Tools';
  
  async goto() {
    await this.navigateTo('Developer Tools');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  async switchTab(tabName: string) {
    await this.page.getByRole('button', { name: tabName }).click();
    await this.waitForLoadingFinished();
  }

  async runSandboxWebhookTest() {
    await this.page.getByRole('button', { name: 'Run sandbox webhook test' }).click();
    await this.waitForAlert(/Sandbox webhook test successful/i);
  }

  async expectEventInList(eventId: string | RegExp) {
    await expect(this.page.locator('table tbody tr').first()).toContainText(eventId);
  }
}
