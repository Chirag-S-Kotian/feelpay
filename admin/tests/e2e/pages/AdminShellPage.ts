import { expect, Page } from '@playwright/test';
import { setupApiGuards } from '../helpers/apiGuards';
import { setupConsoleGuards } from '../helpers/consoleGuards';

export class AdminShellPage {
  constructor(private readonly page: Page) {}

  setupGuards() {
    setupConsoleGuards(this.page);
    setupApiGuards(this.page);
  }

  async goto() {
    this.setupGuards();
    await this.page.goto('/');
    await expect(this.page.locator('.desktop-sidebar, .mobile-sidebar').first()).toBeVisible();
  }

  async expectLoaded() {
    await expect(this.page.getByRole('banner').or(this.page.locator('.top-header')).first()).toBeVisible();
  }

  async navigateTo(label: string) {
    await this.page.locator('.sidebar:visible .nav-item', { hasText: label }).first().click();
    await this.page.waitForLoadState('networkidle').catch(() => undefined);
  }
}

