import { expect, Page } from '@playwright/test';

const SENSITIVE_TEXT_PATTERNS = [
  /\bapi[_ -]?secret\s*[:=]\s*[a-z0-9._-]{12,}/i,
  /\bapi[_ -]?key\s*[:=]\s*(sk|pk|fp)_[a-z0-9_]{12,}/i,
  /\bwebhook[_ -]?(signing[_ -]?)?secret\s*[:=]\s*[a-z0-9._-]{12,}/i,
  /\bbearer\s+[a-z0-9._-]{20,}/i,
  /\bauthorization\s*:/i,
  /\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b/,
  /\b(?:cvv|cvc)\s*[:=]\s*[0-9]{3,4}\b/i,
  /\b(?:account|acct|bank)[^\n]{0,24}\b[0-9]{9,18}\b/i,
  /\bpostgres(?:ql)?:\/\/[^\s]+/i,
  /\bdatabase_url\b/i,
  /\bjwt[_ -]?secret\b/i,
  /-----BEGIN (?:RSA |EC |OPENSSH |)PRIVATE KEY-----/,
];

const FORBIDDEN_ADMIN_WORDING = [
  /\bmerchant-admin\b/i,
  /\bseller-admin\b/i,
  /\bvendor-admin\b/i,
];

const RAW_TECHNICAL_ERRORS = [
  /\bAuth failed\b/i,
  /\bFailed to fetch\b/i,
  /\bCORS\b/i,
  /\bPDF generator not configured\b/i,
  /\bwebhook-events\s+404\b/i,
  /\bwebhook-test\s+400\b/i,
  /\bRoute missing \(404\)/i,
  /\bBackend error \(5\d\d\)/i,
];

export async function expectNoSensitiveLeak(page: Page) {
  const bodyText = await page.locator('body').innerText();

  for (const pattern of SENSITIVE_TEXT_PATTERNS) {
    expect(bodyText, `Sensitive value leaked in page text: ${pattern}`).not.toMatch(pattern);
  }
}

export async function expectNoForbiddenAdminWording(page: Page) {
  const bodyText = await page.locator('body').innerText();

  for (const pattern of FORBIDDEN_ADMIN_WORDING) {
    expect(bodyText, `Forbidden admin wording shown: ${pattern}`).not.toMatch(pattern);
  }
}

export async function expectNoRawTechnicalError(page: Page) {
  const bodyText = await page.locator('body').innerText();

  for (const pattern of RAW_TECHNICAL_ERRORS) {
    expect(bodyText, `Raw technical error shown: ${pattern}`).not.toMatch(pattern);
  }
}

export async function expectSecurityCleanPage(page: Page) {
  await expectNoSensitiveLeak(page);
  await expectNoForbiddenAdminWording(page);
  await expectNoRawTechnicalError(page);
}
