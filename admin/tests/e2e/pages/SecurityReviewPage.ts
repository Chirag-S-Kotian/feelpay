import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class SecurityReviewPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/security-review', navLabel: 'Security Review', heading: /Security Review|Access denied/i });
  }
}

