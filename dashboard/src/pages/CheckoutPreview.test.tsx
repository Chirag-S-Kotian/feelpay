import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, test, vi } from 'vitest';
import CheckoutPreview from './CheckoutPreview';

const mockState = vi.hoisted(() => ({
  resolvePay: undefined as undefined | ((value: unknown) => void),
}));

vi.mock('../hooks/useApi', () => ({
  apiRequest: vi.fn(async (_method: string, path: string) => {
    if (path === '/v1/checkout/preview-config') {
      return {
        merchant: { name: 'Acme Store' },
        sample_order: {
          order_id: 'order_preview_001',
          amount: 249900,
          currency: 'INR',
          customer_email: 'buyer@example.com',
          description: 'Preview',
        },
        methods: [
          {
            method_type: 'upi',
            enabled: true,
            config: { display_name: 'UPI', checkout_note: 'Fast UPI' },
          },
        ],
        event_flow: [],
      };
    }
    if (path === '/v1/merchants/checkout-settings') return {};
    if (path === '/v1/merchants/checkout-settings/audit') return { items: [] };
    if (path === '/v1/checkout/session') {
      return {
        session_id: 'cs_test_123',
        checkout_url: 'http://checkout/cs_test_123',
        embed_url: 'http://checkout/embed/cs_test_123',
        expires_at: new Date().toISOString(),
      };
    }
    if (path === '/v1/checkout/session/cs_test_123/pay') {
      return new Promise((resolve) => {
        mockState.resolvePay = resolve;
      });
    }
    return {};
  }),
}));

beforeEach(() => {
  vi.spyOn(localStorage, 'getItem').mockImplementation((key: string) => {
    if (key === 'feelpay_merchant') return JSON.stringify({ id: '123e4567-e89b-12d3-a456-426614174999' });
    return null;
  });
});

test('runs preview payment through checkout session and shows captured test badge', async () => {
  render(<CheckoutPreview />);
  await waitFor(() => expect(screen.getAllByText('Acme Store').length).toBeGreaterThan(0));

  await userEvent.click(screen.getByRole('button', { name: /pay securely with cards/i }));

  expect(await screen.findByText(/processing cards/i)).toBeInTheDocument();
  mockState.resolvePay?.({
    session_id: 'cs_test_123',
    payment_id: '123e4567-e89b-12d3-a456-426614174000',
    status: 'paid',
    payment_status: 'captured',
    connector: 'simulator',
    payment_method: 'card',
    mode_badge: 'TEST',
  });

  await waitFor(() => expect(screen.getByText(/payment captured/i)).toBeInTheDocument());
  expect(screen.getByText('TEST')).toBeInTheDocument();
  expect(screen.getAllByText(/simulator/i).length).toBeGreaterThan(0);
});
