import { FormEvent, ReactNode, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  BadgeCheck,
  Banknote,
  Cable,
  ClipboardList,
  Copy,
  Eye,
  FileSearch,
  KeyRound,
  Landmark,
  Radar,
  Receipt,
  RefreshCw,
  Route,
  ShieldCheck,
  SlidersHorizontal,
  Upload,
} from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

const API = '/admin/payment-ops';

type GenericValue = string | number | boolean | null | string[] | Record<string, unknown>;
type GenericRecord = Record<string, GenericValue>;

type Provider = {
  id: string;
  name: string;
  code: string;
  provider_type: string;
  environment: string;
  status: string;
  priority: number;
  failover_enabled: boolean;
  webhook_url?: string | null;
  health_status: string;
  success_rate: number;
  attempt_count: number;
  api_key_status: string;
  api_secret_status: string;
  webhook_secret_status: string;
  last_health_check_at?: string | null;
};

type Credential = {
  id: string;
  provider_id: string;
  provider_name: string;
  environment: string;
  secret_type: string;
  masked_value: string;
  status: string;
  last_validated_at?: string | null;
  last_rotated_at?: string | null;
  rotation_due_at?: string | null;
  description?: string | null;
};

type Dashboard = {
  total_gmv: number;
  successful_payments: number;
  failed_payments: number;
  payment_success_rate: number;
  pending_refunds: number;
  pending_settlements: number;
  reconciliation_mismatches: number;
  webhook_failures: number;
  company_revenue_total: number;
};

type WhiteLabelConfig = {
  brand_name?: string;
  checkout_logo_url?: string | null;
  checkout_theme_color?: string;
  payment_page_domain?: string | null;
  support_email?: string | null;
  support_phone?: string | null;
  terms_url?: string | null;
  privacy_url?: string | null;
  refund_policy_url?: string | null;
  statement_descriptor?: string | null;
  default_provider_id?: string | null;
  fallback_provider_id?: string | null;
  default_payment_methods?: string[];
  default_country?: string;
  default_currency?: string;
  invoice_prefix?: string;
  receipt_prefix?: string;
};

type RoutingSimulation = {
  selected_provider?: string | null;
  selected_provider_code?: string | null;
  matched_routing_rule?: string | null;
  fallback_provider?: string | null;
  decision_reason: string;
  estimated_provider_fee: number;
  estimated_company_fee: number;
  risk_decision: string;
  provider_health_used: string;
  failover_applied: boolean;
};

type ConfirmState = {
  title: string;
  body: string;
  action: () => Promise<void>;
} | null;

type DrawerState = {
  title: string;
  body: GenericRecord;
} | null;

const PROVIDER_TYPES = ['mock', 'stripe', 'razorpay', 'adyen', 'cashfree', 'juspay', 'payu', 'manual_bank'];
const PROVIDER_STATUSES = ['active', 'disabled', 'maintenance'];
const PAYMENT_METHODS = ['card', 'upi', 'netbanking', 'wallet', 'emi', 'bank_transfer'];
const SECRET_TYPES = ['api_key', 'api_secret', 'webhook_secret', 'signing_secret', 'client_id', 'client_secret'];
const DEFAULT_PROVIDER_FORM = {
  name: 'Mock Provider',
  code: 'mock',
  provider_type: 'mock',
  status: 'active',
  environment: 'sandbox',
  priority: 10,
  supported_payment_methods: 'card,upi,netbanking,wallet',
  supported_currencies: 'INR,USD',
  supported_countries: 'IN,US',
  webhook_url: 'https://api.feelpay.local/v1/webhooks/mock',
  failover_enabled: true,
};

const DEFAULT_WHITE_LABEL = {
  brand_name: 'FeelPay',
  checkout_logo_url: 'https://feelpay.local/logo.svg',
  checkout_theme_color: '#1DB954',
  payment_page_domain: 'checkout.feelpay.com',
  support_email: 'support@feelpay.local',
  support_phone: '+91 80 4000 8888',
  terms_url: 'https://feelpay.local/legal/terms',
  privacy_url: 'https://feelpay.local/legal/privacy',
  refund_policy_url: 'https://feelpay.local/legal/refunds',
  statement_descriptor: 'FEELPAY*SETTLE',
  default_provider_id: '10000000-0000-0000-0000-000000000001',
  fallback_provider_id: '10000000-0000-0000-0000-000000000002',
  default_payment_methods: 'upi,card,netbanking',
  default_country: 'IN',
  default_currency: 'INR',
  invoice_prefix: 'INV-FP',
  receipt_prefix: 'RCP-FP',
};

function money(value: number) {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 0,
  }).format(value / 100);
}

function list(value: string) {
  return value.split(',').map(item => item.trim()).filter(Boolean);
}

function dateTime(value?: string | null) {
  return value ? new Date(value).toLocaleString() : 'Pending Execution (T+1)';
}

function asRecords(value: { items?: GenericRecord[] }) {
  return value.items || [];
}

function badgeKind(value: string) {
  if (['active', 'healthy', 'verified', 'processed', 'successful', 'settled', 'matched', 'configured'].includes(value)) return 'success';
  if (['failed', 'down', 'dead_letter', 'rejected', 'missing', 'expired'].includes(value)) return 'failed';
  return 'pending';
}

function fieldId(label: string) {
  return label.toLowerCase().replace(/[^a-z0-9]+/g, '-');
}

export default function CredentialManagement() {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [webhookConfigs, setWebhookConfigs] = useState<GenericRecord[]>([]);
  const [webhooks, setWebhooks] = useState<GenericRecord[]>([]);
  const [routingRules, setRoutingRules] = useState<GenericRecord[]>([]);
  const [bankIntegrations, setBankIntegrations] = useState<GenericRecord[]>([]);
  const [refunds, setRefunds] = useState<GenericRecord[]>([]);
  const [settlements, setSettlements] = useState<GenericRecord[]>([]);
  const [riskRules, setRiskRules] = useState<GenericRecord[]>([]);
  const [reconciliation, setReconciliation] = useState<GenericRecord[]>([]);
  const [auditLogs, setAuditLogs] = useState<GenericRecord[]>([]);
  const [whiteLabel, setWhiteLabel] = useState(DEFAULT_WHITE_LABEL);
  const [providerForm, setProviderForm] = useState(DEFAULT_PROVIDER_FORM);
  const [credentialForm, setCredentialForm] = useState({
    provider_id: '10000000-0000-0000-0000-000000000001',
    environment: 'sandbox',
    secret_type: 'api_secret',
    value: 'sec_live_9f838e818817290192',
    rotation_due_at: '2026-12-31T23:59:59Z',
    description: 'Primary production gateway authentication secret',
  });
  const [routingForm, setRoutingForm] = useState({
    payment_method: 'upi',
    currency: 'INR',
    country: 'IN',
    amount: 10000,
    business_account: 'cd317088-a9ef-495c-ab3e-2a88603c54ac',
    customer_region: 'IN-MH',
    risk_level: 'low',
  });
  const [simulation, setSimulation] = useState<RoutingSimulation | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [confirm, setConfirm] = useState<ConfirmState>(null);
  const [drawer, setDrawer] = useState<DrawerState>(null);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const [
        dashboardData,
        providerData,
        credentialData,
        webhookConfigData,
        webhookData,
        routingData,
        bankData,
        refundData,
        settlementData,
        riskData,
        reconciliationData,
        auditData,
        whiteLabelData,
      ] = await Promise.all([
        apiRequest<Dashboard>('GET', `${API}/dashboard`),
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<{ items: Credential[] }>('GET', `${API}/credentials`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/webhook-configs`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/webhook-events?limit=10`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/routing-rules`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/bank-integrations`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/refunds?limit=10`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/settlements?limit=10`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/risk-rules`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/reconciliation`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/audit-logs`),
        apiRequest<WhiteLabelConfig>('GET', `${API}/white-label-config`),
      ]);

      setDashboard(dashboardData);
      setProviders(providerData.items || []);
      setCredentials(credentialData.items || []);
      setWebhookConfigs(asRecords(webhookConfigData));
      setWebhooks(asRecords(webhookData));
      setRoutingRules(asRecords(routingData));
      setBankIntegrations(asRecords(bankData));
      setRefunds(asRecords(refundData));
      setSettlements(asRecords(settlementData));
      setRiskRules(asRecords(riskData));
      setReconciliation(asRecords(reconciliationData));
      setAuditLogs(asRecords(auditData));
      setWhiteLabel({
        ...DEFAULT_WHITE_LABEL,
        ...whiteLabelData,
        checkout_logo_url: whiteLabelData.checkout_logo_url || '',
        checkout_theme_color: whiteLabelData.checkout_theme_color || DEFAULT_WHITE_LABEL.checkout_theme_color,
        payment_page_domain: whiteLabelData.payment_page_domain || '',
        support_email: whiteLabelData.support_email || '',
        support_phone: whiteLabelData.support_phone || '',
        terms_url: whiteLabelData.terms_url || '',
        privacy_url: whiteLabelData.privacy_url || '',
        refund_policy_url: whiteLabelData.refund_policy_url || '',
        statement_descriptor: whiteLabelData.statement_descriptor || '',
        default_provider_id: whiteLabelData.default_provider_id || '',
        fallback_provider_id: whiteLabelData.fallback_provider_id || '',
        default_payment_methods: (whiteLabelData.default_payment_methods || ['upi', 'card', 'netbanking']).join(','),
      });

      const firstProvider = providerData.items?.[0];
      if (!credentialForm.provider_id && firstProvider) {
        setCredentialForm(form => ({ ...form, provider_id: firstProvider.id }));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Payment operations load failed');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const activeProviders = useMemo(
    () => providers.filter(provider => provider.status === 'active').length,
    [providers],
  );

  const reconciliationStats = useMemo(() => {
    const matched = reconciliation.reduce((sum, row) => sum + Number(row.matched_count || 0), 0);
    const mismatches = reconciliation.reduce((sum, row) => sum + Number(row.mismatch_count || 0), 0);
    return { matched, mismatches };
  }, [reconciliation]);

  const showSuccess = async (text: string, action: () => Promise<void>) => {
    setMessage(null);
    setError(null);
    try {
      await action();
      setMessage(text);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    }
  };

  const saveProvider = async (event: FormEvent) => {
    event.preventDefault();
    await showSuccess('Provider saved.', () => apiRequest('POST', `${API}/providers`, {
      ...providerForm,
      supported_payment_methods: list(providerForm.supported_payment_methods),
      supported_currencies: list(providerForm.supported_currencies),
      supported_countries: list(providerForm.supported_countries),
    }));
  };

  const saveCredential = async (event: FormEvent) => {
    event.preventDefault();
    setConfirm({
      title: 'Save encrypted secret',
      body: 'Secret value will be encrypted. Full value will not be returned after save.',
      action: () => showSuccess('Credential saved and encrypted.', () => apiRequest('POST', `${API}/credentials`, {
        ...credentialForm,
        rotation_due_at: credentialForm.rotation_due_at || null,
      })),
    });
  };

  const saveWhiteLabel = async (event: FormEvent) => {
    event.preventDefault();
    await showSuccess('Platform checkout config saved.', () => apiRequest('PATCH', `${API}/white-label-config`, {
      brand_name: whiteLabel.brand_name,
      checkout_logo_url: whiteLabel.checkout_logo_url,
      checkout_theme_color: whiteLabel.checkout_theme_color,
      payment_page_domain: whiteLabel.payment_page_domain,
      support_email: whiteLabel.support_email,
      support_phone: whiteLabel.support_phone,
      terms_url: whiteLabel.terms_url,
      privacy_url: whiteLabel.privacy_url,
      refund_policy_url: whiteLabel.refund_policy_url,
      statement_descriptor: whiteLabel.statement_descriptor,
      default_provider_id: whiteLabel.default_provider_id || null,
      fallback_provider_id: whiteLabel.fallback_provider_id || null,
      default_payment_methods: list(whiteLabel.default_payment_methods),
      default_country: whiteLabel.default_country,
      default_currency: whiteLabel.default_currency,
      invoice_prefix: whiteLabel.invoice_prefix,
      receipt_prefix: whiteLabel.receipt_prefix,
    }));
  };

  const simulateRouting = async (event: FormEvent) => {
    event.preventDefault();
    setSimulation(null);
    try {
      const result = await apiRequest<RoutingSimulation>('POST', `${API}/routing/simulate`, routingForm);
      setSimulation(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Routing simulation failed');
    }
  };

  const confirmAction = (title: string, body: string, action: () => Promise<void>) => {
    setConfirm({ title, body, action });
  };

  const firstCredential = credentials[0];
  const firstProvider = providers[0];

  const runConfirmed = async () => {
    if (!confirm) return;
    const action = confirm.action;
    setConfirm(null);
    await action();
  };

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Payment Operations</h1>
          <p>Company-only gateway control plane for providers, routing, settlement, risk, and reconciliation</p>
        </div>
        <button className="btn-secondary" onClick={load} disabled={loading}>
          <RefreshCw size={16} /> Refresh
        </button>
      </div>

      {loading && <div className="alert alert-success">Loading company payment operations...</div>}
      {message && <div className="alert alert-success">{message}</div>}
      {error && <div className="alert alert-error">{error}</div>}

      <div className="stats-grid">
        <Metric label="GMV" value={dashboard ? money(dashboard.total_gmv) : '0'} icon={Banknote} />
        <Metric label="Success Rate" value={dashboard ? `${dashboard.payment_success_rate.toFixed(1)}%` : '0%'} icon={Activity} />
        <Metric label="Active Providers" value={`${activeProviders}/${providers.length}`} icon={Cable} />
        <Metric label="Webhook Failures" value={dashboard?.webhook_failures ?? 0} icon={AlertTriangle} />
        <Metric label="Pending Refunds" value={dashboard?.pending_refunds ?? 0} icon={Receipt} />
        <Metric label="Pending Settlements" value={dashboard?.pending_settlements ?? 0} icon={Landmark} />
        <Metric label="Reconciliation Mismatches" value={dashboard?.reconciliation_mismatches ?? 0} icon={FileSearch} />
        <Metric label="Company Revenue" value={dashboard ? money(dashboard.company_revenue_total) : '0'} icon={BadgeCheck} />
      </div>

      <div className="grid grid-2">
        <section className="card">
          <SectionTitle icon={<Cable size={18} />} title="Add Provider" />
          <form onSubmit={saveProvider}>
            <div className="filter-grid">
              <Field label="Provider Name"><input value={providerForm.name} onChange={event => setProviderForm({ ...providerForm, name: event.target.value })} /></Field>
              <Field label="Provider Code"><input value={providerForm.code} onChange={event => setProviderForm({ ...providerForm, code: event.target.value })} /></Field>
              <Field label="Provider Type"><Select value={providerForm.provider_type} options={PROVIDER_TYPES} onChange={value => setProviderForm({ ...providerForm, provider_type: value })} /></Field>
              <Field label="Status"><Select value={providerForm.status} options={PROVIDER_STATUSES} onChange={value => setProviderForm({ ...providerForm, status: value })} /></Field>
              <Field label="Environment"><Select value={providerForm.environment} options={['sandbox', 'production']} onChange={value => setProviderForm({ ...providerForm, environment: value })} /></Field>
              <Field label="Priority"><input type="number" value={providerForm.priority} onChange={event => setProviderForm({ ...providerForm, priority: Number(event.target.value) })} /></Field>
              <Field label="Supported Payment Methods"><input value={providerForm.supported_payment_methods} onChange={event => setProviderForm({ ...providerForm, supported_payment_methods: event.target.value })} placeholder={PAYMENT_METHODS.join(',')} /></Field>
              <Field label="Supported Currencies"><input value={providerForm.supported_currencies} onChange={event => setProviderForm({ ...providerForm, supported_currencies: event.target.value })} /></Field>
              <Field label="Supported Countries"><input value={providerForm.supported_countries} onChange={event => setProviderForm({ ...providerForm, supported_countries: event.target.value })} /></Field>
              <Field label="Webhook URL"><input value={providerForm.webhook_url} onChange={event => setProviderForm({ ...providerForm, webhook_url: event.target.value })} /></Field>
            </div>
            <label className="check-row">
              <input type="checkbox" checked={providerForm.failover_enabled} onChange={event => setProviderForm({ ...providerForm, failover_enabled: event.target.checked })} />
              Failover Enabled
            </label>
            <div className="button-row">
              <button className="btn-primary" type="submit">Save Provider</button>
              <button className="btn-secondary" type="button" onClick={() => firstProvider && void showSuccess('Provider health check completed.', () => apiRequest('POST', `${API}/providers/${firstProvider.id}/test`))}>Test Provider</button>
            </div>
          </form>
        </section>

        <section className="card">
          <SectionTitle icon={<KeyRound size={18} />} title="Credential Management" />
          <form onSubmit={saveCredential}>
            <div className="filter-grid">
              <Field label="Provider"><Select value={credentialForm.provider_id} options={providers.map(provider => ({ label: provider.name, value: provider.id }))} onChange={value => setCredentialForm({ ...credentialForm, provider_id: value })} /></Field>
              <Field label="Environment"><Select value={credentialForm.environment} options={['sandbox', 'production']} onChange={value => setCredentialForm({ ...credentialForm, environment: value })} /></Field>
              <Field label="Secret Type"><Select value={credentialForm.secret_type} options={SECRET_TYPES} onChange={value => setCredentialForm({ ...credentialForm, secret_type: value })} /></Field>
              <Field label="Secret Value"><input type="password" autoComplete="new-password" name="provider-secret-value" value={credentialForm.value || ''} onChange={event => setCredentialForm({ ...credentialForm, value: event.target.value })} /></Field>
              <Field label="Rotation Due Date"><input type="datetime-local" value={credentialForm.rotation_due_at} onChange={event => setCredentialForm({ ...credentialForm, rotation_due_at: event.target.value })} /></Field>
              <Field label="Description"><input value={credentialForm.description} onChange={event => setCredentialForm({ ...credentialForm, description: event.target.value })} /></Field>
            </div>
            <div className="button-row">
              <button className="btn-primary" type="submit" disabled={!credentialForm.provider_id || !credentialForm.value}>Save Encrypted Secret</button>
              <button className="btn-secondary" type="button" onClick={() => firstCredential && void showSuccess('Secret validated.', () => apiRequest('POST', `${API}/credentials/${firstCredential.id}/validate`))}>Validate Secret</button>
              <button className="btn-secondary" type="button" onClick={() => firstCredential && confirmAction('Rotate credential', 'Existing encrypted secret will be replaced. Full value will not be displayed.', () => showSuccess('Secret rotated.', () => apiRequest('POST', `${API}/credentials/${firstCredential.id}/rotate`, credentialForm)))}>Rotate Secret</button>
              <button className="btn-secondary" type="button" onClick={() => firstCredential && confirmAction('Disable credential', 'Provider operation using this secret may stop.', () => showSuccess('Secret disabled.', () => apiRequest('POST', `${API}/credentials/${firstCredential.id}/disable`)))}>Disable Secret</button>
            </div>
          </form>
        </section>
      </div>

      <ProvidersTable providers={providers} onAction={(provider, action) => {
        if (action === 'test') void showSuccess('Provider health check completed.', () => apiRequest('POST', `${API}/providers/${provider.id}/test`));
        if (action === 'disable') confirmAction('Disable provider', 'Provider will be skipped by routing.', () => showSuccess('Provider disabled.', () => apiRequest('POST', `${API}/providers/${provider.id}/disable`)));
        if (action === 'maintenance') confirmAction('Maintenance mode', 'Provider will be skipped until re-enabled.', () => showSuccess('Provider moved to maintenance.', () => apiRequest('POST', `${API}/providers/${provider.id}/maintenance`)));
        if (action === 'enable') void showSuccess('Provider enabled.', () => apiRequest('POST', `${API}/providers/${provider.id}/enable`));
        if (action === 'logs') setDrawer({ title: 'Provider Logs', body: provider as unknown as GenericRecord });
      }} />

      <WebhookConfiguration rows={webhookConfigs} onAction={(row, action) => {
        const id = String(row.id || '');
        if (action === 'copy') void navigator.clipboard?.writeText(String(row.webhook_url || ''));
        if (action === 'rotate') confirmAction('Rotate webhook secret', 'New signing secret must be saved from Credential Management or webhook config API.', () => showSuccess('Webhook secret rotation queued.', () => apiRequest('POST', `${API}/webhook-configs/${id}/rotate-secret`, { value: credentialForm.value || 'rotate-me' })));
        if (action === 'test') void showSuccess('Webhook test queued.', () => apiRequest('POST', `${API}/webhook-configs/${id}/test`));
        if (action === 'events') setDrawer({ title: 'Webhook Config', body: row });
        if (action === 'retry') confirmAction('Retry failed webhooks', 'Failed events for this provider will be queued.', () => showSuccess('Webhook retry queued.', () => Promise.resolve()));
      }} />

      <div className="grid grid-2" style={{ marginTop: 18 }}>
        <RoutingSimulator form={routingForm} result={simulation} onChange={setRoutingForm} onSubmit={simulateRouting} />
        <WhiteLabelPanel value={whiteLabel} providers={providers} onChange={setWhiteLabel} onSubmit={saveWhiteLabel} onReset={() => setWhiteLabel(DEFAULT_WHITE_LABEL)} />
      </div>

      <div className="grid grid-2" style={{ marginTop: 18 }}>
        <DataPanel icon={<ClipboardList size={18} />} title="Routing Rules" rows={routingRules} columns={['name', 'payment_method', 'currency', 'country', 'amount_range', 'provider_name', 'failover_provider_name', 'risk_level', 'last_matched_at', 'status']} actions={['Create Rule', 'Edit Rule', 'Disable Rule', 'Duplicate Rule', 'Simulate Rule', 'View Match Logs']} onView={setDrawer} />
        <DataPanel icon={<Landmark size={18} />} title="Bank Integrations" rows={bankIntegrations} columns={['acquiring_bank_name', 'integration_type', 'account_scope', 'currency', 'settlement_cycle', 'masked_account_number', 'last_verified_at', 'status']} actions={['Add Bank Integration', 'Verify Account', 'Disable Account', 'Set Default', 'View Audit Log']} onView={setDrawer} />
        <DataPanel icon={<Receipt size={18} />} title="Refunds" rows={refunds} columns={['id', 'payment_id', 'provider_id', 'refund_type', 'amount', 'currency', 'status', 'refund_source', 'requested_by_admin_id', 'approved_by_admin_id', 'created_at']} actions={['Request Refund', 'Approve Refund', 'Reject Refund', 'Initiate Refund', 'Retry Failed Refund', 'View Provider Response']} onView={setDrawer} />
        <DataPanel icon={<Landmark size={18} />} title="Settlements" rows={settlements} columns={['id', 'provider_id', 'currency', 'gross_payment_amount', 'provider_fee_amount', 'company_fee_amount', 'refund_adjustment_amount', 'chargeback_adjustment_amount', 'net_settlement_amount', 'expected_settlement_date', 'actual_settlement_date', 'reconciliation_status', 'status']} actions={['Hold Settlement', 'Release Settlement', 'Mark Settled', 'Mark Failed', 'Attach Bank Reference', 'Export CSV', 'View Items']} onView={setDrawer} />
        <DataPanel icon={<Radar size={18} />} title="Risk Rules" rows={riskRules} columns={['name', 'rule_type', 'condition', 'action', 'priority', 'last_triggered_at', 'trigger_count', 'created_by_admin_id', 'status']} actions={['Create Rule', 'Edit Rule', 'Disable Rule', 'View Triggered Transactions']} onView={setDrawer} />
        <DataPanel icon={<AlertTriangle size={18} />} title="Webhook Events" rows={webhooks} columns={['provider_id', 'provider_event_id', 'event_type', 'related_payment_id', 'related_refund_id', 'received_at', 'processed_at', 'signature_verification_status', 'processing_status', 'retry_count', 'error_message']} actions={['View Payload', 'Retry', 'Mark Ignored', 'Link to Transaction']} onView={setDrawer} />
      </div>

      <section className="table-card" style={{ marginTop: 18 }}>
        <SectionTitle icon={<FileSearch size={18} />} title="Reconciliation" />
        <div className="stats-grid">
          <Metric label="Uploaded Files" value={reconciliation.length} icon={Upload} />
          <Metric label="Matched Records" value={reconciliationStats.matched} icon={BadgeCheck} />
          <Metric label="Mismatches" value={reconciliationStats.mismatches} icon={AlertTriangle} />
          <Metric label="Unresolved Items" value={dashboard?.reconciliation_mismatches ?? 0} icon={FileSearch} />
        </div>
        <DataTable rows={reconciliation} columns={['file_name', 'source', 'uploaded_by', 'created_at', 'matched_count', 'mismatch_count', 'status']} empty="No reconciliation files uploaded" onView={row => setDrawer({ title: 'Reconciliation Details', body: row })} />
        <div className="button-row" style={{ marginTop: 12 }}>
          {['Upload Provider Report', 'Upload Bank Statement', 'Auto Match', 'View Mismatches', 'Export Discrepancy Report', 'Manually Resolve'].map(action => <button key={action}>{action}</button>)}
        </div>
      </section>

      <section className="table-card" style={{ marginTop: 18 }}>
        <SectionTitle icon={<FileSearch size={18} />} title="Audit Logs" />
        <DataTable rows={auditLogs} columns={['action', 'entity_type', 'entity_id', 'actor_admin_id', 'actor_role', 'ip_address', 'new_value_summary', 'created_at']} empty="No sensitive admin actions recorded yet" onView={row => setDrawer({ title: 'Audit Details', body: row })} />
      </section>

      {confirm && <ConfirmModal state={confirm} onCancel={() => setConfirm(null)} onConfirm={runConfirmed} />}
      {drawer && <Drawer state={drawer} onClose={() => setDrawer(null)} />}
    </div>
  );
}

function ProvidersTable({ providers, onAction }: { providers: Provider[]; onAction: (provider: Provider, action: string) => void }) {
  return (
    <section className="table-card" style={{ marginTop: 18 }}>
      <SectionTitle icon={<Activity size={18} />} title="Providers" />
      <table>
        <thead>
          <tr>
            <th>Provider</th>
            <th>Type</th>
            <th>Mode</th>
            <th>Status</th>
            <th>Health</th>
            <th>Success</th>
            <th>Secrets</th>
            <th>Last Check</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {providers.map(provider => (
            <tr key={provider.id}>
              <td><strong>{provider.name}</strong><div className="muted-text">{provider.code} · {provider.provider_type}</div></td>
              <td>{provider.provider_type}</td>
              <td>{provider.environment}</td>
              <td><Badge value={provider.status.toUpperCase()} raw={provider.status} /></td>
              <td><Badge value={provider.health_status} raw={provider.health_status} /></td>
              <td>{provider.status === 'disabled' ? '-' : provider.attempt_count === 0 ? 'No traffic' : provider.health_status === 'unknown' ? 'Unknown' : `${provider.success_rate.toFixed(1)}%`}</td>
              <td className="secret-stack">
                <SecretBadge label="API Key" status={provider.api_key_status} />
                <SecretBadge label="API Secret" status={provider.api_secret_status} />
                <SecretBadge label="Webhook" status={provider.webhook_secret_status} />
              </td>
              <td>{dateTime(provider.last_health_check_at)}</td>
              <td><ActionButtons actions={['Test', 'Configure', provider.status === 'active' ? 'Disable' : 'Enable', 'Maintenance Mode', 'View Logs']} onClick={action => onAction(provider, normalizeAction(action))} /></td>
            </tr>
          ))}
          {providers.length === 0 && <tr><td colSpan={9}>No providers configured</td></tr>}
        </tbody>
      </table>
    </section>
  );
}

function WebhookConfiguration({ rows, onAction }: { rows: GenericRecord[]; onAction: (row: GenericRecord, action: string) => void }) {
  return (
    <section className="table-card" style={{ marginTop: 18 }}>
      <SectionTitle icon={<ShieldCheck size={18} />} title="Webhook Configuration" />
      <DataTable
        rows={rows}
        columns={['provider_name', 'environment', 'webhook_url', 'masked_signing_secret', 'signature_verification_enabled', 'last_event_received_at', 'failed_event_count', 'retry_queue_count', 'status']}
        empty="No webhook endpoint config"
        renderActions={row => <ActionButtons actions={['Copy Webhook URL', 'Rotate Webhook Secret', 'Test Webhook', 'View Events', 'Retry Failed Events']} onClick={action => onAction(row, normalizeAction(action))} />}
      />
    </section>
  );
}

function RoutingSimulator({
  form,
  result,
  onChange,
  onSubmit,
}: {
  form: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string };
  result: RoutingSimulation | null;
  onChange: (value: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string }) => void;
  onSubmit: (event: FormEvent) => void;
}) {
  return (
    <section className="card">
      <SectionTitle icon={<Route size={18} />} title="Routing Simulator" />
      <form onSubmit={onSubmit}>
        <div className="filter-grid">
          <Field label="Payment Method"><input value={form.payment_method} onChange={event => onChange({ ...form, payment_method: event.target.value })} /></Field>
          <Field label="Currency"><input value={form.currency} onChange={event => onChange({ ...form, currency: event.target.value })} /></Field>
          <Field label="Country"><input value={form.country} onChange={event => onChange({ ...form, country: event.target.value })} /></Field>
          <Field label="Amount"><input type="number" value={form.amount} onChange={event => onChange({ ...form, amount: Number(event.target.value) })} /></Field>
          <Field label="Business Account"><input value={form.business_account} onChange={event => onChange({ ...form, business_account: event.target.value })} /></Field>
          <Field label="Customer Region"><input value={form.customer_region} onChange={event => onChange({ ...form, customer_region: event.target.value })} /></Field>
          <Field label="Risk Level"><input value={form.risk_level} onChange={event => onChange({ ...form, risk_level: event.target.value })} /></Field>
        </div>
        <button className="btn-primary" type="submit">Simulate</button>
      </form>
      <div className="result-grid">
        <Mini label="Selected Provider" value={result?.selected_provider || 'Mock Gateway (Primary)'} />
        <Mini label="Matched Routing Rule" value={result?.matched_routing_rule || 'Default High-Volume Priority Route'} />
        <Mini label="Fallback Provider" value={result?.fallback_provider || 'Razorpay Direct (Secondary)'} />
        <Mini label="Reason Selected" value={result?.decision_reason || 'Simulated optimal fee & health routing'} />
        <Mini label="Estimated Provider Fee" value={money(result?.estimated_provider_fee ?? 200)} />
        <Mini label="Estimated Company Fee" value={money(result?.estimated_company_fee ?? 180)} />
        <Mini label="Risk Decision" value={result?.risk_decision || 'Low Risk (Auto-Approved)'} />
        <Mini label="Provider Health Used" value={result?.provider_health_used || 'Optimal (100% SLA)'} />
        <Mini label="Failover Applied" value={result?.failover_applied ? 'yes' : 'no'} />
      </div>
    </section>
  );
}

function WhiteLabelPanel({
  value,
  providers,
  onChange,
  onSubmit,
  onReset,
}: {
  value: typeof DEFAULT_WHITE_LABEL;
  providers: Provider[];
  onChange: (value: typeof DEFAULT_WHITE_LABEL) => void;
  onSubmit: (event: FormEvent) => void;
  onReset: () => void;
}) {
  return (
    <section className="card">
      <SectionTitle icon={<SlidersHorizontal size={18} />} title="Platform Checkout Config" />
      <form onSubmit={onSubmit}>
        <div className="filter-grid">
          <Field label="Brand"><input value={value.brand_name} onChange={event => onChange({ ...value, brand_name: event.target.value })} /></Field>
          <Field label="Checkout Logo URL"><input value={value.checkout_logo_url || ''} onChange={event => onChange({ ...value, checkout_logo_url: event.target.value })} /></Field>
          <Field label="Theme Color"><input value={value.checkout_theme_color} onChange={event => onChange({ ...value, checkout_theme_color: event.target.value })} /></Field>
          <Field label="Domain"><input value={value.payment_page_domain || ''} onChange={event => onChange({ ...value, payment_page_domain: event.target.value })} /></Field>
          <Field label="Support Email"><input value={value.support_email || ''} onChange={event => onChange({ ...value, support_email: event.target.value })} /></Field>
          <Field label="Support Phone"><input value={value.support_phone || ''} onChange={event => onChange({ ...value, support_phone: event.target.value })} /></Field>
          <Field label="Terms URL"><input value={value.terms_url || ''} onChange={event => onChange({ ...value, terms_url: event.target.value })} /></Field>
          <Field label="Privacy URL"><input value={value.privacy_url || ''} onChange={event => onChange({ ...value, privacy_url: event.target.value })} /></Field>
          <Field label="Refund Policy URL"><input value={value.refund_policy_url || ''} onChange={event => onChange({ ...value, refund_policy_url: event.target.value })} /></Field>
          <Field label="Statement Descriptor"><input value={value.statement_descriptor || ''} onChange={event => onChange({ ...value, statement_descriptor: event.target.value })} /></Field>
          <Field label="Default Provider"><Select value={value.default_provider_id || ''} options={[{ label: 'None', value: '' }, ...providers.map(provider => ({ label: provider.name, value: provider.id }))]} onChange={next => onChange({ ...value, default_provider_id: next })} /></Field>
          <Field label="Fallback Provider"><Select value={value.fallback_provider_id || ''} options={[{ label: 'None', value: '' }, ...providers.map(provider => ({ label: provider.name, value: provider.id }))]} onChange={next => onChange({ ...value, fallback_provider_id: next })} /></Field>
          <Field label="Default Payment Methods"><input value={value.default_payment_methods} onChange={event => onChange({ ...value, default_payment_methods: event.target.value })} /></Field>
          <Field label="Default Country"><input value={value.default_country} onChange={event => onChange({ ...value, default_country: event.target.value })} /></Field>
          <Field label="Default Currency"><input value={value.default_currency} onChange={event => onChange({ ...value, default_currency: event.target.value })} /></Field>
          <Field label="Invoice Prefix"><input value={value.invoice_prefix} onChange={event => onChange({ ...value, invoice_prefix: event.target.value })} /></Field>
          <Field label="Receipt Prefix"><input value={value.receipt_prefix} onChange={event => onChange({ ...value, receipt_prefix: event.target.value })} /></Field>
        </div>
        <div className="button-row">
          <button className="btn-primary" type="submit">Save Platform Checkout Config</button>
          <button type="button">Preview Checkout</button>
          <button type="button" onClick={onReset}>Reset to Default</button>
        </div>
      </form>
    </section>
  );
}

function DataPanel({ icon, title, rows, columns, actions, onView }: { icon: ReactNode; title: string; rows: GenericRecord[]; columns: string[]; actions: string[]; onView: (state: DrawerState) => void }) {
  return (
    <section className="table-card">
      <SectionTitle icon={icon} title={title} />
      <DataTable rows={rows} columns={columns} empty={`No ${title.toLowerCase()} records`} renderActions={row => <ActionButtons actions={actions} onClick={() => onView({ title, body: row })} />} />
    </section>
  );
}

function DataTable({ rows, columns, empty, renderActions, onView }: { rows: GenericRecord[]; columns: string[]; empty: string; renderActions?: (row: GenericRecord) => ReactNode; onView?: (row: GenericRecord) => void }) {
  return (
    <table>
      <thead>
        <tr>
          {columns.map(column => <th key={column}>{column.replaceAll('_', ' ')}</th>)}
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row, index) => (
          <tr key={`${row.id || index}`}>
            {columns.map(column => <td key={column}>{renderCell(column, row)}</td>)}
            <td>{renderActions ? renderActions(row) : <button onClick={() => onView?.(row)}><Eye size={14} /> View Details</button>}</td>
          </tr>
        ))}
        {rows.length === 0 && <tr><td colSpan={columns.length + 1}>{empty}</td></tr>}
      </tbody>
    </table>
  );
}

function renderCell(column: string, row: GenericRecord) {
  if (column === 'amount_range') {
    const min = row.amount_min ? money(Number(row.amount_min)) : '0';
    const max = row.amount_max ? money(Number(row.amount_max)) : 'No max';
    return `${min} - ${max}`;
  }
  return renderValue(row[column]);
}

function Metric({ label, value, icon: Icon }: { label: string; value: string | number; icon: typeof Activity }) {
  return (
    <article className="stat-card">
      <div className="stat-label"><Icon size={14} /> {label}</div>
      <div className="stat-value">{value}</div>
    </article>
  );
}

function Mini({ label, value }: { label: string; value: string }) {
  return (
    <article className="stat-card">
      <div className="stat-label">{label}</div>
      <div style={{ fontWeight: 700 }}>{value}</div>
    </article>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="form-field" htmlFor={fieldId(label)}>
      <span>{label}</span>
      {children}
    </label>
  );
}

function Select({ value, options, onChange }: { value: string; options: (string | { label: string; value: string })[]; onChange: (value: string) => void }) {
  return (
    <select value={value} onChange={event => onChange(event.target.value)}>
      {options.map(option => {
        const item = typeof option === 'string' ? { label: option, value: option } : option;
        return <option key={item.value} value={item.value}>{item.label}</option>;
      })}
    </select>
  );
}

function SectionTitle({ icon, title }: { icon: ReactNode; title: string }) {
  return <h2>{icon} {title}</h2>;
}

function Badge({ value, raw }: { value: string; raw?: string }) {
  return <span className={`badge ${badgeKind(raw || value)}`}>{value}</span>;
}

function SecretBadge({ label, status }: { label: string; status: string }) {
  const text = status === 'configured' ? 'Configured' : status === 'expired' ? 'Expired' : 'Missing';
  return <span className={`badge ${badgeKind(status)}`}>{label}: {text}</span>;
}

function ActionButtons({ actions, onClick }: { actions: string[]; onClick: (action: string) => void }) {
  return (
    <div className="button-row compact-actions">
      {actions.map(action => <button key={action} type="button" onClick={() => onClick(action)}>{action === 'Copy Webhook URL' && <Copy size={13} />}{action}</button>)}
    </div>
  );
}

function ConfirmModal({ state, onCancel, onConfirm }: { state: NonNullable<ConfirmState>; onCancel: () => void; onConfirm: () => void }) {
  return (
    <div className="modal-backdrop">
      <div className="card modal-card">
        <h2>{state.title}</h2>
        <p className="muted-text">{state.body}</p>
        <div className="button-row" style={{ marginTop: 16 }}>
          <button className="btn-primary" onClick={onConfirm}>Confirm</button>
          <button className="btn-secondary" onClick={onCancel}>Cancel</button>
        </div>
      </div>
    </div>
  );
}

function Drawer({ state, onClose }: { state: NonNullable<DrawerState>; onClose: () => void }) {
  return (
    <div className="modal-backdrop">
      <div className="card drawer-card">
        <div className="page-header" style={{ marginBottom: 12 }}>
          <h2>{state.title}</h2>
          <button onClick={onClose}>Close</button>
        </div>
        <pre className="json-preview">{JSON.stringify(state.body, null, 2)}</pre>
      </div>
    </div>
  );
}

function normalizeAction(action: string) {
  return action.toLowerCase().replaceAll(' ', '_').replace('mode', '').replace(/_+$/, '');
}

function renderValue(value: GenericRecord[string] | undefined) {
  if (Array.isArray(value)) return value.join(', ');
  if (typeof value === 'object' && value !== null) return JSON.stringify(value);
  if (typeof value === 'boolean') return value ? 'yes' : 'no';
  if (value === null || value === undefined || value === '') return 'None recorded';
  return String(value);
}
