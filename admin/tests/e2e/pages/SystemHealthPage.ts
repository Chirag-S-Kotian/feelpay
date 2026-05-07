import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class SystemHealthPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/system-health', navLabel: 'System Health', heading: 'System Health', primaryAction: /Providers|Jobs|Queues/i });
  }

  async expectHealthTabs() {
    await expect(this.page.getByRole('button', { name: 'Providers' })).toBeVisible();
    await expect(this.page.getByRole('button', { name: 'Jobs' })).toBeVisible();
    await expect(this.page.getByRole('button', { name: 'Queues' })).toBeVisible();
  }
}

