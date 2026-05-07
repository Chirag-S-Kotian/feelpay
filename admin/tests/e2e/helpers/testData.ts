export const TEST_ADMIN_ID = 'cd317088-a9ef-495c-ab3e-2a88603c54ac';
export const TEST_BUSINESS_ACCOUNT_ID = '7a7ac2af-39bd-4eef-a599-b84f86eec401';

export function uniqueName(prefix: string) {
  return `${prefix} ${Date.now()}`;
}

export function sandboxBankDetails() {
  return {
    name: uniqueName('Sandbox Settlement Bank'),
    type: 'API / sandbox test',
    currency: 'INR',
    accountNumber: '****5678',
  };
}

export function paymentLinkPayload() {
  return {
    title: uniqueName('Sandbox Checkout'),
    amount: 1250,
    currency: 'INR',
    customer_email: 'sandbox-buyer@feelpay.local',
  };
}

export function webhookTestPayload() {
  return {
    event_type: 'payment.captured',
    endpoint_url: 'https://webhooks.feelpay.local/sandbox',
    note: 'Sandbox event from admin E2E',
  };
}

