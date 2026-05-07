import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class AdminUsersRolesPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/admin-users-roles', navLabel: 'Admin Users & Roles', heading: 'Admin Users & Roles', primaryAction: /Invite/i });
  }

  async expectRoleControls() {
    await expect(this.page.getByRole('button', { name: /Invite/i })).toBeVisible();
  }
}

