import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class TransactionsPage extends AdminShellPage {
  readonly pageTitle = 'Transactions';

  async goto() {
    await this.navigateTo('Transactions');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  override async search(query: string) {
    await this.page.getByPlaceholder(/Search transactions/i).fill(query);
    await this.page.press('body', 'Enter');
    await this.waitForLoadingFinished();
  }

  async filterByStatus(status: string) {
    await this.page.getByLabel(/Status/i).selectOption(status);
    await this.waitForLoadingFinished();
  }

  async getFirstTransactionRow() {
    return this.page.getByRole('row').nth(1); // Row 0 is header
  }

  async captureTransaction(row: any) {
    await row.getByRole('button', { name: 'Capture' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForAlert(/Capture requested/i);
  }

  async refundTransaction(row: any) {
    await row.getByRole('button', { name: 'Request refund' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForAlert(/Refund requested/i);
  }

  async cancelTransaction(row: any) {
    await row.getByRole('button', { name: 'Cancel' }).click();
    await this.page.getByRole('button', { name: 'Confirm' }).click();
    await this.waitForAlert(/Cancel requested/i);
  }
}
