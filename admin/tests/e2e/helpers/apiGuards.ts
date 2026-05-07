import { Page } from '@playwright/test';

export function setupApiGuards(page: Page) {
  page.on('response', response => {
    const status = response.status();
    const url = response.url();

    if (/\.(js|css|png|svg|ico|woff2?|map)(\?|$)/.test(url)) return;

    if (status >= 500) {
      throw new Error(`SERVER ERROR DETECTED: [${status}] ${url}`);
    }

    if (status === 400 && /webhook-test/.test(url)) {
      throw new Error(`WEBHOOK TEST BAD REQUEST: [${status}] ${url}`);
    }

    if (status === 404 && /\/admin\/payment-ops\/.*(?:webhook-events|webhook-test|ledger|reports|transactions)/.test(url)) {
      throw new Error(`CORE ADMIN API ROUTE MISSING: [${status}] ${url}`);
    }
  });
}
