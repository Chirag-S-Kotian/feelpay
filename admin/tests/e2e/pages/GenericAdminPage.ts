import { expect, Locator, Page } from '@playwright/test';
import { setupApiGuards } from '../helpers/apiGuards';
import { setupConsoleGuards } from '../helpers/consoleGuards';
import { expectSecurityCleanPage } from '../helpers/securityAssertions';

export type AdminPageConfig = {
  path: string;
  navLabel: string;
  heading: string | RegExp;
  primaryAction?: string | RegExp;
};

export class GenericAdminPage {
  constructor(
    protected readonly page: Page,
    protected readonly config: AdminPageConfig,
  ) {}

  async goto() {
    setupConsoleGuards(this.page);
    setupApiGuards(this.page);
    await this.page.goto(this.config.path);
    await this.expectLoaded();
  }

  async expectLoaded() {
    await expect(this.page.getByRole('heading', { name: this.config.heading }).first()).toBeVisible();
    await expect(this.page.locator('.page, main').first()).toBeVisible();
    await expect(this.page.locator('body')).not.toContainText(/Something went wrong|Application error|white screen/i);
  }

  async expectNoBrokenEmptyState() {
    const emptyStates = this.page
      .locator('.empty-state')
      .or(this.page.getByText(/No .* yet|No .* configured|No .* found|ready to export/i));
    const count = await emptyStates.count();

    for (let index = 0; index < count; index += 1) {
      const state = emptyStates.nth(index);
      await expect(state).not.toContainText(/\bundefined\b|\bnull\b|\bNaN\b|failed to fetch|route missing/i);
    }
  }

  async expectNoSensitiveLeak() {
    await expectSecurityCleanPage(this.page);
  }

  async primaryActionSmoke() {
    if (!this.config.primaryAction) {
      await this.expectLoaded();
      return;
    }

    const action = this.page.getByRole('button', { name: this.config.primaryAction }).first();
    await expect(action).toBeVisible();
  }

  protected get firstTableRow(): Locator {
    return this.page.locator('table tbody tr').first();
  }
}
