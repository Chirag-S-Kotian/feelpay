import { useEffect, useMemo, useState } from 'react';
import { CheckCircle2, CreditCard, RefreshCw, Save, SlidersHorizontal, Smartphone, Wallet } from 'lucide-react';
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

const METHOD_META: Record<MethodKey, { title: string; description: string; icon: typeof CreditCard }> = {
  card: { title: 'Cards', description: 'Credit, debit, RuPay, Visa, Mastercard.', icon: CreditCard },
  upi: { title: 'UPI', description: 'Collect, intent, QR, and PSP app flows.', icon: Smartphone },
  netbanking: { title: 'Netbanking', description: 'Major Indian bank redirects.', icon: SlidersHorizontal },
  wallet: { title: 'Wallets', description: 'Consumer wallets and stored balances.', icon: Wallet },
  emi: { title: 'EMI', description: 'Credit/debit card EMI and no-cost plans.', icon: CreditCard },
  bnpl: { title: 'Pay Later', description: 'BNPL providers after merchant approval.', icon: Wallet },
};

const DEFAULT_METHODS: PaymentMethod[] = (Object.keys(METHOD_META) as MethodKey[]).map((method_type, index) => ({
  method_type,
  enabled: true,
  config: {
    priority: index + 1,
    display_name: METHOD_META[method_type].title,
    checkout_note: METHOD_META[method_type].description,
  },
}));

function normalize(methods: PaymentMethod[]) {
  const byKey = new Map(methods.map((method) => [method.method_type, method]));
  return DEFAULT_METHODS.map((fallback) => ({
    ...fallback,
    ...(byKey.get(fallback.method_type) || {}),
    config: {
      ...fallback.config,
      ...(byKey.get(fallback.method_type)?.config || {}),
    },
  }));
}

export default function PaymentMethods() {
  const [methods, setMethods] = useState<PaymentMethod[]>(DEFAULT_METHODS);
  const [loading, setLoading] = useState(true);
  const [savingKey, setSavingKey] = useState<MethodKey | null>(null);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');

  const enabledMethods = useMemo(
    () => methods.filter((method) => method.enabled).sort((a, b) => (a.config?.priority || 99) - (b.config?.priority || 99)),
    [methods],
  );

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      const data = await apiRequest<PaymentMethod[]>('GET', '/v1/merchants/payment-methods', undefined, undefined, { cacheMs: 5_000 });
      setMethods(normalize(data || []));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to load payment methods.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const patchMethod = (methodType: MethodKey, patch: Partial<PaymentMethod> & { config?: PaymentMethod['config'] }) => {
    setMethods((current) => current.map((method) => {
      if (method.method_type !== methodType) return method;
      return {
        ...method,
        ...patch,
        config: {
          ...(method.config || {}),
          ...(patch.config || {}),
        },
      };
    }));
  };

  const saveMethod = async (method: PaymentMethod) => {
    setSavingKey(method.method_type);
    setNotice('');
    setError('');
    try {
      await apiRequest('PUT', `/v1/merchants/payment-methods/${method.method_type}`, {
        enabled: method.enabled,
        config: method.config || {},
      });
      setNotice(`${METHOD_META[method.method_type].title} settings saved.`);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to save payment method.');
    } finally {
      setSavingKey(null);
    }
  };

  return (
    <div className="fade-in payment-methods-page mc-page">
      <div className="page-header">
        <div>
          <h1>Payment Methods</h1>
          <p>Choose methods shown on checkout and tune display order.</p>
        </div>
        <button className="btn btn-secondary" onClick={() => void load()}>
          <RefreshCw size={16} /> Refresh
        </button>
      </div>

      {notice && <div className="card methods-alert success"><CheckCircle2 size={16} /> {notice}</div>}
      {error && <div className="card methods-alert danger">{error}</div>}

      <div className="methods-layout">
        <section className="methods-list">
          {loading ? (
            <div className="card mc-loader-card"><span className="fp-loader"></span> Loading payment methods...</div>
          ) : methods.map((method) => {
            const meta = METHOD_META[method.method_type];
            const Icon = meta.icon;
            return (
              <article key={method.method_type} className="card method-card">
                <div className="method-top">
                  <div className="method-title">
                    <span className="method-icon"><Icon size={18} /></span>
                    <div>
                      <h3>{meta.title}</h3>
                      <p>{meta.description}</p>
                    </div>
                  </div>
                  <label className="switch-row">
                    <input
                      type="checkbox"
                      checked={method.enabled}
                      onChange={(e) => patchMethod(method.method_type, { enabled: e.target.checked })}
                    />
                    <span>{method.enabled ? 'Enabled' : 'Disabled'}</span>
                  </label>
                </div>

                <div className="method-fields">
                  <label>
                    Display name
                    <input
                      value={method.config?.display_name || ''}
                      onChange={(e) => patchMethod(method.method_type, { config: { display_name: e.target.value } })}
                    />
                  </label>
                  <label>
                    Priority
                    <input
                      type="number"
                      min="1"
                      value={method.config?.priority || 1}
                      onChange={(e) => patchMethod(method.method_type, { config: { priority: Number(e.target.value) } })}
                    />
                  </label>
                  <label className="method-span">
                    Checkout note
                    <input
                      value={method.config?.checkout_note || ''}
                      onChange={(e) => patchMethod(method.method_type, { config: { checkout_note: e.target.value } })}
                    />
                  </label>
                </div>

                <button className="btn btn-primary method-save" onClick={() => void saveMethod(method)} disabled={savingKey === method.method_type}>
                  <Save size={16} /> {savingKey === method.method_type ? 'Saving...' : 'Save Method'}
                </button>
              </article>
            );
          })}
        </section>

        <aside className="card checkout-order-card">
          <h3 className="section-title"><SlidersHorizontal size={17} /> Checkout Order</h3>
          <div className="checkout-order-list">
            {enabledMethods.map((method, index) => (
              <div className="checkout-order-item" key={method.method_type}>
                <strong>{index + 1}</strong>
                <span>{method.config?.display_name || METHOD_META[method.method_type].title}</span>
                <small>{method.method_type}</small>
              </div>
            ))}
          </div>
          <p className="mini-help">Disabled methods stay hidden from hosted checkout preview and merchant integrations.</p>
        </aside>
      </div>

      <style>{`
        .payment-methods-page .page-header {
          align-items: center;
          border: 1px solid #d5c9af;
          border-radius: 34px;
          padding: 26px;
          background: #fff9ec;
          box-shadow: 0 16px 34px rgba(20, 20, 19, 0.07);
        }
        .methods-layout { display: grid; grid-template-columns: minmax(0, 1fr) 340px; gap: 20px; align-items: start; }
        .methods-list { display: grid; grid-template-columns: repeat(2, minmax(260px, 1fr)); gap: 16px; }
        .method-card { display: flex; flex-direction: column; gap: 16px; min-height: 280px; background: #fff9ec !important; border-radius: 30px !important; box-shadow: 0 14px 30px rgba(20, 20, 19, 0.08) !important; }
        .method-top { display: flex; justify-content: space-between; gap: 16px; align-items: flex-start; }
        .method-title { display: flex; gap: 14px; align-items: flex-start; }
        .method-title h3 { margin: 0; font-size: 22px; line-height: 1.1; color: #312d28; }
        .method-title p { margin: 6px 0 0; color: #6b6862; font-size: 14px; line-height: 1.45; }
        .method-icon { width: 52px; height: 52px; border-radius: 999px; display: grid; place-items: center; color: #312d28; background: #f2ead8; border: 1px solid #d5c9af; flex: 0 0 auto; }
        .switch-row { display: inline-flex; align-items: center; gap: 8px; color: #6b6862; font-size: 13px; white-space: nowrap; }
        .switch-row input { width: 18px; height: 18px; accent-color: #eb001b; }
        .method-fields { display: grid; grid-template-columns: minmax(0, 1.2fr) 110px; gap: 12px; }
        .method-fields label { display: flex; flex-direction: column; gap: 6px; color: #6b6862; font-size: 12px; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; }
        .method-fields input { background: #fff9ec; border: 1px solid #d5c9af; color: #312d28; border-radius: 999px; padding: 12px 14px; box-shadow: none; }
        .method-fields input:focus { border-color: #eb001b; outline: 3px solid rgba(235, 0, 27, 0.13); }
        .method-span { grid-column: 1 / -1; }
        .methods-alert { margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }
        .methods-alert.success { border-color: rgba(35, 110, 62, 0.34); color: #236e3e; }
        .methods-alert.danger { border-color: rgba(200, 31, 47, 0.34); color: #a01525; }
        .method-save { margin-top: auto; width: 100%; justify-content: center; }
        .checkout-order-card { position: sticky; top: 20px; background: #f5efe2 !important; border-radius: 34px !important; }
        .checkout-order-card h3 { color: #312d28; }
        .checkout-order-list { display: grid; gap: 10px; margin-top: 12px; }
        .checkout-order-item { display: grid; grid-template-columns: 36px minmax(0, 1fr); gap: 6px 10px; padding: 13px; border: 1px solid #d5c9af; border-radius: 24px; background: #fff9ec; }
        .checkout-order-item strong { grid-row: span 2; width: 34px; height: 34px; border-radius: 999px; display: grid; place-items: center; background: #eb001b; color: #fff9ec; }
        .checkout-order-item small { color: #6b6862; text-transform: uppercase; font-size: 11px; letter-spacing: 0.06em; }
        @media (min-width: 1500px) { .methods-list { grid-template-columns: repeat(3, minmax(260px, 1fr)); } }
        @media (max-width: 1180px) { .methods-layout { grid-template-columns: 1fr; } .checkout-order-card { position: static; } }
        @media (max-width: 760px) { .methods-list { grid-template-columns: 1fr; } .method-top, .method-fields { grid-template-columns: 1fr; flex-direction: column; } }
      `}</style>
    </div>
  );
}
