import { BasePage } from './BasePage';

export class RefundsPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Refunds' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async getFirstRefundRow() {
    return this.page.locator('table tbody tr').first();
  }

  async approveRefund(row: any) {
    await row.getByRole('button', { name: 'Actions' }).click();
    await this.page.getByRole('option', { name: 'Approve' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForLoadingFinished();
  }
}
