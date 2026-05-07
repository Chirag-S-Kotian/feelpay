import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class ProviderConnectorsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/connectors', navLabel: 'Provider Connectors', heading: 'Provider Connectors', primaryAction: /Run sandbox test/i });
  }

  async runMockSandboxTest() {
    await this.page.getByRole('button', { name: /Run sandbox test/i }).first().click();
    await expect(this.page.locator('.alert')).toContainText(/sandbox test (successful|passed)/i);
  }
}

