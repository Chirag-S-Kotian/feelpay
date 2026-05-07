import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class SettlementsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/settlements', navLabel: 'Settlements', heading: 'Settlements', primaryAction: /View detail|Initiate payout/i });
  }
}

