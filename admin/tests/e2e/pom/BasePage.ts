import { Page, expect } from '@playwright/test';
import { setupConsoleGuards, expectNoSensitiveLeak } from '../helpers/consoleGuards';
import { setupApiGuards } from '../helpers/apiGuards';

export abstract class BasePage {
  constructor(protected readonly page: Page) {}

  async setupGuards() {
    setupConsoleGuards(this.page);
    setupApiGuards(this.page);
  }

  async verifyNoLeaks() {
    await expectNoSensitiveLeak(this.page);
  }

  async waitForAlert(message?: string | RegExp) {
    const alert = this.page.locator('.alert');
    await expect(alert).toBeVisible();
    if (message) {
      await expect(alert).toContainText(message);
    }
  }

  async waitForLoadingFinished() {
    // Wait for any skeleton loaders or spinners to disappear if they exist
    await this.page.waitForSelector('.animate-pulse', { state: 'detached' });
    await this.page.waitForSelector('.loading-spinner', { state: 'detached' });
  }
}
