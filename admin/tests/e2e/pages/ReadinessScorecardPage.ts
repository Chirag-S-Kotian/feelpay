import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class ReadinessScorecardPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/readiness-scorecard', navLabel: 'Readiness Scorecard', heading: 'Readiness Scorecard', primaryAction: /Export readiness report/i });
  }
}

