import { expect, Page } from '@playwright/test';
import { GenericAdminPage } from './GenericAdminPage';

export class WhiteLabelCheckoutPage extends GenericAdminPage {
  constructor(page: Page) {
    super(page, { path: '/white-label-checkout', navLabel: 'White-label Checkout', heading: 'White-label Checkout', primaryAction: /Save changes|Save configuration/i });
  }

  async openPreview() {
    await this.page.getByRole('button', { name: 'Preview' }).click();
    await expect(this.page.getByRole('heading', { name: /Checkout preview/i })).toBeVisible();
  }
}

