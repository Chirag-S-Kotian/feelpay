import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class LedgerPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/ledger', navLabel: 'Ledger', heading: 'Ledger', primaryAction: /Export ledger/i });
  }
}

