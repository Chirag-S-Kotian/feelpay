import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class RevenuePage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/revenue', navLabel: 'Revenue', heading: /Revenue/i });
  }
}

