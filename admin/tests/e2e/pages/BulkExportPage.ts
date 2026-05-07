import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class BulkExportPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/bulk-export', navLabel: 'Bulk Export', heading: /Bulk Export/i });
  }
}

