import { useEffect, useMemo, useState } from 'react';
import { CreditCard, Monitor, ShieldCheck, Smartphone, WalletCards } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type MethodKey = 'card' | 'upi' | 'netbanking' | 'wallet' | 'emi' | 'bnpl';

type PaymentMethod = {
  method_type: MethodKey;
  enabled: boolean;
  config?: {
    priority?: number;
    display_name?: string;
    checkout_note?: string;
  } | null;
};

type PreviewConfig = {
  merchant?: { name?: string; business_name?: string | null };
  sample_order?: {
    order_id: string;
    amount: number;
    currency: string;
    customer_email: string;
    description: string;
  };
  methods?: PaymentMethod[];
  event_flow?: Array<{ label: string; status: string; detail: string }>;
};

const LABELS: Record<MethodKey, string> = {
  card: 'Cards',
  upi: 'UPI',
  netbanking: 'Netbanking',
  wallet: 'Wallets',
  emi: 'EMI',
  bnpl: 'Pay Later',
};

const ICONS: Record<MethodKey, typeof CreditCard> = {
  card: CreditCard,
  upi: Smartphone,
  netbanking: ShieldCheck,
  wallet: WalletCards,
  emi: CreditCard,
  bnpl: WalletCards,
};

const FALLBACK_METHODS: PaymentMethod[] = (Object.keys(LABELS) as MethodKey[]).map((method_type, index) => ({
  method_type,
  enabled: true,
  config: {
    priority: index + 1,
    display_name: LABELS[method_type],
    checkout_note: 'Available on FeelPay hosted checkout.',
  },
}));

function normalize(methods: PaymentMethod[]) {
  const incoming = new Map(methods.map((method) => [method.method_type, method]));
  return FALLBACK_METHODS.map((fallback) => ({
    ...fallback,
    ...(incoming.get(fallback.method_type) || {}),
    config: {
      ...fallback.config,
      ...(incoming.get(fallback.method_type)?.config || {}),
    },
  }));
}

export default function CheckoutPreview() {
  const [viewport, setViewport] = useState<'desktop' | 'mobile'>('desktop');
  const [amount, setAmount] = useState('2499');
  const [customer, setCustomer] = useState('customer@example.com');
  const [merchantName, setMerchantName] = useState('FeelPay Merchant');
  const [merchantTagline, setMerchantTagline] = useState('Payments that build trust');
  const [eventFlow, setEventFlow] = useState<PreviewConfig['event_flow']>([]);
  const [methods, setMethods] = useState<PaymentMethod[]>(FALLBACK_METHODS);
  const [selected, setSelected] = useState<MethodKey>('upi');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await apiRequest<PreviewConfig>('GET', '/v1/checkout/preview-config', undefined, undefined, { cacheMs: 20_000 });
        const normalized = normalize(data.methods || []);
        setMethods(normalized);
        setMerchantName(data.merchant?.business_name || data.merchant?.name || 'FeelPay Merchant');
        setMerchantTagline('Payments that build trust');
        if (data.sample_order?.amount) setAmount(String(data.sample_order.amount / 100));
        if (data.sample_order?.customer_email) setCustomer(data.sample_order.customer_email);
        setEventFlow(data.event_flow || []);
        const first = normalized.find((method) => method.enabled)?.method_type;
        if (first) setSelected(first);
      } catch {
        setMethods(FALLBACK_METHODS);
      } finally {
        setLoading(false);
      }
    };
    void load();
  }, []);

  const visibleMethods = useMemo(
    () => methods.filter((method) => method.enabled).sort((a, b) => (a.config?.priority || 99) - (b.config?.priority || 99)),
    [methods],
  );
  const selectedMethod = visibleMethods.find((method) => method.method_type === selected) || visibleMethods[0];

  return (
    <div className="fade-in checkout-preview-page mc-page">
      <div className="page-header">
        <div>
          <h1>Checkout Preview</h1>
          <p>Preview hosted payment experience across website and mobile layouts.</p>
        </div>
        <div className="pill-tabs">
          <button className={`pill-tab ${viewport === 'desktop' ? 'active' : ''}`} onClick={() => setViewport('desktop')}>
            <Monitor size={15} /> Website
          </button>
          <button className={`pill-tab ${viewport === 'mobile' ? 'active' : ''}`} onClick={() => setViewport('mobile')}>
            <Smartphone size={15} /> Mobile
          </button>
        </div>
      </div>

      {loading && <div className="card mc-loader-card"><span className="fp-loader"></span> Loading checkout configuration...</div>}

      <div className="preview-shell">
        <section className="card preview-controls">
          <h3>Preview Data</h3>
          <div className="preview-merchant-name">{merchantName}</div>
          <label>
            Amount
            <input value={amount} onChange={(e) => setAmount(e.target.value)} type="number" min="1" />
          </label>
          <label>
            Customer
            <input value={customer} onChange={(e) => setCustomer(e.target.value)} type="email" />
          </label>
          <div className="preview-method-list">
              <span>Visible methods from merchant settings</span>
            {visibleMethods.map((method) => (
              <button
                type="button"
                key={method.method_type}
                className={selectedMethod?.method_type === method.method_type ? 'selected' : ''}
                onClick={() => setSelected(method.method_type)}
              >
                {method.config?.display_name || LABELS[method.method_type]}
              </button>
            ))}
          </div>
        </section>

        <section className={`checkout-stage ${viewport}`}>
          <div className="browser-bar">
            <span></span><span></span><span></span>
            <strong>checkout.feelpay.in</strong>
          </div>
          <div className="checkout-card">
            <div className="checkout-brand">
              <img src="/feelpay.png" alt="FeelPay" />
              <div>
                <strong>{merchantName}</strong>
                <span>{merchantTagline}</span>
              </div>
            </div>

            <div className="checkout-amount">
              <span>Amount payable</span>
              <strong>₹{Number(amount || 0).toLocaleString('en-IN')}</strong>
              <small>{customer}</small>
            </div>

            <div className="checkout-methods">
              {visibleMethods.map((method) => {
                const Icon = ICONS[method.method_type];
                const active = selectedMethod?.method_type === method.method_type;
                return (
                  <button key={method.method_type} className={active ? 'active' : ''} onClick={() => setSelected(method.method_type)}>
                    <Icon size={18} />
                    <span>{method.config?.display_name || LABELS[method.method_type]}</span>
                    <small>{method.config?.checkout_note}</small>
                  </button>
                );
              })}
            </div>

            <button className="checkout-pay-button">
              Pay with {selectedMethod?.config?.display_name || 'selected method'}
            </button>
            <div className="checkout-secure">
              <ShieldCheck size={14} />
              PCI-ready card vault, signed webhooks, live/test isolation
            </div>
          </div>
        </section>
      </div>

      <section className="card payment-flow-panel">
        <h3>How payment works</h3>
        <div className="payment-flow-grid">
          {(eventFlow || []).map((step, index) => (
            <article key={`${step.status}-${step.label}`}>
              <strong>{String(index + 1).padStart(2, '0')}</strong>
              <div>
                <h4>{step.label}</h4>
                <p>{step.detail}</p>
                <span>{step.status}</span>
              </div>
            </article>
          ))}
        </div>
      </section>

      <style>{`
        .checkout-preview-page .page-header {
          border: 1px solid #d5c9af;
          border-radius: 34px;
          background: #fff9ec;
          padding: 24px;
        }
        .preview-shell { display: grid; grid-template-columns: 330px minmax(0, 1fr); gap: 20px; align-items: start; margin-top: 16px; }
        .preview-controls { display: grid; gap: 14px; position: sticky; top: 20px; background: #fff9ec !important; border-radius: 30px !important; }
        .preview-controls h3 { margin: 0; font-size: 22px; }
        .preview-merchant-name { color: #312d28; background: #f5efe2; border: 1px solid #d5c9af; border-radius: 999px; padding: 12px 14px; font-weight: 800; font-size: 18px; }
        .preview-controls label { display: flex; flex-direction: column; gap: 6px; color: var(--fp-text-muted); font-size: 13px; }
        .preview-controls input { background: #fff9ec; border: 1px solid #d5c9af; color: #141413; border-radius: 999px; padding: 10px 12px; }
        .preview-method-list { display: grid; gap: 8px; }
        .preview-method-list span { color: var(--fp-text-muted); font-size: 13px; }
        .preview-method-list button { text-align: left; background: #fff9ec; border: 1px solid #d5c9af; color: #6b6862; border-radius: 999px; padding: 9px 12px; cursor: pointer; }
        .preview-method-list button.selected { border-color: #eb001b; color: #141413; background: #f5efe2; }
        .checkout-stage { min-height: 720px; border: 1px solid #d5c9af; border-radius: 34px; background: #f5efe2; padding: 24px; display: flex; flex-direction: column; align-items: center; overflow: hidden; }
        .checkout-stage.desktop { width: 100%; }
        .checkout-stage.mobile { width: 390px; min-height: 812px; margin: 0 auto; padding: 14px; border-radius: 34px; }
        .browser-bar { width: 100%; max-width: 820px; height: 38px; border-radius: 28px 28px 0 0; background: #fff9ec; display: flex; align-items: center; gap: 7px; padding: 0 12px; color: #141413; }
        .browser-bar span { width: 9px; height: 9px; border-radius: 99px; background: #6b6862; }
        .browser-bar strong { margin-left: 10px; font-size: 12px; font-weight: 600; }
        .checkout-stage.mobile .browser-bar, .checkout-stage.mobile .checkout-card { max-width: 344px; }
        .checkout-card { width: 100%; max-width: 820px; min-height: 600px; background: #fff9ec; color: #141413; border: 1px solid #d5c9af; border-top: none; border-radius: 0 0 28px 28px; padding: 28px; display: grid; gap: 18px; align-content: start; box-shadow: none; }
        .checkout-stage.mobile .checkout-card { min-height: 720px; padding: 18px; gap: 14px; }
        .checkout-brand { display: flex; align-items: center; gap: 14px; }
        .checkout-brand img { width: 62px; height: 62px; border-radius: 20px; object-fit: contain; border: 1px solid #d5c9af; background: #fff; padding: 4px; }
        .checkout-stage.mobile .checkout-brand img { width: 54px; height: 54px; border-radius: 18px; }
        .checkout-brand div { display: flex; flex-direction: column; line-height: 1.2; }
        .checkout-brand strong { font-size: 20px; color: #312d28; }
        .checkout-stage.mobile .checkout-brand strong { font-size: 17px; }
        .checkout-brand span { color: #6b6862; font-size: 12px; text-transform: uppercase; letter-spacing: 0.08em; }
        .checkout-amount { border: 1px solid #d5c9af; border-radius: 28px; padding: 16px; display: grid; gap: 2px; background: #f5efe2; }
        .checkout-amount span, .checkout-amount small { color: #6b6862; }
        .checkout-amount strong { font-size: 34px; line-height: 1.1; }
        .checkout-methods { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
        .checkout-stage.mobile .checkout-methods { grid-template-columns: 1fr; }
        .checkout-methods button { display: grid; grid-template-columns: 24px minmax(0, 1fr); gap: 2px 10px; text-align: left; border: 1px solid #d5c9af; background: #fff9ec; color: #141413; border-radius: 28px; padding: 14px; cursor: pointer; min-height: 70px; }
        .checkout-methods button.active { border-color: #eb001b; background: #f5efe2; box-shadow: inset 0 0 0 1px #eb001b; }
        .checkout-methods small { grid-column: 2; color: #6b6862; line-height: 1.35; }
        .checkout-pay-button { border: none; border-radius: 999px; padding: 14px 18px; font-weight: 800; background: #eb001b; color: #fff9ec; cursor: pointer; min-height: 50px; }
        .checkout-secure { display: inline-flex; gap: 7px; align-items: center; justify-content: center; color: #6b6862; font-size: 12px; }
        .payment-flow-panel { margin-top: 18px; }
        .payment-flow-panel h3 { margin-bottom: 14px; }
        .payment-flow-grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 10px; }
        .payment-flow-grid article { border: 1px solid #d5c9af; background: #fff9ec; color: #141413; border-radius: 28px; padding: 14px; display: grid; gap: 10px; }
        .payment-flow-grid article > strong { color: #eb001b; font-size: 13px; }
        .payment-flow-grid h4 { margin: 0 0 4px; font-size: 14px; }
        .payment-flow-grid p { margin: 0; color: #6b6862; font-size: 12px; line-height: 1.45; }
        .payment-flow-grid span { display: inline-flex; margin-top: 10px; border: 1px solid #d5c9af; border-radius: 999px; padding: 3px 8px; color: #6b6862; font-size: 11px; }
        @media (max-width: 980px) { .preview-shell { grid-template-columns: 1fr; } .preview-controls { position: static; } .checkout-stage { min-height: auto; } }
        @media (max-width: 430px) { .checkout-stage.mobile { width: 100%; } }
        @media (max-width: 980px) { .payment-flow-grid { grid-template-columns: 1fr; } }
      `}</style>
    </div>
  );
}
