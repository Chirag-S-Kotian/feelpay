import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class GoLivePage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/go-live', navLabel: 'Go-Live', heading: /Go-Live|Go Live|Access denied/i });
  }
}

