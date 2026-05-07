import { expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class AdminShellPage extends BasePage {
  readonly sidebar = this.page.locator('.sidebar');
  readonly topHeader = this.page.locator('.top-header');
  readonly logoutButton = this.topHeader.getByRole('button', { name: 'Logout' });
  readonly searchInput = this.topHeader.getByPlaceholder('Search payments, accounts, or providers');
  readonly notificationBell = this.topHeader.getByRole('button', { name: 'Notifications' });

  async navigateTo(label: string) {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: label }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async logout() {
    await this.logoutButton.click();
    await expect(this.page).toHaveURL(/\/login/);
  }

  async search(query: string) {
    await this.searchInput.fill(query);
    await this.searchInput.press('Enter');
    await this.waitForLoadingFinished();
  }

  async expectPageTitle(title: string | RegExp) {
    await expect(this.page.locator('h1')).toHaveText(title);
  }
}
