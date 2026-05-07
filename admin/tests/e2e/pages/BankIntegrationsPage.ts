import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class BankIntegrationsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/bank-integrations', navLabel: 'Bank Integrations', heading: 'Bank Integrations', primaryAction: /Save bank integration/i });
  }

  async saveSandboxBank(name: string) {
    await this.page.getByLabel('Acquiring bank name').fill(name);
    await this.page.getByLabel('Integration type').selectOption({ label: 'API / sandbox test' });
    await this.page.getByLabel('Currency').fill('INR');
    await this.page.getByLabel('Masked account display').fill('****5678');
    await this.page.getByRole('button', { name: 'Save bank integration' }).click();
    await expect(this.page.locator('.alert')).toContainText(/Bank integration saved/i);
  }
}

