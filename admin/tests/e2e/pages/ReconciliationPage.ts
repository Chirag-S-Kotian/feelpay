import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class ReconciliationPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/reconciliation', navLabel: 'Reconciliation', heading: 'Reconciliation', primaryAction: /Upload provider report/i });
  }

  async expectWorkflowTabs() {
    await expect(this.page.getByRole('button', { name: 'Uploads', exact: true })).toBeVisible();
    await expect(this.page.getByRole('button', { name: 'Mismatches', exact: true })).toBeVisible();
  }
}
