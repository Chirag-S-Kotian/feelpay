export type FeelPayConfig = {
  apiKey: string;
  baseUrl?: string;
  apiVersion?: string;
};

export type CreatePaymentRequest = {
  amount: number;
  currency?: string;
  order_id: string;
  customer_email?: string;
  customer_phone?: string;
  description?: string;
  return_url?: string;
  method_data: unknown;
};

export class FeelPay {
  private baseUrl: string;

  constructor(private config: FeelPayConfig) {
    this.baseUrl = config.baseUrl ?? 'https://api.feelpay.in/v1';
  }

  async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: {
        'content-type': 'application/json',
        'x-api-key': this.config.apiKey,
        'x-api-version': this.config.apiVersion ?? '2026-04-28'
      },
      body: body === undefined ? undefined : JSON.stringify(body)
    });
    const text = await res.text();
    const data = text ? JSON.parse(text) : null;
    if (!res.ok) throw new Error(data?.message ?? `FeelPay API error ${res.status}`);
    return data as T;
  }

  createPayment(payload: CreatePaymentRequest) {
    return this.request('POST', '/payments', payload);
  }

  getPayment(id: string) {
    return this.request('GET', `/payments/${id}`);
  }

  listPayments(params = new URLSearchParams()) {
    const qs = params.toString();
    return this.request('GET', `/payments${qs ? `?${qs}` : ''}`);
  }
}
