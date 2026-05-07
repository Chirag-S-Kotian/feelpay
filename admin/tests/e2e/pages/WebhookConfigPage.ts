import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class WebhookConfigPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/webhook-config', navLabel: 'Webhook Config', heading: 'Webhook Config', primaryAction: /Save retry policy/i });
  }

  async openEvents() {
    await this.page.getByRole('button', { name: 'Events' }).click();
    await expect(this.page.locator('table, .empty-state').first()).toBeVisible();
  }
}

