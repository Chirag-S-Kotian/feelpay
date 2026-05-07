import { expect, type Page, type Route } from '@playwright/test';

type SandboxPayment = {
  id: string;
  order_id: string;
  amount: number;
  payment_method: string;
  connector: string;
  status: string;
  customer_email: string;
  created_at: string;
};

type PaymentLinkRow = {
  id: string;
  link_id: string;
  amount: number;
  currency: string;
  description: string;
  status: string;
  full_url: string;
  created_at: string;
  customer_email: string;
  view_count: number;
  attempt_count: number;
};

const merchant = {
  id: '00000000-0000-0000-0000-000000000001',
  name: 'Demo Merchant',
  email: 'merchant@example.com',
  api_key: 'fp_test_demo_key',
};

const nowIso = () => new Date().toISOString();

function json(route: Route, body: unknown, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });
}

function pathFromRoute(route: Route) {
  const url = new URL(route.request().url());
  return url.pathname.replace(/^\/api/, '') + url.search;
}

function hasAuth(route: Route) {
  const headers = route.request().headers();
  return Boolean(headers.authorization || headers['x-api-key']);
}

export async function installMerchantApi(page: Page) {
  const methodNames: Record<string, string> = {
    card: 'Cards',
    upi: 'UPI',
    wallet: 'Wallets',
    netbanking: 'Netbanking',
    emi: 'EMI',
    bnpl: 'Pay Later',
  };
  const methodState = ['card', 'upi', 'wallet', 'netbanking', 'emi', 'bnpl'].map((methodType, index) => ({
    method_type: methodType,
    enabled: true,
    config: {
      priority: index + 1,
      display_name: methodNames[methodType],
    },
  }));
  const payments: SandboxPayment[] = [
    {
      id: 'pay_seed_success',
      order_id: 'order_seed_1',
      amount: 249900,
      payment_method: 'upi',
      connector: 'simulator',
      status: 'captured',
      customer_email: 'buyer@example.com',
      created_at: nowIso(),
    },
  ];
  const paymentLinks: PaymentLinkRow[] = [];
  const payouts: Array<{
    id: string;
    amount: number;
    currency: string;
    status: string;
    method: string;
    bank_account_id: string;
    payout_reference: string;
    gateway_reference: string;
    created_at: string;
  }> = [];
  const webhookEvents = [
    { id: 'evt_seed', event_type: 'payment.captured', status: 'delivered', attempts: 1, created_at: nowIso(), response_code: 200 },
  ];
  const auditItems = [{ created_at: nowIso(), actor_id: merchant.id }];
  const bankAccounts = [
    { id: 'ba_test_1', account_holder_name: 'Demo Merchant', bank_name: 'HDFC Bank', account_number_masked: 'XXXXXX7890', ifsc_code: 'HDFC0001234', account_type: 'current', verification_status: 'verified', is_primary: true, created_at: nowIso(), verified_at: nowIso() },
  ];

  await page.context().route('**/{api/,}v1/**', async (route) => {
    const path = pathFromRoute(route);
    const method = route.request().method();
    const publicPath =
      path === '/v1/merchants/login' ||
      path === '/v1/merchants/register' ||
      path === '/v1/merchants/forgot-password' ||
      path.startsWith('/v1/public/');

    if (!publicPath && !hasAuth(route)) {
      return json(route, { message: 'Authentication required. Provide valid x-api-key or Bearer token' }, 401);
    }

    if (path === '/v1/merchants/login') {
      return json(route, { token: 'fake-token', merchant_id: merchant.id, name: merchant.name, email: merchant.email, api_key: merchant.api_key, expires_in: 86400 });
    }
    if (path === '/v1/merchants/logout') return json(route, { ok: true });
    if (path === '/v1/merchants/profile') return json(route, { ...merchant, business_name: 'Demo Store', business_type: 'individual', webhook_url: 'https://example.com/webhooks/feelpay' });
    if (path === '/v1/merchants/keys') return json(route, { api_key: merchant.api_key });
    if (path === '/v1/merchants/settings/security') return json(route, { mfa_enabled: false, settlement_cycle: 'T+2', per_transaction_limit: 100000, daily_limit: 1000000, monthly_limit: 10000000, allowed_payment_methods: {}, webhook_secret_hint: 'whsec_test_••••1234', updated_at: nowIso() });
    if (path === '/v1/merchants/team') return json(route, []);
    if (path === '/v1/merchants/account-settings-hub') return json(route, { sections: [] });
    if (path === '/v1/merchants/dashboard-signals') {
      return json(route, {
        announcements: [],
        ecosystem_health: [{ name: 'Sandbox Bank Rail', success_rate: 100, avg_latency_ms: 120, is_active: true, circuit_breaker_state: 'closed' }],
      });
    }
    if (path === '/v1/onboarding/status') return json(route, { status: 'verified', kyc_status: 'verified', bank_status: 'verified', live_enabled: false });
    if (path === '/v1/onboarding/limits') return json(route, { daily_limit: 1000000, monthly_limit: 10000000 });
    if (path === '/v1/onboarding/live-preflight') return json(route, { can_go_live: false, blockers: ['Live mode disabled in sandbox demo'] });
    if (path.includes('/v1/onboarding/') && method === 'POST') return json(route, { status: 'verified' });

    if (path.startsWith('/v1/analytics/summary')) {
      const successful = payments.filter((p) => p.status === 'captured').length;
      const totalVolume = payments.filter((p) => p.status === 'captured').reduce((sum, p) => sum + p.amount, 0) / 100;
      return json(route, { total_transactions: payments.length, total_volume: totalVolume, successful, failed: payments.filter((p) => p.status === 'failed').length, success_rate: payments.length ? Math.round((successful / payments.length) * 100) : 0 });
    }
    if (path.startsWith('/v1/analytics/volume')) return json(route, { chart: [{ day: 'Today', volume: payments.reduce((sum, p) => sum + p.amount, 0) / 100, count: payments.length }] });
    if (path.startsWith('/v1/analytics/alerts')) return json(route, { alerts: [] });
    if (path.startsWith('/v1/payments')) return json(route, { data: payments });

    if (path === '/v1/checkout/preview-config') {
      return json(route, {
        merchant: { name: merchant.name, business_name: 'Demo Store' },
        sample_order: { order_id: 'preview_order', amount: 249900, currency: 'INR', customer_email: 'buyer@example.com', description: 'Sandbox order' },
        methods: methodState,
        event_flow: [{ label: 'Create session', status: 'ready', detail: 'Sandbox checkout session is created.' }],
      });
    }
    if (path === '/v1/checkout/session') return json(route, { session_id: `cs_test_${Date.now()}` });
    if (path.includes('/v1/checkout/session/') && path.endsWith('/pay')) {
      const id = `pay_test_${payments.length + 1}`;
      payments.unshift({ id, order_id: `preview_${payments.length + 1}`, amount: 249900, payment_method: 'upi', connector: 'simulator', status: 'captured', customer_email: 'buyer@example.com', created_at: nowIso() });
      webhookEvents.unshift({ id: `evt_${id}`, event_type: 'payment.captured', status: 'delivered', attempts: 1, created_at: nowIso(), response_code: 200 });
      return json(route, { session_id: 'cs_test_paid', payment_id: id, payment_status: 'captured', connector: 'simulator', payment_method: 'upi', mode_badge: 'TEST' });
    }
    if (path === '/v1/merchants/checkout-settings') return json(route, {});
    if (path === '/v1/merchants/checkout-settings/audit') return json(route, { items: auditItems });
    if (path === '/v1/merchants/checkout-settings' && method === 'PUT') return json(route, { ok: true });
    if (path === '/v1/merchants/payment-methods') return json(route, methodState);
    if (path.startsWith('/v1/merchants/payment-methods/') && method === 'PUT') {
      const methodType = path.split('/').pop();
      const body = route.request().postDataJSON() as { enabled?: boolean; config?: { priority?: number; display_name?: string; checkout_note?: string } };
      const target = methodState.find((item) => item.method_type === methodType);
      if (target) {
        target.enabled = Boolean(body.enabled);
        target.config = { ...target.config, ...(body.config || {}) };
      }
      return json(route, { ok: true });
    }

    if (path === '/v1/payment-links' && method === 'GET') return json(route, { data: paymentLinks, meta: { page: 1, per_page: 20, total: paymentLinks.length, total_pages: 1 } });
    if (path === '/v1/payment-links' && method === 'POST') {
      const body = route.request().postDataJSON() as { amount?: number; description?: string; customer_email?: string };
      paymentLinks.unshift({ id: `plink_${paymentLinks.length + 1}`, link_id: `plink_${paymentLinks.length + 1}`, amount: body.amount || 10000, currency: 'INR', description: body.description || 'Sandbox payment link', status: 'active', full_url: `https://checkout.feelpay.test/pay/plink_${paymentLinks.length + 1}`, created_at: nowIso(), customer_email: body.customer_email || 'buyer@example.com', view_count: 0, attempt_count: 0 });
      return json(route, paymentLinks[0]);
    }
    if (path.includes('/v1/payment-links/') && path.endsWith('/cancel')) return json(route, { ok: true });

    if (path === '/v1/customers') return json(route, { data: [{ id: 'cus_test_1', email: 'buyer@example.com', phone: '+919999999999', total_paid: 249900, payment_count: payments.length, last_payment_at: nowIso(), created_at: nowIso() }] });
    if (path.startsWith('/v1/customers/') && path.endsWith('/payment-methods')) return json(route, []);
    if (path === '/v1/disputes') return json(route, { disputes: [] });
    if (path.startsWith('/v1/disputes/')) return json(route, { status: 'submitted' });

    if (path.startsWith('/v1/settlements?') || path === '/v1/settlements') return json(route, { settlements: [{ id: 'set_test_1', settlement_date: nowIso(), created_at: nowIso(), status: 'completed', amount: 249900, fee_amount: 4900, tax_amount: 882, net_amount: 244118, utr: 'UTRTEST123' }] });
    if (path === '/v1/settlements/balance') return json(route, { available: 244118, pending: 0 });
    if (path === '/v1/settlements/reconciliation') return json(route, { gross: 249900, fees: 4900, tax: 882, net: 244118, variance: 0, counts: { completed: 1, pending: 0, failed: 0 }, gross_amount: 249900, fee_amount: 4900, tax_amount: 882, net_amount: 244118, captured_count: payments.length, processed_count: payments.length, failed_count: 0 });
    if (path.startsWith('/v1/settlements/')) return json(route, {
      id: 'set_test_1',
      settlement_date: nowIso(),
      created_at: nowIso(),
      status: 'completed',
      amount: 249900,
      fee_amount: 4900,
      tax_amount: 882,
      net_amount: 244118,
      utr: 'UTRTEST123',
      bank_reference: 'IMPS-SANDBOX-SETTLEMENT',
      processed_at: nowIso(),
      items: payments.map((payment) => ({ transaction_id: payment.id, amount: payment.amount, fee: 4900 })),
    });
    if (path === '/v1/payouts' && method === 'GET') return json(route, payouts);
    if (path === '/v1/payouts' && method === 'POST') {
      const body = route.request().postDataJSON() as { amount?: number; method?: string; bank_account_id?: string };
      const bankAccount = bankAccounts.find((account) => account.id === body.bank_account_id);
      if (!bankAccount || bankAccount.verification_status !== 'verified') {
        return json(route, { message: 'Verified bank account required before payout.' }, 400);
      }
      payouts.unshift({
        id: `po_test_${payouts.length + 1}`,
        amount: body.amount || 0,
        currency: 'INR',
        status: 'processed',
        method: body.method || 'imps',
        bank_account_id: bankAccount.id,
        payout_reference: `POUTREF${payouts.length + 1}`,
        gateway_reference: `IMPS-SANDBOX-${payouts.length + 1}`,
        created_at: nowIso(),
      });
      return json(route, payouts[0]);
    }
    if (path === '/v1/payouts/bank-accounts') return json(route, bankAccounts);
    if (path.includes('/bank-accounts')) return json(route, { bank_accounts: bankAccounts });

    if (path === '/v1/developers/webhooks') return json(route, { events: webhookEvents, webhooks: [{ id: 'wh_test_1', url: 'https://example.com/webhooks/feelpay', status: 'active', event_subscriptions: ['payment.success', 'payment.failed', 'settlement.processed'] }] });
    if (path.startsWith('/v1/developers/api-logs') || path === '/v1/developers/request-logs') return json(route, { logs: [{ id: 'req_test_1', method: 'GET', path: '/v1/payments', endpoint: '/v1/payments', status: '200', latency_ms: 45, request_id: 'req_test_1', created_at: nowIso(), payload: { card: '[redacted]' } }] });
    if (path.includes('/webhooks/test')) {
      webhookEvents.unshift({ id: `evt_test_${webhookEvents.length + 1}`, event_type: 'payment.success', status: 'delivered', attempts: 1, created_at: nowIso(), response_code: 200 });
      return json(route, { test_id: 'wh_test_delivery_1', status: 'sent', http_status: 200, response_time_ms: 42, response_body: 'ok' });
    }
    if (path.includes('/webhooks/replay')) {
      webhookEvents.unshift({ id: `evt_replay_${webhookEvents.length + 1}`, event_type: 'payment.success', status: 'replayed', attempts: 2, created_at: nowIso(), response_code: 200 });
      return json(route, { status: 'delivered', attempts: 2, response_code: 200 });
    }
    if (path.startsWith('/v1/merchants/') && path.includes('/exports') && route.request().method() === 'GET') return json(route, { exports: [{ job_id: 'exp_test_1', job_type: 'transactions', status: 'completed', format: 'csv', record_count: payments.length, created_at: nowIso(), completed_at: nowIso(), download_url: '/downloads/exp_test_1.csv' }] });
    if (path.startsWith('/v1/merchants/') && path.includes('/exports')) return json(route, { job_id: 'exp_test_1', status: 'completed', download_url: '/downloads/exp_test_1.csv' });
    if (path.startsWith('/v1/exports')) return json(route, { exports: [{ id: 'exp_test_1', export_type: 'transactions', status: 'ready', format: 'csv', row_count: payments.length, download_url: '/downloads/exp_test_1.csv', created_at: nowIso() }] });
    if (path.includes('/scheduled-reports')) return json(route, { reports: [{ id: 'sched_test_1', report_type: 'settlement', frequency: 'daily', time_of_day: '09:00', recipient_emails: [merchant.email], format: 'csv', is_active: true, created_at: nowIso(), last_run_at: nowIso(), next_run_at: nowIso() }] });

    return json(route, {});
  });
}

export async function loginMerchant(page: Page) {
  await page.goto('/login');
  await page.getByPlaceholder('merchant@business.com').fill(merchant.email);
  await page.getByPlaceholder('••••••••').fill('StrongPass123!');
  await page.getByRole('button', { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/$/);
}

export async function seedMerchantSession(page: Page) {
  await page.addInitScript((seed) => {
    localStorage.setItem('feelpay_token', seed.token);
    localStorage.setItem('feelpay_session_expires_at', String(Date.now() + 86_400_000));
    localStorage.setItem('feelpay_merchant', JSON.stringify(seed.merchant));
  }, { token: 'fake-token', merchant });
}
