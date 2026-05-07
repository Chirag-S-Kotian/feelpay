import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class DisputesPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/disputes', navLabel: 'Disputes', heading: 'Disputes', primaryAction: /View detail|Submit dispute evidence/i });
  }
}

