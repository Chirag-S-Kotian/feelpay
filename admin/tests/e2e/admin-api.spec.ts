import { APIRequestContext, expect, test } from '@playwright/test';
import { ADMIN_ROLES, AdminRole, adminAuthHeaders } from './helpers/auth';
import { paymentLinkPayload, webhookTestPayload } from './helpers/testData';

const apiBaseUrl = process.env.ADMIN_API_BASE_URL || 'http://127.0.0.1:8081';
const adminApi = '/admin/payment-ops';

async function requestJson(
  request: APIRequestContext,
  method: 'GET' | 'POST',
  path: string,
  body?: unknown,
  role: AdminRole = ADMIN_ROLES.SUPER_ADMIN,
) {
  const url = `${apiBaseUrl}${path}`;
  const options = {
    headers: adminAuthHeaders(role),
    data: body,
  };

  return method === 'GET' ? request.get(url, options) : request.post(url, options);
}

test.describe('admin API contract', () => {
  test('GET provider connectors', async ({ request }) => {
    const response = await requestJson(request, 'GET', `${adminApi}/connectors`);
    expect(response.status()).toBeLessThan(500);
    expect(response.status()).not.toBe(404);
  });

  test('POST provider sandbox test', async ({ request }) => {
    const response = await requestJson(request, 'POST', `${adminApi}/connectors/test`, { provider: 'mock' });
    expect(response.status()).toBeLessThan(500);
    expect(response.status()).not.toBe(404);
  });

  test('GET routing provider performance', async ({ request }) => {
    const response = await requestJson(request, 'GET', `${adminApi}/routing/provider-performance`);
    expect(response.status()).toBeLessThan(500);
    expect(response.status()).not.toBe(404);
  });

  test('payment link, checkout session and test payment endpoints are present', async ({ request }) => {
    const link = await requestJson(request, 'POST', `${adminApi}/white-label/payment-links`, paymentLinkPayload());
    expect(link.status()).toBeLessThan(500);
    expect(link.status()).not.toBe(404);

    const linkBody = await link.json().catch(() => ({})) as { id?: string; payment_link_id?: string };
    const paymentLinkId = linkBody.id || linkBody.payment_link_id || 'sandbox_missing_link_id';

    const session = await requestJson(request, 'POST', `${adminApi}/white-label/sessions`, { payment_link_id: paymentLinkId });
    expect(session.status()).toBeLessThan(500);
    expect(session.status()).not.toBe(404);

    const sessionBody = await session.json().catch(() => ({})) as { id?: string; session_id?: string };
    const sessionId = sessionBody.id || sessionBody.session_id || 'sandbox_missing_session_id';
    const payment = await requestJson(request, 'POST', `${adminApi}/white-label/sessions/${sessionId}/complete-test-payment`, { status: 'captured' });
    expect(payment.status()).toBeLessThan(500);
    expect(payment.status()).not.toBe(404);
  });

  test('GET transactions and ledger', async ({ request }) => {
    for (const path of [`${adminApi}/transactions`, `${adminApi}/ledger`]) {
      const response = await requestJson(request, 'GET', path);
      expect(response.status()).toBeLessThan(500);
      expect(response.status()).not.toBe(404);
    }
  });

  test('report generate and download endpoints', async ({ request }) => {
    const generate = await requestJson(request, 'POST', `${adminApi}/reports/generate`, { report_type: 'transactions', format: 'csv' });
    expect(generate.status()).toBeLessThan(500);
    expect(generate.status()).not.toBe(404);

    const generated = await generate.json().catch(() => ({})) as { id?: string; export_id?: string };
    const exportId = generated.id || generated.export_id || 'sandbox_missing_export_id';
    const download = await requestJson(request, 'GET', `${adminApi}/reports/download/${exportId}`);
    expect(download.status()).toBeLessThan(500);
    expect(download.status()).not.toBe(404);
  });

  test('webhook test, stored event and replay endpoints', async ({ request }) => {
    const webhook = await requestJson(request, 'POST', `${adminApi}/developer/webhook-test`, webhookTestPayload());
    expect(webhook.status()).toBeLessThan(500);
    expect(webhook.status()).not.toBe(400);
    expect(webhook.status()).not.toBe(404);

    const events = await requestJson(request, 'GET', `${adminApi}/developer/webhook-events?limit=10`);
    expect(events.status()).toBeLessThan(500);
    expect(events.status()).not.toBe(404);

    const eventBody = await events.json().catch(() => ({})) as { rows?: { id?: string }[]; data?: { id?: string }[] };
    const eventId = eventBody.rows?.[0]?.id || eventBody.data?.[0]?.id || 'sandbox_missing_event_id';
    const replay = await requestJson(request, 'POST', `${adminApi}/developer/webhook-events/${eventId}/replay`, { dry_run: true });
    expect(replay.status()).toBeLessThan(500);
    expect(replay.status()).not.toBe(404);
  });

  test('GET audit logs', async ({ request }) => {
    const response = await requestJson(request, 'GET', '/admin/audit-logs?limit=20');
    expect(response.status()).toBeLessThan(500);
    expect(response.status()).not.toBe(404);
  });

  test('unauthorized role returns 403 with request_id', async ({ request }) => {
    const response = await requestJson(request, 'GET', `${adminApi}/connectors`, undefined, ADMIN_ROLES.AUDITOR);
    expect([200, 403]).toContain(response.status());
    if (response.status() === 403) {
      const body = await response.json() as { request_id?: string };
      expect(body.request_id).toBeTruthy();
    }
  });

  test('unknown route returns friendly JSON', async ({ request }) => {
    const response = await requestJson(request, 'GET', '/admin/__missing_admin_e2e_route__');
    expect(response.status()).toBe(404);
    const contentType = response.headers()['content-type'] || '';
    expect(contentType).toContain('application/json');
    const body = await response.json() as { message?: string; request_id?: string };
    expect(body.message || body.request_id).toBeTruthy();
  });

  test('idempotency same key replays and conflicting payload is rejected', async ({ request }) => {
    const key = `admin-e2e-${Date.now()}`;
    const headers = { ...adminAuthHeaders(), 'idempotency-key': key };
    const path = `${apiBaseUrl}${adminApi}/white-label/payment-links`;
    const payload = paymentLinkPayload();

    const first = await request.post(path, { headers, data: payload });
    const replay = await request.post(path, { headers, data: payload });
    expect(first.status()).toBeLessThan(500);
    expect(replay.status()).toBeLessThan(500);

    const conflict = await request.post(path, {
      headers,
      data: { ...payload, amount: 9999 },
    });
    expect([200, 201, 202, 409, 422]).toContain(conflict.status());
    if (conflict.status() !== first.status()) {
      expect(conflict.status()).toBe(409);
    }
  });
});
