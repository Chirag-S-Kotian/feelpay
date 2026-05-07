import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class BankIntegrationsPage extends AdminShellPage {
  readonly pageTitle = 'Bank Integrations';
  readonly form = this.page.locator('form');
  readonly bankNameInput = this.form.getByLabel('Acquiring bank name');
  readonly integrationTypeSelect = this.form.getByLabel('Integration type');
  readonly saveButton = this.form.getByRole('button', { name: 'Save bank integration' });

  async goto() {
    await this.navigateTo('Bank Integrations');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  async createBankIntegration(details: {
    name: string;
    type: string;
    currency: string;
    accountNumber: string;
  }) {
    await this.bankNameInput.fill(details.name);
    await this.form.getByLabel('Integration type').selectOption({ label: details.type });
    await this.form.getByLabel('Currency').fill(details.currency);
    await this.form.getByLabel('Masked account display').fill(details.accountNumber);
    await this.saveButton.click();
    await this.waitForAlert(/Bank integration saved/i);
  }

  async verifySandboxBank(bankName: string) {
    const row = this.page.locator('table tbody tr').filter({ hasText: bankName });
    await row.getByRole('button', { name: 'Verify sandbox bank setup' }).click();
    await this.waitForAlert(/Sandbox bank setup verified/i);
  }
}
