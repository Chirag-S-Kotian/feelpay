import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class DeveloperToolsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/developer-tools', navLabel: 'Developer Tools', heading: 'Developer Tools', primaryAction: /Send sandbox test event/i });
  }

  async sendWebhookTest() {
    await this.page.getByRole('button', { name: 'Webhook Testing' }).click();
    await this.page.getByRole('button', { name: /Send sandbox test event/i }).click();
    await expect(this.page.locator('.alert')).toContainText(/Webhook test sent|successful/i);
  }
}

