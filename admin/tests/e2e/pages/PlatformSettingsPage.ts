import { Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class PlatformSettingsPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/settings', navLabel: 'Platform Settings', heading: 'Platform Settings', primaryAction: /Save settings/i });
  }
}

