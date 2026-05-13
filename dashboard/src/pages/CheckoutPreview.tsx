import { useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
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
import { formatPaise, parseInrToPaise } from '../utils/money';

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
    theme_color?: string;
    accent_color?: string;
    border_style?: 'rounded' | 'sharp';
    font_family?: string;
    layout?: 'standard' | 'split' | 'compact';
    logo_url?: string;
    button_text?: string;
    corner_radius?: number;
    show_sidebar_graphic?: boolean;
    show_trusted_badge?: boolean;
  };
  features?: {
    show_mandate_summary?: boolean;
    default_language?: string;
    collect_email?: boolean;
    collect_contact?: boolean;
    show_message_banner?: boolean;
    banner_text?: string;
    flash_checkout?: boolean;
    retry_enabled?: boolean;
    show_saved_methods?: boolean;
    show_fee_breakup?: boolean;
    remember_customer?: boolean;
    readonly_email?: boolean;
    readonly_contact?: boolean;
  };
  payment_configuration?: {
    preset_name?: string;
    allow_custom_configuration?: boolean;
    preferred_method?: MethodKey;
    promoted_methods?: MethodKey[];
    method_order?: MethodKey[];
    blocked_card_networks?: string[];
    allowed_upi_apps?: string[];
  };
};

type CheckoutAuditItem = {
  created_at: string;
  actor_id?: string | null;
};

type PaymentResult = {
  session_id?: string;
  payment_id?: string;
  payment_status?: string;
  connector?: string;
  payment_method?: string;
  mode_badge?: string;
  error?: string;
};

type SimulationAction = 'success' | 'fail' | 'processing';

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

const LANGUAGES = ['English', 'Hindi', 'Kannada', 'Tamil', 'Telugu', 'Marathi'];
const CARD_NETWORKS = ['Visa', 'Mastercard', 'RuPay', 'Amex', 'Diners'];
const UPI_APPS = ['GPay', 'Paytm', 'BHIM'];

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
  const [paymentResult, setPaymentResult] = useState<PaymentResult | null>(null);
  const [simulationAction, setSimulationAction] = useState<SimulationAction>('success');
  const [settingsTab, setSettingsTab] = useState<'styling' | 'features' | 'payment_configuration'>('styling');
  const [settings, setSettings] = useState<CheckoutSettings>({
    styling: {
      background_color: '#fff9ec',
      theme_color: '#0b72e7',
      accent_color: '#10b981',
      border_style: 'rounded',
      font_family: 'IBM Plex Sans',
      layout: 'split',
      logo_url: '/feelpay.png',
      button_text: 'Pay securely',
      corner_radius: 28,
      show_sidebar_graphic: true,
      show_trusted_badge: true,
    },
    features: {
      show_mandate_summary: true,
      default_language: 'English',
      collect_email: true,
      collect_contact: true,
      show_message_banner: true,
      banner_text: 'Test mode checkout. No real money moves.',
      flash_checkout: true,
      retry_enabled: true,
      show_saved_methods: true,
      show_fee_breakup: true,
      remember_customer: true,
      readonly_email: false,
      readonly_contact: false,
    },
    payment_configuration: {
      preset_name: 'High conversion checkout',
      allow_custom_configuration: true,
      preferred_method: 'upi',
      promoted_methods: ['upi'],
      method_order: ['upi', 'card', 'wallet', 'netbanking', 'emi', 'bnpl'],
      blocked_card_networks: [],
      allowed_upi_apps: UPI_APPS,
    },
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
        const order = checkoutSettings.payment_configuration?.method_order || [];
        const ordered = normalized
          .map((method) => ({
            ...method,
            config: {
              ...method.config,
              priority: order.includes(method.method_type) ? order.indexOf(method.method_type) + 1 : method.config?.priority,
            },
          }));
        setMethods(ordered);
        setMerchantName(data.merchant?.business_name || data.merchant?.name || 'FeelPay Merchant');
        setMerchantTagline('Payments that build trust');
        if (data.sample_order?.amount) setAmount(formatPaise(data.sample_order.amount, { symbol: false }));
        if (data.sample_order?.customer_email) setCustomer(data.sample_order.customer_email);
        setEventFlow(data.event_flow || []);
        setSettings((prev) => ({
          styling: { ...prev.styling, ...(checkoutSettings.styling || {}) },
          features: { ...prev.features, ...(checkoutSettings.features || {}) },
          payment_configuration: { ...prev.payment_configuration, ...(checkoutSettings.payment_configuration || {}) },
        }));
        setAuditItems(auditRes.items || []);
        const first = (checkoutSettings.payment_configuration?.preferred_method || ordered.find((method) => method.enabled)?.method_type);
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
  const themeColor = settings.styling?.theme_color || settings.styling?.background_color || '#0b72e7';
  const accentColor = settings.styling?.accent_color || '#10b981';
  const checkoutBg = settings.styling?.background_color || '#fff9ec';
  const cornerRadius = Number(settings.styling?.corner_radius || 28);
  const promotedMethods = settings.payment_configuration?.promoted_methods || [];
  const isPromoted = selectedMethod ? promotedMethods.includes(selectedMethod.method_type) : false;

  const selectMethod = (methodType: MethodKey) => {
    setSelected(methodType);
    setPaymentState('idle');
    setPaymentResult(null);
  };

  const setFeature = <K extends keyof NonNullable<CheckoutSettings['features']>>(key: K, value: NonNullable<CheckoutSettings['features']>[K]) => {
    setSettings((p) => ({ ...p, features: { ...p.features, [key]: value } }));
  };

  const setStyle = <K extends keyof NonNullable<CheckoutSettings['styling']>>(key: K, value: NonNullable<CheckoutSettings['styling']>[K]) => {
    setSettings((p) => ({ ...p, styling: { ...p.styling, [key]: value } }));
  };

  const setPaymentConfig = <K extends keyof NonNullable<CheckoutSettings['payment_configuration']>>(key: K, value: NonNullable<CheckoutSettings['payment_configuration']>[K]) => {
    setSettings((p) => ({ ...p, payment_configuration: { ...p.payment_configuration, [key]: value } }));
  };

  const persistMethodOrder = (nextMethods: PaymentMethod[]) => {
    setPaymentConfig('method_order', nextMethods.sort((a, b) => (a.config?.priority || 99) - (b.config?.priority || 99)).map((m) => m.method_type));
  };

  const toggleMethod = (methodType: MethodKey) => {
    setMethods((current) => {
      const next = current.map((method) => method.method_type === methodType ? { ...method, enabled: !method.enabled } : method);
      persistMethodOrder(next);
      return next;
    });
  };

  const moveMethod = (methodType: MethodKey, direction: -1 | 1) => {
    setMethods((current) => {
      const ordered = [...current].sort((a, b) => (a.config?.priority || 99) - (b.config?.priority || 99));
      const index = ordered.findIndex((method) => method.method_type === methodType);
      const target = index + direction;
      if (index < 0 || target < 0 || target >= ordered.length) return current;
      const currentMethod = ordered[index];
      const targetMethod = ordered[target];
      if (!currentMethod || !targetMethod) return current;
      ordered[index] = targetMethod;
      ordered[target] = currentMethod;
      const next = ordered.map((method, idx) => ({ ...method, config: { ...method.config, priority: idx + 1 } }));
      persistMethodOrder(next);
      return next;
    });
  };

  const togglePromotedMethod = (methodType: MethodKey) => {
    const current = settings.payment_configuration?.promoted_methods || [];
    setPaymentConfig(
      'promoted_methods',
      current.includes(methodType) ? current.filter((item) => item !== methodType) : [...current, methodType],
    );
  };

  const togglePaymentConfigList = (
    key: 'blocked_card_networks' | 'allowed_upi_apps',
    value: string,
  ) => {
    const current = settings.payment_configuration?.[key] || [];
    setPaymentConfig(key, current.includes(value) ? current.filter((item) => item !== value) : [...current, value]);
  };

  const runPreviewPayment = async () => {
    setPaymentState('processing');
    setPaymentResult(null);
    try {
      const rawMerchant = localStorage.getItem('feelpay_merchant');
      const merchant = rawMerchant ? (JSON.parse(rawMerchant) as { id?: string }) : {};
      if (!merchant.id) throw new Error('Merchant session missing.');

      const session = await apiRequest<{ session_id: string }>('POST', '/v1/checkout/session', {
        merchant_id: merchant.id,
        order_id: `preview_${Date.now()}`,
        amount: parseInrToPaise(amount || '0') || 0,
        currency: 'INR',
        customer_email: customer,
        metadata: { source: 'dashboard_checkout_preview' },
      });

      const result = await apiRequest<PaymentResult>('POST', `/v1/checkout/session/${session.session_id}/pay`, {
        method_data: { Simulator: { action: simulationAction } },
        processor_override: 'simulator',
        simulation_action: simulationAction,
        selected_payment_method: selectedMethod?.method_type || 'simulator',
      });

      setPaymentResult(result);
      setPaymentState(['captured', 'authorized'].includes(result.payment_status || '') ? 'success' : 'idle');
    } catch (err) {
      setPaymentResult({ error: err instanceof Error ? err.message : 'Payment failed' });
      setPaymentState('idle');
    }
  };

  const saveCheckoutSettings = async () => {
    const hexFields = [
      settings.styling?.background_color,
      settings.styling?.theme_color,
      settings.styling?.accent_color,
    ].filter(Boolean) as string[];
    if (hexFields.some((value) => !/^#([0-9A-Fa-f]{6})$/.test(value.trim()))) {
      setSettingsNotice('Colors must be hex like #0B72E7');
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
            <strong>Payment captured</strong>
            <span>
              <b>{paymentResult?.mode_badge || 'TEST'}</b> • {paymentResult?.connector || 'simulator'} • {paymentResult?.payment_id || 'payment created'}
            </span>
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

    if (paymentResult?.error || paymentResult?.payment_status === 'failed') {
      return (
        <div className="checkout-method-panel failed">
          <AlertCircle size={22} />
          <div>
            <strong>Payment failed</strong>
            <span>{paymentResult?.error || 'Simulated gateway decline. Retry option can route next best method.'}</span>
          </div>
        </div>
      );
    }

    if (paymentResult?.payment_status === 'processing') {
      return (
        <div className="checkout-method-panel processing">
          <LoaderCircle size={22} className="spin" />
          <div>
            <strong>Payment pending</strong>
            <span>Simulator returned processing. Status polling and webhook update remain pending.</span>
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
              <div className="mini-actions"><button>GPay</button><button>Paytm</button><button>BHIM</button></div>
            </div>
          </div>
        );
      case 'card':
        return (
          <div className="checkout-method-panel card-panel">
            <CreditCard size={22} />
            <div className="card-fields-preview">
              <strong>Test card authorization</strong>
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

  const simulationSteps = [
    { label: 'Session', value: paymentResult?.session_id ? 'created' : 'ready' },
    { label: 'Connector', value: paymentResult?.connector || 'simulator' },
    { label: 'Result', value: paymentResult?.payment_status || simulationAction },
  ];

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
                Checkout background
                <input value={settings.styling?.background_color || ''} onChange={(e) => setStyle('background_color', e.target.value)} />
              </label>
              <label>
                Brand color
                <input value={settings.styling?.theme_color || ''} onChange={(e) => setStyle('theme_color', e.target.value)} />
              </label>
              <label>
                Success accent
                <input value={settings.styling?.accent_color || ''} onChange={(e) => setStyle('accent_color', e.target.value)} />
              </label>
              <label>
                Border style
                <select value={settings.styling?.border_style || 'rounded'} onChange={(e) => setStyle('border_style', e.target.value as 'rounded' | 'sharp')}>
                  <option value="rounded">Rounded</option>
                  <option value="sharp">Sharp</option>
                </select>
              </label>
              <label>
                Template
                <select value={settings.styling?.layout || 'split'} onChange={(e) => setStyle('layout', e.target.value as 'standard' | 'split' | 'compact')}>
                  <option value="split">Split: brand + payment</option>
                  <option value="standard">Standard popup</option>
                  <option value="compact">Compact mobile first</option>
                </select>
              </label>
              <label>
                Font
                <select value={settings.styling?.font_family || 'IBM Plex Sans'} onChange={(e) => setStyle('font_family', e.target.value)}>
                  <option value="IBM Plex Sans">IBM Plex Sans</option>
                  <option value="Inter">Inter</option>
                  <option value="Roboto Slab">Roboto Slab</option>
                </select>
              </label>
              <label>
                Button text
                <input value={settings.styling?.button_text || ''} onChange={(e) => setStyle('button_text', e.target.value)} />
              </label>
              <label>
                Corner radius
                <input type="number" min="8" max="36" value={settings.styling?.corner_radius || 28} onChange={(e) => setStyle('corner_radius', Number(e.target.value))} />
              </label>
            </div>
          )}

          {settingsTab === 'features' && (
            <div className="checkout-settings-grid">
              <label>
                Language
                <select value={settings.features?.default_language || 'English'} onChange={(e) => setFeature('default_language', e.target.value)}>
                  {LANGUAGES.map((language) => <option key={language} value={language}>{language}</option>)}
                </select>
              </label>
              <label><input type="checkbox" checked={Boolean(settings.features?.show_mandate_summary)} onChange={(e) => setFeature('show_mandate_summary', e.target.checked)} /> Mandatory order summary</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.collect_email)} onChange={(e) => setFeature('collect_email', e.target.checked)} /> Collect customer email</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.collect_contact)} onChange={(e) => setFeature('collect_contact', e.target.checked)} /> Collect phone number</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.readonly_email)} onChange={(e) => setFeature('readonly_email', e.target.checked)} /> Lock prefilled email</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.readonly_contact)} onChange={(e) => setFeature('readonly_contact', e.target.checked)} /> Lock prefilled phone</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.show_message_banner)} onChange={(e) => setFeature('show_message_banner', e.target.checked)} /> Message banner</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.flash_checkout)} onChange={(e) => setFeature('flash_checkout', e.target.checked)} /> Flash checkout</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.retry_enabled)} onChange={(e) => setFeature('retry_enabled', e.target.checked)} /> Smart retry after failure</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.show_saved_methods)} onChange={(e) => setFeature('show_saved_methods', e.target.checked)} /> Saved instruments</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.show_fee_breakup)} onChange={(e) => setFeature('show_fee_breakup', e.target.checked)} /> Fee breakup</label>
              <label><input type="checkbox" checked={Boolean(settings.features?.remember_customer)} onChange={(e) => setFeature('remember_customer', e.target.checked)} /> Remember customer</label>
              <label>
                Banner text
                <input value={settings.features?.banner_text || ''} onChange={(e) => setFeature('banner_text', e.target.value)} />
              </label>
            </div>
          )}

          {settingsTab === 'payment_configuration' && (
            <div className="checkout-settings-grid">
              <label>
                Preset name
                <input value={settings.payment_configuration?.preset_name || ''} onChange={(e) => setSettings((p) => ({ ...p, payment_configuration: { ...p.payment_configuration, preset_name: e.target.value } }))} />
              </label>
              <label>
                <input type="checkbox" checked={Boolean(settings.payment_configuration?.allow_custom_configuration)} onChange={(e) => setPaymentConfig('allow_custom_configuration', e.target.checked)} />
                Allow custom payment configuration
              </label>
              <label>
                Preferred method
                <select value={settings.payment_configuration?.preferred_method || selected} onChange={(e) => { const method = e.target.value as MethodKey; setPaymentConfig('preferred_method', method); selectMethod(method); }}>
                  {(Object.keys(LABELS) as MethodKey[]).map((method) => <option key={method} value={method}>{LABELS[method]}</option>)}
                </select>
              </label>
              <label>
                Simulation result
                <select value={simulationAction} onChange={(e) => setSimulationAction(e.target.value as SimulationAction)}>
                  <option value="success">Success captured</option>
                  <option value="fail">Failure declined</option>
                  <option value="processing">Processing / pending</option>
                </select>
              </label>
              <div className="method-config-builder">
                <span>Methods, order, promoted blocks</span>
                {[...methods].sort((a, b) => (a.config?.priority || 99) - (b.config?.priority || 99)).map((method) => (
                  <div key={method.method_type} className="method-config-row">
                    <label><input type="checkbox" checked={method.enabled} onChange={() => toggleMethod(method.method_type)} /> {LABELS[method.method_type]}</label>
                    <button type="button" onClick={() => moveMethod(method.method_type, -1)}>↑</button>
                    <button type="button" onClick={() => moveMethod(method.method_type, 1)}>↓</button>
                    <button type="button" className={promotedMethods.includes(method.method_type) ? 'active' : ''} onClick={() => togglePromotedMethod(method.method_type)}>Promote</button>
                  </div>
                ))}
              </div>
              <div className="chip-field">
                <span>Blocked card networks</span>
                <div className="settings-chip-list" role="group" aria-label="Blocked card networks">
                  {CARD_NETWORKS.map((network) => {
                    const active = (settings.payment_configuration?.blocked_card_networks || []).includes(network);
                    return (
                      <button key={network} type="button" className={active ? 'active' : ''} onClick={() => togglePaymentConfigList('blocked_card_networks', network)}>
                        {network}
                      </button>
                    );
                  })}
                </div>
              </div>
              <div className="chip-field">
                <span>UPI apps shown</span>
                <div className="settings-chip-list" role="group" aria-label="UPI apps shown">
                  {UPI_APPS.map((app) => {
                    const active = (settings.payment_configuration?.allowed_upi_apps || []).includes(app);
                    return (
                      <button key={app} type="button" className={active ? 'active' : ''} onClick={() => togglePaymentConfigList('allowed_upi_apps', app)}>
                        {app}
                      </button>
                    );
                  })}
                </div>
              </div>
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

        <section className={`checkout-stage ${viewport} template-${settings.styling?.layout || 'split'}`}>
          {viewport === 'laptop' && <div className="laptop-lid-dot"></div>}
          <div className="browser-bar">
            <span></span><span></span><span></span>
            <strong>checkout.feelpay.in</strong>
          </div>
          <div className="checkout-card">
            <div className="test-mode-ribbon">
              <span>TEST MODE</span>
              <strong>{simulationAction === 'success' ? 'Approve payment' : simulationAction === 'fail' ? 'Decline payment' : 'Keep pending'}</strong>
            </div>
            <div className="checkout-brand">
              <img src={settings.styling?.logo_url || '/feelpay.png'} alt="FeelPay" />
              <div>
                <strong>{merchantName}</strong>
                <span>{merchantTagline}</span>
              </div>
            </div>

            {settings.features?.show_message_banner && (
              <div className="checkout-banner">
                {settings.features?.banner_text || 'Secure checkout is ready.'}
              </div>
            )}

            <div className="simulation-switcher">
              {(['success', 'fail', 'processing'] as SimulationAction[]).map((action) => (
                <button
                  key={action}
                  type="button"
                  className={simulationAction === action ? 'active' : ''}
                  onClick={() => {
                    setSimulationAction(action);
                    setPaymentResult(null);
                    setPaymentState('idle');
                  }}
                >
                  {action === 'success' ? 'Success' : action === 'fail' ? 'Fail' : 'Pending'}
                </button>
              ))}
            </div>

            <div className="checkout-amount">
              <span>Amount payable</span>
              <strong>{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
              <small>
                {settings.features?.collect_email ? customer : 'Email collection off'}
                {settings.features?.readonly_email ? ' • locked' : ''}
              </small>
              {settings.features?.collect_contact && <small>+91 99999 99999{settings.features?.readonly_contact ? ' • locked' : ''}</small>}
            </div>

            {settings.features?.show_mandate_summary && (
              <div className="checkout-summary-strip">
                <span>Order summary required</span>
                <strong>Preview order • INR • {settings.features?.default_language || 'English'}</strong>
              </div>
            )}

            {settings.features?.show_saved_methods && (
              <div className="saved-methods-strip">
                <span>Saved</span>
                <button type="button">HDFC Visa • 1111</button>
                <button type="button">buyer@upi</button>
              </div>
            )}

            <div className="checkout-methods">
              {visibleMethods.map((method) => {
                const Icon = ICONS[method.method_type];
                const active = selectedMethod?.method_type === method.method_type;
                return (
                  <button key={method.method_type} className={active ? 'active' : ''} onClick={() => selectMethod(method.method_type)}>
                    <Icon size={18} />
                    <span>
                      {method.config?.display_name || LABELS[method.method_type]}
                      {promotedMethods.includes(method.method_type) && <b>Promoted</b>}
                    </span>
                    <small>{method.config?.checkout_note}</small>
                  </button>
                );
              })}
            </div>

            {renderMethodExperience()}

            <div className="simulation-timeline" aria-label="Payment simulation timeline">
              {simulationSteps.map((step) => (
                <div key={step.label}>
                  <span>{step.label}</span>
                  <strong>{step.value}</strong>
                </div>
              ))}
            </div>

            {settings.features?.show_fee_breakup && (
              <div className="fee-breakup">
                <span>Amount</span><strong>{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
                <span>Gateway fee</span><strong>₹0.00</strong>
                <span>Total</span><strong>{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
              </div>
            )}

            {isPromoted && <div className="recommended-method">Recommended for better success rate</div>}

            <button className="checkout-pay-button" onClick={() => void runPreviewPayment()} disabled={paymentState === 'processing'}>
              {paymentState === 'processing' ? 'Processing...' : `${settings.styling?.button_text || 'Pay securely'} with ${selectedLabel}`}
            </button>
            {settings.features?.retry_enabled && paymentResult?.error && <button className="retry-button" type="button" onClick={() => void runPreviewPayment()}>Retry with suggested method</button>}
            <div className="checkout-secure">
              <ShieldCheck size={14} />
              {settings.styling?.show_trusted_badge === false ? 'Test checkout preview' : 'PCI-ready vault, signed webhooks, live/test isolation'}
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
          border: 1px solid #d9e2ef;
          border-radius: 34px;
          background: #ffffff;
          padding: 24px;
        }
        .preview-shell { display: grid; grid-template-columns: minmax(300px, 360px) minmax(0, 1fr); gap: 20px; align-items: start; margin-top: 16px; }
        .preview-controls { display: grid; gap: 14px; position: sticky; top: 20px; background: #ffffff !important; border-radius: 18px !important; }
        .preview-controls h3 { margin: 0; font-size: 22px; }
        .checkout-settings-tabs { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 6px; }
        .checkout-settings-tabs button { border: 1px solid #d9e2ef; background: #ffffff; border-radius: 999px; padding: 8px 10px; font-size: 12px; font-weight: 700; color: #64748b; cursor: pointer; }
        .checkout-settings-tabs button.active { border-color: ${themeColor}; color: #0f172a; background: #eef6ff; }
        .checkout-settings-grid { display: grid; gap: 10px; }
        .checkout-settings-grid label { display: grid; gap: 6px; font-size: 13px; color: #4f4a43; font-weight: 700; }
        .checkout-settings-grid label:has(input[type='checkbox']) { min-height: 44px; display: flex; flex-direction: row; align-items: center; gap: 10px; border: 1px solid #d9e2ef; background: #f8fafc; border-radius: 16px; padding: 10px 12px; color: #172033; }
        .checkout-settings-grid input[type='checkbox'] { width: 22px; height: 22px; margin: 0; accent-color: #eb001b; flex: 0 0 auto; }
        .checkout-audit { display: grid; gap: 6px; border-top: 1px dashed #d9e2ef; padding-top: 10px; }
        .checkout-audit > span { font-size: 12px; color: #64748b; font-weight: 700; }
        .checkout-audit-item { font-size: 12px; color: #0f172a; background: #f6f8fb; border: 1px solid #d9e2ef; border-radius: 10px; padding: 6px 8px; }
        .preview-merchant-name { color: #172033; background: #f6f8fb; border: 1px solid #d9e2ef; border-radius: 999px; padding: 12px 14px; font-weight: 800; font-size: 18px; }
        .preview-controls label { display: flex; flex-direction: column; gap: 6px; color: var(--fp-text-muted); font-size: 13px; }
        .preview-controls input,
        .preview-controls select { background: #ffffff; border: 1px solid #d9e2ef; color: #0f172a; border-radius: 999px; padding: 10px 12px; min-width: 0; min-height: 42px; font-size: 14px; }
        .preview-method-list { display: grid; gap: 8px; }
        .preview-method-list span { color: var(--fp-text-muted); font-size: 13px; }
        .preview-method-list button { text-align: left; background: #ffffff; border: 1px solid #d9e2ef; color: #64748b; border-radius: 999px; padding: 9px 12px; cursor: pointer; }
        .preview-method-list button.selected { border-color: ${themeColor}; color: #0f172a; background: #eef6ff; }
        .method-config-builder { display: grid; gap: 8px; }
        .method-config-builder > span { color: #6b6862; font-size: 12px; font-weight: 800; }
        .method-config-row { display: grid; grid-template-columns: minmax(0, 1fr) 40px 40px minmax(78px, auto); gap: 7px; align-items: center; border: 1px solid #d9e2ef; background: #ffffff; border-radius: 18px; padding: 9px; }
        .method-config-row label { display: inline-flex; flex-direction: row; align-items: center; gap: 7px; color: #172033; }
        .method-config-row button { border: 1px solid #d9e2ef; background: #f6f8fb; color: #172033; border-radius: 999px; min-height: 38px; cursor: pointer; font-weight: 800; }
        .method-config-row button.active { border-color: ${themeColor}; color: ${themeColor}; background: #fff; }
        .chip-field { display: grid; gap: 8px; }
        .chip-field > span { color: #4f4a43; font-size: 13px; font-weight: 800; }
        .settings-chip-list { display: flex; flex-wrap: wrap; gap: 8px; border: 1px solid #d9e2ef; background: #f8fafc; border-radius: 18px; padding: 10px; }
        .settings-chip-list button { border: 1px solid #d9e2ef; background: #ffffff; color: #172033; border-radius: 999px; min-height: 40px; padding: 8px 12px; cursor: pointer; font-weight: 800; }
        .settings-chip-list button.active { border-color: ${themeColor}; color: ${themeColor}; background: #eef6ff; box-shadow: inset 0 0 0 1px ${themeColor}; }
        .checkout-stage { min-height: 720px; border: 1px solid #d9e2ef; border-radius: 34px; background: #f6f8fb; padding: 24px; display: flex; flex-direction: column; align-items: center; overflow: hidden; }
        .checkout-stage.browser { width: 100%; }
        .checkout-stage.laptop { width: min(100%, 980px); margin: 0 auto; padding: 18px 18px 54px; border-radius: 26px; background: #172033; position: relative; }
        .checkout-stage.laptop::after { content: ''; position: absolute; left: 50%; bottom: 14px; transform: translateX(-50%); width: 58%; height: 10px; border-radius: 0 0 999px 999px; background: #1f1c19; }
        .laptop-lid-dot { width: 8px; height: 8px; border-radius: 999px; background: #6b6862; margin-bottom: 10px; }
        .checkout-stage.mobile { width: 390px; min-height: 812px; margin: 0 auto; padding: 14px; border-radius: 42px; box-shadow: inset 0 0 0 8px #172033; background: #172033; }
        .browser-bar { width: 100%; max-width: 820px; height: 38px; border-radius: 28px 28px 0 0; background: #ffffff; display: flex; align-items: center; gap: 7px; padding: 0 12px; color: #0f172a; }
        .browser-bar span { width: 9px; height: 9px; border-radius: 99px; background: #6b6862; }
        .browser-bar strong { margin-left: 10px; font-size: 12px; font-weight: 600; }
        .checkout-stage.laptop .browser-bar, .checkout-stage.laptop .checkout-card { max-width: 880px; }
        .checkout-stage.mobile .browser-bar, .checkout-stage.mobile .checkout-card { max-width: 344px; }
        .checkout-stage.mobile .browser-bar { margin-top: 18px; }
        .checkout-card { width: 100%; max-width: 820px; min-height: 600px; background: ${checkoutBg}; color: #0f172a; border: 1px solid #d9e2ef; border-top: none; border-radius: ${settings.styling?.border_style === 'sharp' ? '0 0 8px 8px' : `0 0 ${cornerRadius}px ${cornerRadius}px`}; padding: 28px; display: grid; gap: 16px; align-content: start; box-shadow: 0 20px 55px rgba(15, 23, 42, 0.08); font-family: ${settings.styling?.font_family || 'IBM Plex Sans'}, sans-serif; }
        .checkout-stage.laptop .checkout-card { min-height: 530px; }
        .checkout-stage.mobile .checkout-card { min-height: 660px; padding: 18px; gap: 14px; }
        .checkout-stage.template-compact .checkout-card { max-width: 470px; }
        .checkout-stage.template-standard .checkout-card { max-width: 620px; }
        .test-mode-ribbon { display: flex; align-items: center; justify-content: space-between; gap: 10px; border: 1px solid #bfdbfe; background: #eff6ff; color: #1d4ed8; border-radius: 18px; padding: 10px 12px; }
        .test-mode-ribbon span { font-size: 11px; font-weight: 900; letter-spacing: 0.08em; }
        .test-mode-ribbon strong { font-size: 13px; }
        .checkout-brand { display: flex; align-items: center; gap: 14px; }
        .checkout-brand img { width: 62px; height: 62px; border-radius: 20px; object-fit: contain; border: 1px solid #d9e2ef; background: #fff; padding: 4px; }
        .checkout-stage.mobile .checkout-brand img { width: 54px; height: 54px; border-radius: 18px; }
        .checkout-brand div { display: flex; flex-direction: column; line-height: 1.2; }
        .checkout-brand strong { font-size: 20px; color: #172033; }
        .checkout-stage.mobile .checkout-brand strong { font-size: 17px; }
        .checkout-brand span { color: #64748b; font-size: 12px; text-transform: uppercase; letter-spacing: 0.08em; }
        .checkout-banner { border: 1px solid ${themeColor}; color: ${themeColor}; background: color-mix(in srgb, ${themeColor} 8%, #ffffff); border-radius: 999px; padding: 10px 14px; font-size: 13px; font-weight: 800; }
        .simulation-switcher { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
        .simulation-switcher button { border: 1px solid #d9e2ef; background: #ffffff; color: #64748b; border-radius: 999px; min-height: 36px; cursor: pointer; font-weight: 800; }
        .simulation-switcher button.active { border-color: ${themeColor}; background: #eef6ff; color: ${themeColor}; }
        .checkout-amount { border: 1px solid #d9e2ef; border-radius: 22px; padding: 16px; display: grid; gap: 2px; background: #f6f8fb; }
        .checkout-amount span, .checkout-amount small { color: #64748b; }
        .checkout-amount strong { font-size: 34px; line-height: 1.1; }
        .checkout-summary-strip { display: flex; align-items: center; justify-content: space-between; gap: 10px; border: 1px dashed #d9e2ef; border-radius: 18px; padding: 10px 12px; color: #64748b; font-size: 12px; }
        .checkout-summary-strip strong { color: #172033; }
        .saved-methods-strip { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
        .saved-methods-strip span { color: #64748b; font-size: 12px; font-weight: 800; }
        .saved-methods-strip button { border: 1px solid #d9e2ef; background: #f6f8fb; color: #172033; border-radius: 999px; padding: 7px 10px; font-size: 12px; font-weight: 800; }
        .checkout-methods { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
        .checkout-stage.mobile .checkout-methods { grid-template-columns: 1fr; }
        .checkout-methods button { display: grid; grid-template-columns: 24px minmax(0, 1fr); gap: 2px 10px; text-align: left; border: 1px solid #d9e2ef; background: #ffffff; color: #0f172a; border-radius: 18px; padding: 14px; cursor: pointer; min-height: 70px; }
        .checkout-methods button.active { border-color: ${themeColor}; background: #eef6ff; box-shadow: inset 0 0 0 1px ${themeColor}; }
        .checkout-methods b { margin-left: 6px; border: 1px solid ${accentColor}; color: ${accentColor}; border-radius: 999px; padding: 1px 6px; font-size: 10px; }
        .checkout-methods small { grid-column: 2; color: #64748b; line-height: 1.35; }
        .checkout-method-panel { display: grid; grid-template-columns: 44px minmax(0, 1fr); gap: 12px; align-items: center; border: 1px solid #d9e2ef; background: #f6f8fb; border-radius: 20px; padding: 14px; color: #172033; }
        .checkout-method-panel strong { display: block; font-size: 15px; margin-bottom: 3px; }
        .checkout-method-panel span { color: #64748b; font-size: 12px; line-height: 1.4; }
        .checkout-method-panel.success { border-color: rgba(35, 110, 62, 0.36); background: #eef8ef; color: #236e3e; }
        .checkout-method-panel.success b { display: inline-flex; border: 1px solid rgba(35, 110, 62, 0.32); border-radius: 999px; padding: 1px 7px; background: #fff9ec; color: #236e3e; font-size: 11px; }
        .checkout-method-panel.processing { border-color: rgba(235, 0, 27, 0.28); }
        .checkout-method-panel.failed { border-color: rgba(200, 31, 47, 0.34); background: #fff0f0; color: #9f1728; }
        .upi-qr { width: 72px; height: 72px; border-radius: 18px; display: grid; place-items: center; background: #ffffff; border: 1px solid #d9e2ef; }
        .upi-panel { grid-template-columns: 72px minmax(0, 1fr); }
        .mini-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
        .mini-actions button { border: 1px solid #d9e2ef; background: #ffffff; color: #172033; border-radius: 999px; padding: 5px 9px; font-size: 11px; font-weight: 800; }
        .checkout-stage.mobile .checkout-method-panel { grid-template-columns: 34px minmax(0, 1fr); padding: 12px; }
        .checkout-stage.mobile .upi-panel { grid-template-columns: 58px minmax(0, 1fr); }
        .checkout-stage.mobile .upi-qr { width: 58px; height: 58px; }
        .simulation-timeline { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
        .simulation-timeline div { border: 1px solid #d9e2ef; background: #ffffff; border-radius: 16px; padding: 9px; display: grid; gap: 2px; min-width: 0; }
        .simulation-timeline span { color: #64748b; font-size: 11px; font-weight: 800; }
        .simulation-timeline strong { color: #172033; font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .fee-breakup { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 6px 10px; border: 1px solid #d9e2ef; border-radius: 18px; padding: 12px; background: #ffffff; font-size: 12px; }
        .fee-breakup span { color: #64748b; }
        .recommended-method { border-radius: 999px; background: color-mix(in srgb, ${accentColor} 12%, #ffffff); color: ${accentColor}; border: 1px solid ${accentColor}; padding: 8px 12px; font-weight: 800; font-size: 12px; }
        .checkout-pay-button { border: none; border-radius: 999px; padding: 14px 18px; font-weight: 800; background: ${themeColor}; color: #fff; cursor: pointer; min-height: 50px; }
        .checkout-pay-button:disabled { opacity: 0.74; cursor: progress; }
        .retry-button { border: 1px solid ${themeColor}; color: ${themeColor}; background: #ffffff; border-radius: 999px; min-height: 44px; font-weight: 800; cursor: pointer; }
        .checkout-secure { display: inline-flex; gap: 7px; align-items: center; justify-content: center; color: #64748b; font-size: 12px; }
        .payment-flow-panel { margin-top: 18px; }
        .payment-flow-panel h3 { margin-bottom: 14px; }
        .payment-flow-grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 10px; }
        .payment-flow-grid article { border: 1px solid #d5c9af; background: #fff9ec; color: #141413; border-radius: 28px; padding: 14px; display: grid; gap: 10px; }
        .payment-flow-grid article > strong { color: #eb001b; font-size: 13px; }
        .payment-flow-grid h4 { margin: 0 0 4px; font-size: 14px; }
        .payment-flow-grid p { margin: 0; color: #6b6862; font-size: 12px; line-height: 1.45; }
        .payment-flow-grid span { display: inline-flex; margin-top: 10px; border: 1px solid #d5c9af; border-radius: 999px; padding: 3px 8px; color: #6b6862; font-size: 11px; }
        @media (max-width: 980px) { .preview-shell { grid-template-columns: 1fr; } .preview-controls { position: static; max-width: none; } .checkout-stage { min-height: auto; } .checkout-settings-tabs { grid-template-columns: 1fr; } }
        @media (max-width: 520px) { .checkout-preview-page .page-header { border-radius: 22px; padding: 18px; } .preview-controls { padding: 16px !important; border-radius: 18px !important; } .method-config-row { grid-template-columns: 1fr 40px 40px; } .method-config-row button:last-child { grid-column: 1 / -1; } .checkout-stage { padding: 10px; } .checkout-stage.mobile { width: 100%; box-shadow: inset 0 0 0 6px #312d28; } .checkout-stage.mobile .browser-bar, .checkout-stage.mobile .checkout-card { max-width: 100%; } .checkout-card { padding: 16px; } .simulation-timeline { grid-template-columns: 1fr; } }
        @media (max-width: 980px) { .payment-flow-grid { grid-template-columns: 1fr; } }
      `}</style>
    </div>
  );
}
