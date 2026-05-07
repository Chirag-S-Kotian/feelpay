import { expect } from '@playwright/test';
import { AdminShellPage } from './AdminShellPage';

export class RiskRulesPage extends AdminShellPage {
  readonly pageTitle = 'Risk Rules';

  async goto() {
    await this.navigateTo('Risk Rules');
    await expect(this.page.locator('h1')).toHaveText(this.pageTitle);
  }

  async createRule(rule: { name: string; condition: string; action: string }) {
    await this.page.getByRole('button', { name: /Add Rule/i }).click();
    await this.page.getByLabel(/Rule Name/i).fill(rule.name);
    await this.page.getByLabel(/Condition/i).fill(rule.condition);
    await this.page.getByLabel(/Action/i).selectOption(rule.action);
    await this.page.getByRole('button', { name: /Save Rule/i }).click();
    await this.waitForAlert(/Rule saved successfully/i);
  }

  async toggleRule(ruleName: string) {
    const row = this.page.getByRole('row', { name: ruleName });
    await row.getByRole('switch').click();
    await this.waitForAlert(/Rule updated/i);
  }
}
