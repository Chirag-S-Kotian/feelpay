import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class ProviderConnectorsPage extends AdminShellPage {
  readonly pageTitle = 'Provider Connectors';
  readonly table = this.page.locator('table');
  readonly rows = this.table.locator('tbody tr');
  readonly runSandboxTestButton = this.page.getByRole('button', { name: 'Run sandbox test' });

  async goto() {
    await this.navigateTo('Provider Connectors');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  async runSandboxTest(providerName: string) {
    // Try finding in a card first (heading + button)
    const card = this.page.locator('.card, .p-4, .border', { has: this.page.getByRole('heading', { name: providerName }) }).first();
    const buttonInCard = card.getByRole('button', { name: 'Run sandbox test' });
    
    if (await buttonInCard.isVisible()) {
      await buttonInCard.click();
    } else {
      // Fallback to table row
      const row = this.rows.filter({ hasText: providerName });
      await row.getByRole('button', { name: 'Run sandbox test' }).click();
    }
    
    await this.waitForAlert(/Sandbox test successful|Sandbox test passed/i);
  }

  async expectProviderHealth(providerName: string, health: string | RegExp) {
    const row = this.rows.filter({ hasText: providerName });
    // Assuming health is in a specific cell, or just check row text
    await expect(row).toContainText(health);
  }

  async getProviderStatus(providerName: string) {
    const row = this.rows.filter({ hasText: providerName });
    // This is a bit brittle without stable column indices, but works if label is unique
    return row.locator('td').nth(4).innerText(); // Status column index might vary
  }
}
