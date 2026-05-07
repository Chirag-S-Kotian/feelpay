import { BasePage } from './BasePage';

export class BusinessAccountsPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Business Accounts' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async reviewKYC(businessName: string) {
    const row = this.page.locator('table tbody tr', { hasText: businessName });
    await row.getByRole('button', { name: 'Actions' }).click();
    await this.page.getByRole('option', { name: 'Review KYC' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForLoadingFinished();
  }
}
