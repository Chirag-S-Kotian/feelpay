import { expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class RoutingRulesPage extends BasePage {
  async goto() {
    const navItem = this.page.locator('.sidebar:visible .nav-item', { hasText: 'Routing Rules' }).first();
    await navItem.click();
    await this.waitForLoadingFinished();
  }

  async runSimulator(details: { payment_method: string, currency: string, amount: number }) {
    await this.page.getByRole('button', { name: 'Simulator' }).click();
    await this.page.getByLabel('Payment method').fill(details.payment_method);
    await this.page.getByLabel('Currency').fill(details.currency);
    await this.page.getByLabel('Amount').fill(details.amount.toString());
    await this.page.getByRole('button', { name: 'Simulate' }).click();
    await this.waitForLoadingFinished();
  }

  async expectSimulationResult(providerName: string | RegExp) {
    const metric = this.page.locator('.stat-card', { hasText: 'Selected provider' });
    await expect(metric.locator('.stat-value')).toHaveText(providerName);
  }
}
