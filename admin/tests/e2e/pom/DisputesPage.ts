import { BasePage } from './BasePage';

export class DisputesPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Disputes' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }
}
