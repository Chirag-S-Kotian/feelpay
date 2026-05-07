import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class NotificationsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/notifications', navLabel: 'Notifications', heading: 'Notifications', primaryAction: /Inbox|Critical/i });
  }
}

