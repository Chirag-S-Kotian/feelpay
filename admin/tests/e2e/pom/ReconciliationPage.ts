import { BasePage } from './BasePage';

export class ReconciliationPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Reconciliation' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }
}
