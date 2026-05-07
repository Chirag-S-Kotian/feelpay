import { Page } from '@playwright/test';
import { expectNoSensitiveLeak as expectNoSensitiveLeakInPage } from './securityAssertions';

export function setupConsoleGuards(page: Page) {
  const forbiddenPatterns = [
    /CORS/i,
    /Failed to fetch/i,
    /Auth failed/i,
    /400 Bad Request/i,
    /404 Not Found/i,
    /500 Internal Server Error/i,
    /value prop on input should not be null/i,
    /controlled input to be uncontrolled/i,
    /uncontrolled input to be controlled/i,
    /PDF generator not configured/i,
    /webhook secret/i,
    /api secret/i,
    /raw card/i,
    /\bcvv\b/i,
    /\bcvc\b/i,
  ];

  page.on('console', msg => {
    const text = msg.text();
    const type = msg.type();

    if (text.includes('[vite] connected')) return;
    if (text.includes('React DevTools')) return;

    for (const pattern of forbiddenPatterns) {
      if (pattern.test(text)) {
        throw new Error(`FORBIDDEN CONSOLE ERROR DETECTED: [${type.toUpperCase()}] ${text}`);
      }
    }
  });

  page.on('pageerror', err => {
    throw new Error(`PAGE ERROR DETECTED: ${err.message}`);
  });
}

export async function expectNoSensitiveLeak(page: Page) {
  await expectNoSensitiveLeakInPage(page);
}
