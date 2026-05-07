import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class ComplianceCenterPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/compliance-center', navLabel: 'Compliance Center', heading: 'Compliance Center', primaryAction: /Start access review/i });
  }

  async expectComplianceTabs() {
    await expect(this.page.getByRole('button', { name: 'Access Reviews' })).toBeVisible();
    await expect(this.page.getByRole('button', { name: 'Security Events' })).toBeVisible();
  }
}

