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
          
          <div className="checkout-viewport-content">
            {settings.styling?.layout === 'standard' && (
              <div className="simulated-merchant-bg">
                <div className="merchant-bg-header">
                  <span className="logo-placeholder">🛍️</span>
                  <strong>Awesome Shop</strong>
                </div>
                <div className="merchant-bg-body">
                  <div className="bg-summary-card">
                    <div className="bg-line-short"></div>
                    <div className="bg-line-long"></div>
                  </div>
                  <div className="bg-line-medium"></div>
                  <div className="bg-line-long"></div>
                </div>
              </div>
            )}

            <div className="checkout-card-wrapper">
              <div className="checkout-card" style={{ borderRadius: settings.styling?.border_style === 'sharp' ? '0' : `${cornerRadius}px`, fontFamily: `${settings.styling?.font_family || 'IBM Plex Sans'}, sans-serif` }}>
                <div className="test-mode-ribbon">
                  <span>TEST MODE</span>
                  <strong>{simulationAction === 'success' ? 'Approve payment' : simulationAction === 'fail' ? 'Decline payment' : 'Keep pending'}</strong>
                </div>

                {paymentState === 'success' ? (
                  <div className="success-screen-overlay animate-fade-in">
                    <div className="success-icon-container">
                      <div className="success-ripple"></div>
                      <div className="success-circle-glow">
                        <CheckCircle2 className="success-checkmark-icon" size={68} />
                      </div>
                    </div>
                    <h2>Payment Successful</h2>
                    <p className="success-subtitle">Your transaction has been processed securely.</p>
                    <div className="success-amount-paid" style={{ color: themeColor }}>
                      {formatPaise(parseInrToPaise(amount || '0') || 0)}
                    </div>
                    <div className="success-receipt-details">
                      <div className="receipt-row">
                        <span>Payment ID</span>
                        <strong>{paymentResult?.payment_id || `pay_${Math.random().toString(36).substring(2, 9)}`}</strong>
                      </div>
                      <div className="receipt-row">
                        <span>Method</span>
                        <strong>{selectedLabel} ({paymentResult?.mode_badge || 'TEST'})</strong>
                      </div>
                      <div className="receipt-row">
                        <span>Connector</span>
                        <strong>{paymentResult?.connector || 'simulator'}</strong>
                      </div>
                    </div>
                    <button className="success-done-btn animate-bounce-subtle" style={{ background: themeColor }} onClick={() => { setPaymentState('idle'); setPaymentResult(null); }}>
                      Back to Demo Store
                    </button>
                  </div>
                ) : (
                  <div className="checkout-content-grid">
                    {/* Left/Top Column: Branding & Info */}
                    <div className="checkout-brand-section">
                      <div className="checkout-brand">
                        <img src={settings.styling?.logo_url || '/feelpay.png'} alt="FeelPay" />
                        <div>
                          <strong>{merchantName}</strong>
                          <span>{merchantTagline}</span>
                        </div>
                      </div>

                      <div className="checkout-amount" style={{ background: `color-mix(in srgb, ${themeColor} 4%, #f8fafc)` }}>
                        <span className="label">Amount payable</span>
                        <strong className="amount">{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
                        <div className="customer-details">
                          {settings.features?.collect_email && (
                            <small className="detail-item">
                              <span>Email:</span> {customer} {settings.features?.readonly_email ? '🔒' : ''}
                            </small>
                          )}
                          {settings.features?.collect_contact && (
                            <small className="detail-item">
                              <span>Phone:</span> +91 99999 99999 {settings.features?.readonly_contact ? '🔒' : ''}
                            </small>
                          )}
                        </div>
                      </div>

                      {settings.features?.show_mandate_summary && (
                        <div className="checkout-summary-strip">
                          <span>Order Details</span>
                          <strong>INR • {settings.features?.default_language || 'English'}</strong>
                        </div>
                      )}
                      
                      <div className="checkout-left-footer">
                        <div className="checkout-secure">
                          <ShieldCheck size={14} style={{ color: accentColor }} />
                          {settings.styling?.show_trusted_badge === false ? 'Test checkout preview' : 'PCI-ready vault, signed webhooks, live/test isolation'}
                        </div>
                      </div>
                    </div>

                    {/* Right/Bottom Column: Payment Processing */}
                    <div className="checkout-payment-section">
                      {settings.features?.show_message_banner && (
                        <div className="checkout-banner" style={{ borderColor: themeColor, background: `color-mix(in srgb, ${themeColor} 6%, #ffffff)` }}>
                          {settings.features?.banner_text || 'Secure checkout is ready.'}
                        </div>
                      )}

                      <div className="simulation-switcher">
                        {(['success', 'fail', 'processing'] as SimulationAction[]).map((action) => (
                          <button
                            key={action}
                            type="button"
                            className={simulationAction === action ? 'active' : ''}
                            style={{ borderColor: simulationAction === action ? themeColor : '#d9e2ef', color: simulationAction === action ? themeColor : '#64748b' }}
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

                      {settings.features?.show_saved_methods && (
                        <div className="saved-methods-strip">
                          <span className="strip-title">Saved</span>
                          <div className="strip-buttons">
                            <button type="button" onClick={() => selectMethod('card')}>💳 HDFC Visa • 1111</button>
                            <button type="button" onClick={() => selectMethod('upi')}>📱 buyer@upi</button>
                          </div>
                        </div>
                      )}

                      <div className="checkout-methods-container">
                        <span className="section-title">Select Payment Method</span>
                        <div className="checkout-methods">
                          {visibleMethods.map((method) => {
                            const Icon = ICONS[method.method_type];
                            const active = selectedMethod?.method_type === method.method_type;
                            return (
                              <button key={method.method_type} className={`method-btn ${active ? 'active' : ''}`} style={{ borderColor: active ? themeColor : '#d9e2ef' }} onClick={() => selectMethod(method.method_type)}>
                                <div className="method-btn-title">
                                  <Icon size={16} style={{ color: active ? themeColor : '#64748b' }} />
                                  <span>{method.config?.display_name || LABELS[method.method_type]}</span>
                                  {promotedMethods.includes(method.method_type) && <b className="promo-badge" style={{ color: accentColor, borderColor: accentColor }}>PROMOTED</b>}
                                </div>
                                {method.config?.checkout_note && <span className="method-note">{method.config?.checkout_note}</span>}
                              </button>
                            );
                          })}
                        </div>
                      </div>

                      <div className="active-experience-container">
                        {renderMethodExperience()}
                      </div>

                      {isPromoted && <div className="recommended-method" style={{ color: accentColor, borderColor: accentColor, background: `color-mix(in srgb, ${accentColor} 12%, #ffffff)` }}>Recommended for better success rate</div>}

                      <div className="checkout-actions">
                        <button className="checkout-pay-button" style={{ background: themeColor }} onClick={() => void runPreviewPayment()} disabled={paymentState === 'processing'}>
                          {paymentState === 'processing' ? 'Processing...' : `${settings.styling?.button_text || 'Pay securely'} with ${selectedLabel}`}
                        </button>
                        {settings.features?.retry_enabled && paymentResult?.error && (
                          <button className="retry-button" style={{ color: themeColor, borderColor: themeColor }} type="button" onClick={() => void runPreviewPayment()}>
                            Retry with suggested method
                          </button>
                        )}
                      </div>

                      <div className="simulation-timeline" aria-label="Payment simulation timeline">
                        {simulationSteps.map((step) => (
                          <div key={step.label} className="timeline-step">
                            <span>{step.label}</span>
                            <strong>{step.value}</strong>
                          </div>
                        ))}
                      </div>

                      {settings.features?.show_fee_breakup && (
                        <div className="fee-breakup">
                          <span>Amount</span><strong>{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
                          <span>Gateway fee</span><strong>₹0.00</strong>
                          <span className="total-label">Total</span><strong className="total-val">{formatPaise(parseInrToPaise(amount || '0') || 0)}</strong>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
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
        .preview-shell { display: grid; grid-template-columns: minmax(300px, 360px) minmax(0, 1fr); gap: 24px; align-items: start; margin-top: 20px; }
        .preview-controls { display: grid; gap: 14px; position: sticky; top: 20px; background: #ffffff !important; border-radius: 20px !important; box-shadow: 0 4px 20px rgba(0,0,0,0.02); }
        .preview-controls h3 { margin: 0; font-size: 20px; font-weight: 800; color: #0f172a; }
        .checkout-settings-tabs { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 6px; }
        .checkout-settings-tabs button { border: 1px solid #d9e2ef; background: #ffffff; border-radius: 999px; padding: 8px 10px; font-size: 11px; font-weight: 700; color: #64748b; cursor: pointer; transition: all 0.2s ease; }
        .checkout-settings-tabs button.active { border-color: ${themeColor}; color: #0b72e7; background: #eef6ff; }
        .checkout-settings-grid { display: grid; gap: 12px; }
        .checkout-settings-grid label { display: grid; gap: 6px; font-size: 13px; color: #475569; font-weight: 700; }
        .checkout-settings-grid label:has(input[type='checkbox']) { min-height: 44px; display: flex; flex-direction: row; align-items: center; gap: 10px; border: 1px solid #d9e2ef; background: #f8fafc; border-radius: 16px; padding: 10px 12px; color: #1e293b; cursor: pointer; }
        .checkout-settings-grid input[type='checkbox'] { width: 18px; height: 18px; margin: 0; accent-color: ${themeColor}; flex: 0 0 auto; }
        .checkout-audit { display: grid; gap: 6px; border-top: 1px dashed #d9e2ef; padding-top: 12px; }
        .checkout-audit > span { font-size: 12px; color: #64748b; font-weight: 700; }
        .checkout-audit-item { font-size: 12px; color: #0f172a; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 10px; padding: 6px 10px; }
        .preview-merchant-name { color: #0f172a; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 999px; padding: 10px 14px; font-weight: 800; font-size: 16px; text-align: center; }
        .preview-controls label { display: flex; flex-direction: column; gap: 6px; color: #475569; font-size: 13px; }
        .preview-controls input,
        .preview-controls select { background: #ffffff; border: 1px solid #d9e2ef; color: #0f172a; border-radius: 999px; padding: 10px 14px; min-width: 0; min-height: 42px; font-size: 14px; outline: none; transition: border 0.2s; }
        .preview-controls input:focus, .preview-controls select:focus { border-color: ${themeColor}; }
        .preview-method-list { display: grid; gap: 6px; }
        .preview-method-list span { color: #64748b; font-size: 12px; font-weight: 700; }
        .preview-method-list button { text-align: left; background: #ffffff; border: 1px solid #d9e2ef; color: #64748b; border-radius: 999px; padding: 8px 14px; cursor: pointer; font-size: 13px; transition: all 0.2s; }
        .preview-method-list button.selected { border-color: ${themeColor}; color: #0b72e7; background: #eef6ff; font-weight: 700; }
        .method-config-builder { display: grid; gap: 8px; }
        .method-config-builder > span { color: #475569; font-size: 12px; font-weight: 800; }
        .method-config-row { display: grid; grid-template-columns: minmax(0, 1fr) 36px 36px minmax(78px, auto); gap: 6px; align-items: center; border: 1px solid #d9e2ef; background: #ffffff; border-radius: 14px; padding: 8px; }
        .method-config-row label { display: inline-flex; flex-direction: row; align-items: center; gap: 7px; color: #1e293b; font-size: 13px; font-weight: 600; cursor: pointer; }
        .method-config-row button { border: 1px solid #d9e2ef; background: #f8fafc; color: #0f172a; border-radius: 999px; min-height: 32px; cursor: pointer; font-weight: 800; font-size: 12px; display: inline-flex; align-items: center; justify-content: center; transition: all 0.2s; }
        .method-config-row button.active { border-color: ${themeColor}; color: ${themeColor}; background: #ffffff; }
        .chip-field { display: grid; gap: 6px; }
        .chip-field > span { color: #475569; font-size: 13px; font-weight: 800; }
        .settings-chip-list { display: flex; flex-wrap: wrap; gap: 6px; border: 1px solid #d9e2ef; background: #f8fafc; border-radius: 16px; padding: 8px; }
        .settings-chip-list button { border: 1px solid #d9e2ef; background: #ffffff; color: #0f172a; border-radius: 999px; min-height: 34px; padding: 6px 12px; cursor: pointer; font-weight: 700; font-size: 12px; transition: all 0.2s; }
        .settings-chip-list button.active { border-color: ${themeColor}; color: ${themeColor}; background: #eef6ff; }
        
        /* Checkout stage viewport container */
        .checkout-stage { min-height: 780px; border: 1px solid #d9e2ef; border-radius: 34px; background: #f8fafc; padding: 24px; display: flex; flex-direction: column; align-items: center; overflow: hidden; justify-content: start; transition: all 0.3s ease; }
        .checkout-stage.browser { width: 100%; }
        .checkout-stage.laptop { width: min(100%, 980px); margin: 0 auto; padding: 18px 18px 54px; border-radius: 26px; background: #0f172a; position: relative; }
        .checkout-stage.laptop::after { content: ''; position: absolute; left: 50%; bottom: 14px; transform: translateX(-50%); width: 30%; height: 6px; border-radius: 999px; background: #334155; }
        .laptop-lid-dot { width: 6px; height: 6px; border-radius: 999px; background: #475569; margin-bottom: 10px; }
        .checkout-stage.mobile { width: 380px; min-height: 760px; margin: 0 auto; padding: 16px; border-radius: 40px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25); background: #0f172a; border: 8px solid #1e293b; }
        .browser-bar { width: 100%; max-width: 820px; height: 38px; border-radius: 20px 20px 0 0; background: #ffffff; display: flex; align-items: center; gap: 7px; padding: 0 16px; color: #475569; border: 1px solid #e2e8f0; border-bottom: none; }
        .browser-bar span { width: 8px; height: 8px; border-radius: 99px; background: #cbd5e1; display: inline-block; }
        .browser-bar span:nth-child(1) { background: #f87171; }
        .browser-bar span:nth-child(2) { background: #fbbf24; }
        .browser-bar span:nth-child(3) { background: #34d399; }
        .browser-bar strong { margin-left: auto; margin-right: auto; font-size: 11px; font-weight: 600; color: #94a3b8; letter-spacing: 0.05em; }
        .checkout-stage.laptop .browser-bar, .checkout-stage.laptop .checkout-viewport-content { max-width: 880px; }
        .checkout-stage.mobile .browser-bar, .checkout-stage.mobile .checkout-viewport-content { max-width: 326px; }
        .checkout-stage.mobile .browser-bar { display: none; }
        
        .checkout-viewport-content { width: 100%; display: flex; flex-direction: column; position: relative; }
        .checkout-stage.mobile .checkout-card-wrapper { border-radius: 20px; overflow: hidden; }
        
        /* Simulated backdrop for Standard Modal Layout */
        .simulated-merchant-bg { width: 100%; height: 500px; background: #ffffff; border: 1px solid #e2e8f0; padding: 20px; display: flex; flex-direction: column; gap: 16px; filter: blur(2px); opacity: 0.35; user-select: none; pointer-events: none; }
        .merchant-bg-header { display: flex; align-items: center; gap: 10px; border-bottom: 1px solid #e2e8f0; padding-bottom: 12px; }
        .logo-placeholder { font-size: 24px; }
        .merchant-bg-body { display: grid; gap: 12px; margin-top: 10px; }
        .bg-summary-card { background: #f8fafc; border-radius: 12px; padding: 16px; border: 1px solid #e2e8f0; }
        .bg-line-short { height: 10px; width: 80px; background: #e2e8f0; border-radius: 4px; margin-bottom: 8px; }
        .bg-line-medium { height: 10px; width: 180px; background: #e2e8f0; border-radius: 4px; margin-bottom: 8px; }
        .bg-line-long { height: 10px; width: 100%; background: #e2e8f0; border-radius: 4px; }
        
        /* Modal centering wrapper for Standard template */
        .template-standard .checkout-card-wrapper { position: absolute; top: 5%; left: 50%; transform: translateX(-50%); width: 90%; max-width: 520px; z-index: 10; box-shadow: 0 20px 40px -15px rgba(0,0,0,0.3); border-radius: 24px; overflow: hidden; }
        .template-compact .checkout-card-wrapper { max-width: 420px; margin: 0 auto; width: 100%; }
        .template-split .checkout-card-wrapper { width: 100%; }

        /* The actual checkout card styling */
        .checkout-card { background: ${checkoutBg}; color: #0f172a; border: 1px solid #d9e2ef; padding: 24px; position: relative; box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04); transition: all 0.3s ease; }
        .test-mode-ribbon { display: flex; align-items: center; justify-content: space-between; gap: 10px; border: 1px solid #bfdbfe; background: #eff6ff; color: #1d4ed8; border-radius: 12px; padding: 8px 12px; font-size: 11px; margin-bottom: 16px; }
        .test-mode-ribbon span { font-weight: 900; letter-spacing: 0.08em; }
        
        /* Grid Layout */
        .checkout-content-grid { display: grid; gap: 24px; }
        .template-split:not(.mobile) .checkout-content-grid { grid-template-columns: 280px 1fr; }
        
        .checkout-brand-section { display: flex; flex-direction: column; gap: 16px; border-right: 1px solid rgba(0, 0, 0, 0.06); padding-right: 16px; }
        .template-standard .checkout-brand-section, .template-compact .checkout-brand-section, .checkout-stage.mobile .checkout-brand-section { border-right: none; padding-right: 0; }
        
        .checkout-brand { display: flex; align-items: center; gap: 12px; }
        .checkout-brand img { width: 48px; height: 48px; border-radius: 14px; object-fit: contain; border: 1px solid #e2e8f0; background: #fff; padding: 2px; }
        .checkout-brand strong { font-size: 16px; color: #0f172a; font-weight: 800; display: block; }
        .checkout-brand span { color: #64748b; font-size: 10px; text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600; }
        
        .checkout-amount { border-radius: 16px; padding: 16px; display: grid; gap: 4px; }
        .checkout-amount .label { color: #64748b; font-size: 11px; font-weight: 700; text-transform: uppercase; }
        .checkout-amount .amount { font-size: 28px; font-weight: 800; color: #0f172a; }
        .customer-details { display: grid; gap: 2px; margin-top: 4px; border-top: 1px solid rgba(0, 0, 0, 0.05); padding-top: 6px; }
        .detail-item { color: #475569; font-size: 11px; display: block; }
        .detail-item span { color: #94a3b8; font-weight: 600; }
        
        .checkout-summary-strip { display: flex; align-items: center; justify-content: space-between; border: 1px dashed #d9e2ef; border-radius: 12px; padding: 8px 12px; color: #64748b; font-size: 11px; }
        .checkout-summary-strip strong { color: #0f172a; }
        
        .checkout-left-footer { margin-top: auto; padding-top: 10px; }
        .checkout-secure { display: flex; gap: 6px; align-items: center; color: #64748b; font-size: 10px; font-weight: 600; line-height: 1.3; }
        
        .checkout-payment-section { display: flex; flex-direction: column; gap: 16px; }
        .checkout-banner { border: 1px solid #d9e2ef; border-radius: 12px; padding: 10px 12px; font-size: 12px; font-weight: 700; text-align: center; }
        .simulation-switcher { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 6px; }
        .simulation-switcher button { border: 1px solid #d9e2ef; background: #ffffff; color: #64748b; border-radius: 10px; min-height: 32px; cursor: pointer; font-weight: 700; font-size: 12px; transition: all 0.2s; }
        .simulation-switcher button.active { background: #eef6ff; font-weight: 800; }
        
        .saved-methods-strip { display: flex; flex-direction: column; gap: 6px; }
        .saved-methods-strip .strip-title { color: #94a3b8; font-size: 10px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.05em; }
        .strip-buttons { display: flex; flex-wrap: wrap; gap: 6px; }
        .strip-buttons button { border: 1px solid #e2e8f0; background: #ffffff; color: #334155; border-radius: 10px; padding: 6px 10px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.2s; }
        .strip-buttons button:hover { background: #f8fafc; border-color: #cbd5e1; }
        
        .checkout-methods-container { display: flex; flex-direction: column; gap: 8px; }
        .checkout-methods-container .section-title { color: #94a3b8; font-size: 10px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.05em; }
        .checkout-methods { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
        .template-compact .checkout-methods, .checkout-stage.mobile .checkout-methods { grid-template-columns: 1fr; }
        
        .method-btn { display: flex; flex-direction: column; text-align: left; border: 1px solid #d9e2ef; background: #ffffff; color: #0f172a; border-radius: 14px; padding: 10px 12px; cursor: pointer; transition: all 0.2s; }
        .method-btn.active { background: #eef6ff; box-shadow: inset 0 0 0 1px ${themeColor}; }
        .method-btn-title { display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 13px; width: 100%; }
        .promo-badge { font-size: 8px; border: 1px solid; border-radius: 4px; padding: 1px 4px; margin-left: auto; letter-spacing: 0.05em; }
        .method-note { color: #64748b; font-size: 10px; margin-top: 2px; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        
        .active-experience-container { margin-top: 2px; }
        .checkout-method-panel { display: flex; gap: 12px; align-items: center; border: 1px solid #d9e2ef; background: #f8fafc; border-radius: 14px; padding: 12px; color: #1e293b; }
        .checkout-method-panel strong { display: block; font-size: 13px; font-weight: 700; margin-bottom: 2px; }
        .checkout-method-panel span { color: #64748b; font-size: 11px; line-height: 1.4; }
        .checkout-method-panel.processing { border-color: #ffedd5; background: #fffbeb; color: #d97706; }
        .checkout-method-panel.failed { border-color: #fee2e2; background: #fef2f2; color: #9f1728; }
        .upi-qr { width: 50px; height: 50px; border-radius: 10px; display: grid; place-items: center; background: #ffffff; border: 1px solid #e2e8f0; flex-shrink: 0; }
        .upi-panel { display: flex; gap: 12px; align-items: center; }
        .mini-actions { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; }
        .mini-actions button { border: 1px solid #d9e2ef; background: #ffffff; color: #0f172a; border-radius: 6px; padding: 3px 6px; font-size: 10px; font-weight: 700; cursor: pointer; }
        
        .checkout-actions { display: grid; gap: 8px; margin-top: 8px; }
        .checkout-pay-button { border: none; border-radius: 999px; padding: 12px 18px; font-weight: 800; color: #ffffff; cursor: pointer; min-height: 44px; font-size: 14px; transition: opacity 0.2s; }
        .checkout-pay-button:disabled { opacity: 0.6; cursor: progress; }
        .retry-button { border: 1px solid #cbd5e1; background: #ffffff; border-radius: 999px; min-height: 40px; font-weight: 800; cursor: pointer; font-size: 13px; transition: all 0.2s; }
        .recommended-method { border-radius: 999px; padding: 6px 12px; font-weight: 800; font-size: 11px; text-align: center; border: 1px solid; }
        
        .simulation-timeline { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 6px; margin-top: 8px; }
        .timeline-step { border: 1px solid #e2e8f0; background: #f8fafc; border-radius: 10px; padding: 6px; display: grid; gap: 1px; min-width: 0; }
        .timeline-step span { color: #94a3b8; font-size: 9px; font-weight: 800; text-transform: uppercase; }
        .timeline-step strong { color: #334155; font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        
        .fee-breakup { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 4px 10px; border: 1px solid #e2e8f0; border-radius: 12px; padding: 10px; background: #ffffff; font-size: 11px; margin-top: 8px; }
        .fee-breakup span { color: #64748b; }
        .fee-breakup .total-label { font-weight: 800; color: #0f172a; }
        .fee-breakup .total-val { font-weight: 800; color: #0b72e7; }
        
        /* Success Screen Animation & Design */
        .success-screen-overlay { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 30px 10px; text-align: center; animation: fadeIn 0.4s ease-out; }
        .success-icon-container { position: relative; width: 100px; height: 100px; display: flex; align-items: center; justify-content: center; margin-bottom: 20px; }
        .success-circle-glow { width: 84px; height: 84px; border-radius: 50%; background: #e6fcf5; display: flex; align-items: center; justify-content: center; z-index: 2; box-shadow: 0 10px 25px -5px rgba(16, 185, 129, 0.2); }
        .success-checkmark-icon { color: #10b981; animation: scaleIn 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275) both; }
        .success-ripple { position: absolute; width: 100px; height: 100px; border-radius: 50%; border: 3px solid #10b981; animation: successRipple 1.4s cubic-bezier(0.1, 0.8, 0.3, 1) infinite; opacity: 0; }
        
        .success-screen-overlay h2 { font-size: 22px; font-weight: 800; color: #1e293b; margin: 0 0 4px; }
        .success-subtitle { color: #64748b; font-size: 13px; margin: 0 0 16px; }
        .success-amount-paid { font-size: 32px; font-weight: 900; margin-bottom: 20px; }
        .success-receipt-details { width: 100%; max-width: 320px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 16px; padding: 14px; display: grid; gap: 8px; margin-bottom: 24px; text-align: left; }
        .receipt-row { display: flex; justify-content: space-between; font-size: 12px; }
        .receipt-row span { color: #64748b; }
        .receipt-row strong { color: #0f172a; }
        .success-done-btn { border: none; border-radius: 999px; padding: 12px 30px; font-weight: 800; color: #ffffff; cursor: pointer; font-size: 14px; min-width: 160px; box-shadow: 0 4px 14px rgba(0, 0, 0, 0.1); }
        
        @keyframes scaleIn {
          0% { transform: scale(0); opacity: 0; }
          100% { transform: scale(1); opacity: 1; }
        }
        @keyframes successRipple {
          0% { transform: scale(0.85); opacity: 0.6; }
          100% { transform: scale(1.4); opacity: 0; }
        }
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(10px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .animate-fade-in { animation: fadeIn 0.4s ease-out; }
        
        @keyframes bounceSubtle {
          0%, 100% { transform: translateY(0); }
          50% { transform: translateY(-3px); }
        }
        .animate-bounce-subtle { animation: bounceSubtle 2s infinite ease-in-out; }

        @media (max-width: 980px) { 
          .preview-shell { grid-template-columns: 1fr; } 
          .preview-controls { position: static; max-width: none; } 
          .checkout-stage { min-height: auto; } 
          .checkout-settings-tabs { grid-template-columns: 1fr; } 
        }
        @media (max-width: 520px) { 
          .checkout-preview-page .page-header { border-radius: 22px; padding: 18px; } 
          .preview-controls { padding: 16px !important; border-radius: 18px !important; } 
          .method-config-row { grid-template-columns: 1fr 36px 36px; } 
          .method-config-row button:last-child { grid-column: 1 / -1; } 
          .checkout-stage { padding: 10px; } 
          .checkout-stage.mobile { width: 100%; border: 4px solid #1e293b; } 
          .checkout-card { padding: 16px; } 
          .simulation-timeline { grid-template-columns: 1fr; } 
        }
        @media (max-width: 980px) { .payment-flow-grid { grid-template-columns: 1fr; } }
      `}</style>
    </div>
  );
}
