import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class RoutingRulesPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/routing-rules', navLabel: 'Routing Rules', heading: 'Routing Rules', primaryAction: /Simulate/i });
  }

  async simulateSandboxRoute() {
    await this.page.getByRole('button', { name: 'Simulator' }).click();
    await this.page.getByLabel('Payment method').fill('upi');
    await this.page.getByLabel('Currency').fill('INR');
    await this.page.getByLabel('Amount').fill('1250');
    await this.page.getByRole('button', { name: 'Simulate' }).click();
    await expect(this.page.locator('.stat-card', { hasText: /Selected provider|No match/i })).toBeVisible();
  }
}

