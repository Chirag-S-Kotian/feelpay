import { useEffect, useMemo, useState } from 'react';
import {
  Banknote,
  CheckCircle2,
  CreditCard,
  Globe2,
  Laptop,
  LoaderCircle,
  QrCode,
  ShieldCheck,
  Smartphone,
  WalletCards,
} from 'lucide-react';
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

type CheckoutSettings = {
  styling?: {
    background_color?: string;
    border_style?: 'rounded' | 'sharp';
    font_family?: string;
    show_sidebar_graphic?: boolean;
    show_trusted_badge?: boolean;
  };
  features?: {
    show_mandate_summary?: boolean;
    default_language?: string;
    collect_email?: boolean;
    show_message_banner?: boolean;
    flash_checkout?: boolean;
  };
  payment_configuration?: {
    preset_name?: string;
    allow_custom_configuration?: boolean;
  };
};

type CheckoutAuditItem = {
  created_at: string;
  actor_id?: string | null;
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
  const [viewport, setViewport] = useState<'browser' | 'laptop' | 'mobile'>('browser');
  const [amount, setAmount] = useState('2499');
  const [customer, setCustomer] = useState('customer@example.com');
  const [merchantName, setMerchantName] = useState('FeelPay Merchant');
  const [merchantTagline, setMerchantTagline] = useState('Payments that build trust');
  const [eventFlow, setEventFlow] = useState<PreviewConfig['event_flow']>([]);
  const [methods, setMethods] = useState<PaymentMethod[]>(FALLBACK_METHODS);
  const [selected, setSelected] = useState<MethodKey>('upi');
  const [loading, setLoading] = useState(true);
  const [paymentState, setPaymentState] = useState<'idle' | 'processing' | 'success'>('idle');
  const [settingsTab, setSettingsTab] = useState<'styling' | 'features' | 'payment_configuration'>('styling');
  const [settings, setSettings] = useState<CheckoutSettings>({
    styling: { background_color: '#C63201', border_style: 'rounded', font_family: 'IBM Plex Sans', show_sidebar_graphic: false, show_trusted_badge: true },
    features: { show_mandate_summary: false, default_language: 'English', collect_email: false, show_message_banner: false, flash_checkout: true },
    payment_configuration: { preset_name: 'Standard payment options', allow_custom_configuration: true },
  });
  const [settingsNotice, setSettingsNotice] = useState('');
  const [settingsSaving, setSettingsSaving] = useState(false);
  const [auditItems, setAuditItems] = useState<CheckoutAuditItem[]>([]);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await apiRequest<PreviewConfig>('GET', '/v1/checkout/preview-config', undefined, undefined, { cacheMs: 20_000 });
        const checkoutSettings: CheckoutSettings = await apiRequest<CheckoutSettings>('GET', '/v1/merchants/checkout-settings', undefined, undefined, { cacheMs: 20_000 }).catch(() => ({} as CheckoutSettings));
        const auditRes = await apiRequest<{ items?: CheckoutAuditItem[] }>('GET', '/v1/merchants/checkout-settings/audit', undefined, undefined, { cacheMs: 10_000 }).catch(() => ({ items: [] }));
        const normalized = normalize(data.methods || []);
        setMethods(normalized);
        setMerchantName(data.merchant?.business_name || data.merchant?.name || 'FeelPay Merchant');
        setMerchantTagline('Payments that build trust');
        if (data.sample_order?.amount) setAmount(String(data.sample_order.amount / 100));
        if (data.sample_order?.customer_email) setCustomer(data.sample_order.customer_email);
        setEventFlow(data.event_flow || []);
        setSettings((prev) => ({
          styling: { ...prev.styling, ...(checkoutSettings.styling || {}) },
          features: { ...prev.features, ...(checkoutSettings.features || {}) },
          payment_configuration: { ...prev.payment_configuration, ...(checkoutSettings.payment_configuration || {}) },
        }));
        setAuditItems(auditRes.items || []);
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
  const selectedLabel = selectedMethod?.config?.display_name || LABELS[selectedMethod?.method_type || 'upi'];

  const selectMethod = (methodType: MethodKey) => {
    setSelected(methodType);
    setPaymentState('idle');
  };

  const runPreviewPayment = () => {
    setPaymentState('processing');
    window.setTimeout(() => setPaymentState('success'), 900);
  };

  const saveCheckoutSettings = async () => {
    if ((settings.styling?.background_color || '').trim() && !/^#([0-9A-Fa-f]{6})$/.test((settings.styling?.background_color || '').trim())) {
      setSettingsNotice('Background color must be hex like #C63201');
      return;
    }
    if (!(settings.payment_configuration?.preset_name || '').trim()) {
      setSettingsNotice('Preset name required.');
      return;
    }
    setSettingsSaving(true);
    setSettingsNotice('');
    try {
      await apiRequest('PUT', '/v1/merchants/checkout-settings', settings);
      setSettingsNotice('Checkout settings saved.');
      const auditRes = await apiRequest<{ items?: CheckoutAuditItem[] }>('GET', '/v1/merchants/checkout-settings/audit', undefined, undefined, { cacheMs: 0 }).catch(() => ({ items: [] }));
      setAuditItems(auditRes.items || []);
    } catch (err) {
      setSettingsNotice(err instanceof Error ? err.message : 'Unable to save checkout settings.');
    } finally {
      setSettingsSaving(false);
    }
  };

  const renderMethodExperience = () => {
    if (paymentState === 'success') {
      return (
        <div className="checkout-method-panel success">
          <CheckCircle2 size={22} />
          <div>
            <strong>Payment authorized</strong>
            <span>Checkout would now capture payment, update order, and send signed webhook.</span>
          </div>
        </div>
      );
    }

    if (paymentState === 'processing') {
      return (
        <div className="checkout-method-panel processing">
          <LoaderCircle size={22} className="spin" />
          <div>
            <strong>Processing {selectedLabel}</strong>
            <span>Simulating gateway authorization and webhook queue.</span>
          </div>
        </div>
      );
    }

    switch (selectedMethod?.method_type) {
      case 'upi':
        return (
          <div className="checkout-method-panel upi-panel">
            <div className="upi-qr"><QrCode size={54} /></div>
            <div>
              <strong>Scan or open UPI app</strong>
              <span>UPI ID: teststore@feelpay • Intent fallback enabled for mobile.</span>
              <div className="mini-actions"><button>PhonePe</button><button>GPay</button><button>Paytm</button></div>
            </div>
          </div>
        );
      case 'card':
        return (
          <div className="checkout-method-panel card-panel">
            <CreditCard size={22} />
            <div className="card-fields-preview">
              <strong>Card authorization</strong>
              <span>4111 1111 1111 1111 • 12/30 • CVV protected</span>
            </div>
          </div>
        );
      case 'netbanking':
        return (
          <div className="checkout-method-panel">
            <Banknote size={22} />
            <div>
              <strong>Select bank</strong>
              <span>HDFC Bank, ICICI Bank, SBI, Axis Bank redirects ready.</span>
            </div>
          </div>
        );
      case 'wallet':
        return (
          <div className="checkout-method-panel">
            <WalletCards size={22} />
            <div>
              <strong>Wallet OTP flow</strong>
              <span>Customer confirms wallet balance and OTP challenge.</span>
            </div>
          </div>
        );
      case 'emi':
        return (
          <div className="checkout-method-panel">
            <CreditCard size={22} />
            <div>
              <strong>EMI plans</strong>
              <span>3, 6, 9, 12 month plans calculated after BIN lookup.</span>
            </div>
          </div>
        );
      default:
        return (
          <div className="checkout-method-panel">
            <WalletCards size={22} />
            <div>
              <strong>Pay Later approval</strong>
              <span>Provider checks customer eligibility before authorization.</span>
            </div>
          </div>
        );
    }
  };

  return (
    <div className="fade-in checkout-preview-page mc-page">
      <div className="page-header">
        <div>
          <h1>Checkout Preview</h1>
          <p>Preview hosted payment experience across website and mobile layouts.</p>
        </div>
        <div className="pill-tabs">
          <button className={`pill-tab ${viewport === 'browser' ? 'active' : ''}`} onClick={() => setViewport('browser')}>
            <Globe2 size={15} /> Browser
          </button>
          <button className={`pill-tab ${viewport === 'laptop' ? 'active' : ''}`} onClick={() => setViewport('laptop')}>
            <Laptop size={15} /> Laptop
          </button>
          <button className={`pill-tab ${viewport === 'mobile' ? 'active' : ''}`} onClick={() => setViewport('mobile')}>
            <Smartphone size={15} /> Mobile
          </button>
        </div>
      </div>

      {loading && <div className="card mc-loader-card"><span className="fp-loader"></span> Loading checkout configuration...</div>}

      <div className="preview-shell">
        <section className="card preview-controls">
          <h3>Checkout Settings</h3>
          <div className="checkout-settings-tabs">
            <button className={settingsTab === 'styling' ? 'active' : ''} type="button" onClick={() => setSettingsTab('styling')}>Checkout Styling</button>
            <button className={settingsTab === 'features' ? 'active' : ''} type="button" onClick={() => setSettingsTab('features')}>Checkout Features</button>
            <button className={settingsTab === 'payment_configuration' ? 'active' : ''} type="button" onClick={() => setSettingsTab('payment_configuration')}>Payment Configuration</button>
          </div>
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
                onClick={() => selectMethod(method.method_type)}
              >
                {method.config?.display_name || LABELS[method.method_type]}
              </button>
            ))}
          </div>

          {settingsTab === 'styling' && (
            <div className="checkout-settings-grid">
              <label>
                Background color
                <input value={settings.styling?.background_color || ''} onChange={(e) => setSettings((p) => ({ ...p, styling: { ...p.styling, background_color: e.target.value } }))} />
              </label>
              <label>
                Border style
                <select value={settings.styling?.border_style || 'rounded'} onChange={(e) => setSettings((p) => ({ ...p, styling: { ...p.styling, border_style: e.target.value as 'rounded' | 'sharp' } }))}>
                  <option value="rounded">Rounded</option>
                  <option value="sharp">Sharp</option>
                </select>
              </label>
              <label>
                Font
                <select value={settings.styling?.font_family || 'IBM Plex Sans'} onChange={(e) => setSettings((p) => ({ ...p, styling: { ...p.styling, font_family: e.target.value } }))}>
                  <option value="IBM Plex Sans">IBM Plex Sans</option>
                  <option value="Inter">Inter</option>
                  <option value="Roboto Slab">Roboto Slab</option>
                </select>
              </label>
            </div>
          )}

          {settingsTab === 'features' && (
            <div className="checkout-settings-grid">
              <label><input type="checkbox" checked={Boolean(settings.features?.show_mandate_summary)} onChange={(e) => setSettings((p) => ({ ...p, features: { ...p.features, show_mandate_summary: e.target.checked } }))} /> Mandate summary page</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.collect_email)} onChange={(e) => setSettings((p) => ({ ...p, features: { ...p.features, collect_email: e.target.checked } }))} /> Collect customer email</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.show_message_banner)} onChange={(e) => setSettings((p) => ({ ...p, features: { ...p.features, show_message_banner: e.target.checked } }))} /> Message banner</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.flash_checkout)} onChange={(e) => setSettings((p) => ({ ...p, features: { ...p.features, flash_checkout: e.target.checked } }))} /> Flash checkout</label>
            </div>
          )}

          {settingsTab === 'payment_configuration' && (
            <div className="checkout-settings-grid">
              <label>
                Preset name
                <input value={settings.payment_configuration?.preset_name || ''} onChange={(e) => setSettings((p) => ({ ...p, payment_configuration: { ...p.payment_configuration, preset_name: e.target.value } }))} />
              </label>
              <label>
                <input type="checkbox" checked={Boolean(settings.payment_configuration?.allow_custom_configuration)} onChange={(e) => setSettings((p) => ({ ...p, payment_configuration: { ...p.payment_configuration, allow_custom_configuration: e.target.checked } }))} />
                Allow custom payment configuration
              </label>
            </div>
          )}

          <button className="btn btn-primary" type="button" onClick={saveCheckoutSettings} disabled={settingsSaving}>
            {settingsSaving ? 'Saving...' : 'Save all changes'}
          </button>
          {settingsNotice && <p className="mini-help">{settingsNotice}</p>}
          <div className="checkout-audit">
            <span>Recent changes</span>
            {(auditItems || []).slice(0, 3).map((item, idx) => (
              <div key={`${item.created_at}-${idx}`} className="checkout-audit-item">
                {new Date(item.created_at).toLocaleString('en-IN')} • {item.actor_id || 'merchant'}
              </div>
            ))}
          </div>
        </section>

        <section className={`checkout-stage ${viewport}`}>
          {viewport === 'laptop' && <div className="laptop-lid-dot"></div>}
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
                  <button key={method.method_type} className={active ? 'active' : ''} onClick={() => selectMethod(method.method_type)}>
                    <Icon size={18} />
                    <span>{method.config?.display_name || LABELS[method.method_type]}</span>
                    <small>{method.config?.checkout_note}</small>
                  </button>
                );
              })}
            </div>

            {renderMethodExperience()}

            <button className="checkout-pay-button" onClick={runPreviewPayment} disabled={paymentState === 'processing'}>
              {paymentState === 'processing' ? 'Processing...' : `Pay with ${selectedLabel}`}
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
        .checkout-settings-tabs { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 6px; }
        .checkout-settings-tabs button { border: 1px solid #d5c9af; background: #fff9ec; border-radius: 999px; padding: 8px 10px; font-size: 12px; font-weight: 700; color: #6b6862; cursor: pointer; }
        .checkout-settings-tabs button.active { border-color: #eb001b; color: #141413; background: #f5efe2; }
        .checkout-settings-grid { display: grid; gap: 8px; }
        .checkout-settings-grid label { display: grid; gap: 6px; font-size: 12px; color: #6b6862; }
        .checkout-settings-grid input[type='checkbox'] { margin-right: 8px; accent-color: #eb001b; }
        .checkout-audit { display: grid; gap: 6px; border-top: 1px dashed #d5c9af; padding-top: 10px; }
        .checkout-audit > span { font-size: 12px; color: #6b6862; font-weight: 700; }
        .checkout-audit-item { font-size: 12px; color: #141413; background: #f5efe2; border: 1px solid #d5c9af; border-radius: 10px; padding: 6px 8px; }
        .preview-merchant-name { color: #312d28; background: #f5efe2; border: 1px solid #d5c9af; border-radius: 999px; padding: 12px 14px; font-weight: 800; font-size: 18px; }
        .preview-controls label { display: flex; flex-direction: column; gap: 6px; color: var(--fp-text-muted); font-size: 13px; }
        .preview-controls input { background: #fff9ec; border: 1px solid #d5c9af; color: #141413; border-radius: 999px; padding: 10px 12px; }
        .preview-method-list { display: grid; gap: 8px; }
        .preview-method-list span { color: var(--fp-text-muted); font-size: 13px; }
        .preview-method-list button { text-align: left; background: #fff9ec; border: 1px solid #d5c9af; color: #6b6862; border-radius: 999px; padding: 9px 12px; cursor: pointer; }
        .preview-method-list button.selected { border-color: #eb001b; color: #141413; background: #f5efe2; }
        .checkout-stage { min-height: 720px; border: 1px solid #d5c9af; border-radius: 34px; background: #f5efe2; padding: 24px; display: flex; flex-direction: column; align-items: center; overflow: hidden; }
        .checkout-stage.browser { width: 100%; }
        .checkout-stage.laptop { width: min(100%, 980px); margin: 0 auto; padding: 18px 18px 54px; border-radius: 26px; background: #312d28; position: relative; }
        .checkout-stage.laptop::after { content: ''; position: absolute; left: 50%; bottom: 14px; transform: translateX(-50%); width: 58%; height: 10px; border-radius: 0 0 999px 999px; background: #1f1c19; }
        .laptop-lid-dot { width: 8px; height: 8px; border-radius: 999px; background: #6b6862; margin-bottom: 10px; }
        .checkout-stage.mobile { width: 390px; min-height: 812px; margin: 0 auto; padding: 14px; border-radius: 42px; box-shadow: inset 0 0 0 8px #312d28; background: #312d28; }
        .browser-bar { width: 100%; max-width: 820px; height: 38px; border-radius: 28px 28px 0 0; background: #fff9ec; display: flex; align-items: center; gap: 7px; padding: 0 12px; color: #141413; }
        .browser-bar span { width: 9px; height: 9px; border-radius: 99px; background: #6b6862; }
        .browser-bar strong { margin-left: 10px; font-size: 12px; font-weight: 600; }
        .checkout-stage.laptop .browser-bar, .checkout-stage.laptop .checkout-card { max-width: 880px; }
        .checkout-stage.mobile .browser-bar, .checkout-stage.mobile .checkout-card { max-width: 344px; }
        .checkout-stage.mobile .browser-bar { margin-top: 18px; }
        .checkout-card { width: 100%; max-width: 820px; min-height: 600px; background: ${settings.styling?.background_color ? '#fff9ec' : '#fff9ec'}; color: #141413; border: 1px solid #d5c9af; border-top: none; border-radius: ${settings.styling?.border_style === 'sharp' ? '0 0 8px 8px' : '0 0 28px 28px'}; padding: 28px; display: grid; gap: 18px; align-content: start; box-shadow: none; font-family: ${settings.styling?.font_family || 'IBM Plex Sans'}, sans-serif; }
        .checkout-stage.laptop .checkout-card { min-height: 530px; }
        .checkout-stage.mobile .checkout-card { min-height: 660px; padding: 18px; gap: 14px; }
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
        .checkout-method-panel { display: grid; grid-template-columns: 44px minmax(0, 1fr); gap: 12px; align-items: center; border: 1px solid #d5c9af; background: #f5efe2; border-radius: 28px; padding: 14px; color: #312d28; }
        .checkout-method-panel strong { display: block; font-size: 15px; margin-bottom: 3px; }
        .checkout-method-panel span { color: #6b6862; font-size: 12px; line-height: 1.4; }
        .checkout-method-panel.success { border-color: rgba(35, 110, 62, 0.36); background: #eef8ef; color: #236e3e; }
        .checkout-method-panel.processing { border-color: rgba(235, 0, 27, 0.28); }
        .upi-qr { width: 72px; height: 72px; border-radius: 18px; display: grid; place-items: center; background: #fff9ec; border: 1px solid #d5c9af; }
        .upi-panel { grid-template-columns: 72px minmax(0, 1fr); }
        .mini-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
        .mini-actions button { border: 1px solid #d5c9af; background: #fff9ec; color: #312d28; border-radius: 999px; padding: 5px 9px; font-size: 11px; font-weight: 800; }
        .checkout-stage.mobile .checkout-method-panel { grid-template-columns: 34px minmax(0, 1fr); padding: 12px; }
        .checkout-stage.mobile .upi-panel { grid-template-columns: 58px minmax(0, 1fr); }
        .checkout-stage.mobile .upi-qr { width: 58px; height: 58px; }
        .checkout-pay-button { border: none; border-radius: 999px; padding: 14px 18px; font-weight: 800; background: #eb001b; color: #fff9ec; cursor: pointer; min-height: 50px; }
        .checkout-pay-button:disabled { opacity: 0.74; cursor: progress; }
        .checkout-secure { display: inline-flex; gap: 7px; align-items: center; justify-content: center; color: #6b6862; font-size: 12px; }
        .payment-flow-panel { margin-top: 18px; }
        .payment-flow-panel h3 { margin-bottom: 14px; }
        .payment-flow-grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 10px; }
        .payment-flow-grid article { border: 1px solid #d5c9af; background: #fff9ec; color: #141413; border-radius: 28px; padding: 14px; display: grid; gap: 10px; }
        .payment-flow-grid article > strong { color: #eb001b; font-size: 13px; }
        .payment-flow-grid h4 { margin: 0 0 4px; font-size: 14px; }
        .payment-flow-grid p { margin: 0; color: #6b6862; font-size: 12px; line-height: 1.45; }
        .payment-flow-grid span { display: inline-flex; margin-top: 10px; border: 1px solid #d5c9af; border-radius: 999px; padding: 3px 8px; color: #6b6862; font-size: 11px; }
        @media (max-width: 980px) { .preview-shell { grid-template-columns: 1fr; } .preview-controls { position: static; } .checkout-stage { min-height: auto; } .checkout-settings-tabs { grid-template-columns: 1fr; } }
        @media (max-width: 430px) { .checkout-stage.mobile { width: 100%; box-shadow: inset 0 0 0 6px #312d28; } }
        @media (max-width: 980px) { .payment-flow-grid { grid-template-columns: 1fr; } }
      `}</style>
    </div>
  );
}
