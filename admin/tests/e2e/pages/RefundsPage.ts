import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class RefundsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/refunds', navLabel: 'Refunds', heading: 'Refunds', primaryAction: /Filter|Request refund/i });
  }
}

