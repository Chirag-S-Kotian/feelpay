import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class RiskRulesPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/risk-rules', navLabel: 'Risk Rules', heading: 'Risk Rules', primaryAction: /Simulator|Simulate risk decision/i });
  }

  async openSimulator() {
    await this.page.getByRole('button', { name: 'Simulator' }).click();
    await expect(this.page.getByRole('heading', { name: /Risk simulator/i })).toBeVisible();
  }
}

