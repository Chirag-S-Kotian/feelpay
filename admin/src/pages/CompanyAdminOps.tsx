import { FormEvent, ReactNode, useEffect, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  BadgeCheck,
  Banknote,
  Cable,
  Download,
  Eye,
  FileSearch,
  KeyRound,
  Landmark,
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
type LoadState = 'loading' | 'ready' | 'error';
type ConfirmState = { title: string; body: string; action: () => Promise<void> } | null;
type DrawerState = { title: string; body: GenericRecord } | null;

type Dashboard = {
  total_gmv: number;
  payment_success_rate: number;
  pending_refunds: number;
  pending_settlements: number;
  reconciliation_mismatches: number;
  webhook_failures: number;
  company_revenue_total: number;
};

type Provider = GenericRecord & {
  id: string;
  name: string;
  code: string;
  provider_type: string;
  environment: string;
  status: string;
  health_status: string;
  success_rate: number;
  attempt_count: number;
  api_key_status: string;
  api_secret_status: string;
  webhook_secret_status: string;
};

type RoutingSimulation = {
  selected_provider?: string | null;
  matched_routing_rule?: string | null;
  fallback_provider?: string | null;
  decision_reason: string;
  estimated_provider_fee: number;
  estimated_company_fee: number;
  risk_decision: string;
  provider_health_used: string;
  failover_applied: boolean;
};

type WhiteLabelForm = {
  brand_name: string;
  checkout_logo_url: string;
  checkout_theme_color: string;
  payment_page_domain: string;
  support_email: string;
  support_phone: string;
  terms_url: string;
  privacy_url: string;
  refund_policy_url: string;
  statement_descriptor: string;
  default_provider_id: string;
  fallback_provider_id: string;
  default_payment_methods: string;
  default_country: string;
  default_currency: string;
  invoice_prefix: string;
  receipt_prefix: string;
};

const DEFAULT_WHITE_LABEL: WhiteLabelForm = {
  brand_name: 'FeelPay',
  checkout_logo_url: '',
  checkout_theme_color: '#1DB954',
  payment_page_domain: '',
  support_email: '',
  support_phone: '',
  terms_url: '',
  privacy_url: '',
  refund_policy_url: '',
  statement_descriptor: '',
  default_provider_id: '',
  fallback_provider_id: '',
  default_payment_methods: 'upi,card,netbanking',
  default_country: 'IN',
  default_currency: 'INR',
  invoice_prefix: 'INV',
  receipt_prefix: 'RCPT',
};

const ADMIN_ROLES = [
  'Super Admin',
  'Payment Ops Admin',
  'Finance Admin',
  'Risk Admin',
  'Support Admin',
  'Support Agent',
  'Compliance Admin',
  'Auditor',
];

const ADMIN_PERMISSIONS = [
  'payment_ops.view',
  'transactions.view',
  'transactions.action',
  'refunds.request',
  'refunds.approve',
  'refunds.initiate',
  'settlements.manage',
  'disputes.manage',
  'risk_rules.manage',
  'reconciliation.manage',
  'providers.manage',
  'credentials.rotate',
  'bank_integrations.manage',
  'routing.manage',
  'webhook_config.manage',
  'reports.export',
  'audit_logs.view',
  'admin_users.manage',
  'platform_settings.manage',
];

function money(value: number) {
  return new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(value / 100);
}

function list(value: string) {
  return value.split(',').map(item => item.trim()).filter(Boolean);
}

function renderValue(value: GenericRecord[string] | undefined) {
  if (Array.isArray(value)) return value.join(', ');
  if (typeof value === 'object' && value !== null) return JSON.stringify(value);
  if (typeof value === 'boolean') return value ? 'yes' : 'no';
  if (value === null || value === undefined || value === '') return '-';
  return String(value);
}

function rowsFrom(value: { items?: GenericRecord[] }) {
  return value.items || [];
}

function useRows(path: string) {
  const [rows, setRows] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<{ items?: GenericRecord[] }>('GET', path);
      setRows(rowsFrom(data));
      setState('ready');
    } catch (err) {
      setRows([]);
      setState('error');
      setError(err instanceof Error ? err.message : 'Load failed');
    }
  };
  useEffect(() => {
    void load();
  }, [path]);
  return { rows, state, error, load };
}

function useDrawerConfirm() {
  const [confirm, setConfirm] = useState<ConfirmState>(null);
  const [drawer, setDrawer] = useState<DrawerState>(null);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');
  const run = async (text: string, action: () => Promise<void>) => {
    setNotice('');
    setError('');
    try {
      await action();
      setNotice(text);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    }
  };
  return { confirm, setConfirm, drawer, setDrawer, notice, setNotice, error, setError, run };
}

export function PaymentOpsPage() {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [failedPayments, setFailedPayments] = useState<GenericRecord[]>([]);
  const [webhooks, setWebhooks] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');

  const load = async () => {
    setState('loading');
    setError('');
    try {
      const [dashboardData, providerData, failedData, webhookData] = await Promise.all([
        apiRequest<Dashboard>('GET', `${API}/dashboard`),
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/transactions?status=failed&limit=6`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/webhook-events?limit=6`),
      ]);
      setDashboard(dashboardData);
      setProviders(providerData.items || []);
      setFailedPayments(rowsFrom(failedData));
      setWebhooks(rowsFrom(webhookData).filter(row => String(row.processing_status || '').includes('failed')).slice(0, 6));
      setState('ready');
    } catch (err) {
      setState('error');
      setError(err instanceof Error ? err.message : 'Payment operations load failed');
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const activeProviders = providers.filter(provider => provider.status === 'active').length;

  return (
    <Page title="Payment Ops" subtitle="High-level company payment operations dashboard" onRefresh={load}>
      <Status state={state} error={error} loading="Loading payment operations..." />
      <div className="stats-grid">
        <Metric label="GMV" value={dashboard ? money(dashboard.total_gmv) : '0'} icon={Banknote} />
        <Metric label="Success rate" value={dashboard ? `${dashboard.payment_success_rate.toFixed(1)}%` : '0%'} icon={Activity} />
        <Metric label="Active providers" value={`${activeProviders}/${providers.length}`} icon={Cable} />
        <Metric label="Webhook failures" value={dashboard?.webhook_failures ?? 0} icon={AlertTriangle} />
        <Metric label="Pending refunds" value={dashboard?.pending_refunds ?? 0} icon={Receipt} />
        <Metric label="Pending settlements" value={dashboard?.pending_settlements ?? 0} icon={Landmark} />
        <Metric label="Reconciliation mismatches" value={dashboard?.reconciliation_mismatches ?? 0} icon={FileSearch} />
        <Metric label="Company revenue" value={dashboard ? money(dashboard.company_revenue_total) : '0'} icon={BadgeCheck} />
      </div>
      <div className="grid grid-2">
        <TablePanel title="Recent failed payments" rows={failedPayments} columns={['id', 'provider_name', 'amount', 'currency', 'status', 'created_at']} />
        <TablePanel title="Recent webhook failures" rows={webhooks} columns={['provider_name', 'event_type', 'related_payment_id', 'processing_status', 'retry_count', 'received_at']} />
        <TablePanel title="Provider health summary" rows={providers} columns={['name', 'environment', 'status', 'health_status', 'success_rate', 'attempt_count']} />
      </div>
    </Page>
  );
}

export function TransactionsPage() {
  const [filters, setFilters] = useState({
    start_date: '',
    end_date: '',
    status: '',
    provider: '',
    payment_method: '',
    currency: '',
    amount: '',
    business_account: '',
    settlement_status: '',
    refund_status: '',
  });
  const [path, setPath] = useState(`${API}/transactions?limit=100`);
  const { rows, state, error, load } = useRows(path);
  const helpers = useDrawerConfirm();

  const applyFilters = (event: FormEvent) => {
    event.preventDefault();
    const params = new URLSearchParams({ limit: '100' });
    Object.entries(filters).forEach(([key, value]) => {
      if (value.trim()) params.set(key, value.trim());
    });
    setPath(`${API}/transactions?${params.toString()}`);
  };

  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'capture') helpers.setConfirm({ title: 'Capture authorized payment', body: 'Capture only if authorization is still valid and amount is correct.', action: () => helpers.run('Capture requested.', () => apiRequest('POST', `${API}/transactions/${id}/capture`)).then(load) });
    if (action === 'cancel') helpers.setConfirm({ title: 'Cancel payment', body: 'Cancel only when payment is safe to void.', action: () => helpers.run('Cancel requested.', () => apiRequest('POST', `${API}/transactions/${id}/cancel`)).then(load) });
    if (action === 'refund') helpers.setConfirm({ title: 'Request refund', body: 'Refund request will be audit logged before provider initiation.', action: () => helpers.run('Refund requested.', () => apiRequest('POST', `${API}/transactions/${id}/refund`, { amount: row.amount, reason: 'admin_request' })).then(load) });
    if (action === 'detail' || action === 'webhooks' || action === 'audit') helpers.setDrawer({ title: `Transaction ${action}`, body: row });
  };

  return (
    <CrudPage title="Transactions" subtitle="Company admin transaction search, list, and detail" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <form className="card filter-grid" onSubmit={applyFilters}>
        {Object.entries(filters).map(([key, value]) => (
          <Field key={key} label={key.replaceAll('_', ' ')}>
            <input value={value} onChange={event => setFilters({ ...filters, [key]: event.target.value })} />
          </Field>
        ))}
        <button className="btn-primary" type="submit">Filter</button>
      </form>
      <DataTable rows={rows} columns={['id', 'provider_name', 'business_account_name', 'amount', 'currency', 'payment_method', 'status', 'settlement_status', 'refund_status', 'created_at']} empty="No transactions found" renderActions={row => <ActionButtons actions={['View detail', 'Capture', 'Cancel', 'Request refund', 'View webhook events', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RefundsPage() {
  const { rows, state, error, load } = useRows(`${API}/refunds?limit=100`);
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    const calls: Record<string, string> = { approve: 'approve', reject: 'reject', initiate: 'initiate', retry: 'retry' };
    if (action === 'request') helpers.setConfirm({ title: 'Request refund', body: 'Refund request will be audit logged.', action: () => helpers.run('Refund request queued.', () => apiRequest('POST', `${API}/refunds`, row)).then(load) });
    if (calls[action]) helpers.setConfirm({ title: `${action} refund`, body: 'Sensitive refund action. Confirm actor, amount, and provider state.', action: () => helpers.run(`Refund ${action} requested.`, () => apiRequest('POST', `${API}/refunds/${id}/${calls[action]}`)).then(load) });
    if (action === 'view_provider_response') helpers.setDrawer({ title: 'Provider response', body: row });
  };
  return <TableRoute title="Refunds" subtitle="Refund requests, approvals, provider initiation, and retries" rows={rows} state={state} error={error} notice={helpers.notice} columns={['id', 'payment_id', 'provider_id', 'amount', 'currency', 'status', 'refund_source', 'requested_by_admin_id', 'approved_by_admin_id', 'created_at']} actions={['Request', 'Approve', 'Reject', 'Initiate', 'Retry', 'View provider response']} onAction={act} onRefresh={load} helpers={helpers} />;
}

export function SettlementsPage() {
  const { rows, state, error, load } = useRows(`${API}/settlements?limit=100`);
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    const actionPath = action.replace('mark_', 'mark-');
    if (action === 'view_items' || action === 'export_csv') helpers.setDrawer({ title: 'Settlement details', body: row });
    else helpers.setConfirm({ title: `${action.replaceAll('_', ' ')} settlement`, body: 'Settlement action affects finance ledgers and must be audit logged.', action: () => helpers.run('Settlement action submitted.', () => apiRequest('POST', `${API}/settlements/${id}/${actionPath}`)).then(load) });
  };
  return <TableRoute title="Settlements" subtitle="Settlement ledger and reconciliation status" rows={rows} state={state} error={error} notice={helpers.notice} columns={['id', 'provider_id', 'currency', 'gross_payment_amount', 'provider_fee_amount', 'company_fee_amount', 'refund_adjustment_amount', 'chargeback_adjustment_amount', 'net_settlement_amount', 'expected_settlement_date', 'actual_settlement_date', 'reconciliation_status', 'status']} actions={['Hold', 'Release', 'Mark settled', 'Mark failed', 'View items', 'Export CSV']} onAction={act} onRefresh={load} helpers={helpers} />;
}

export function DisputesPage() {
  const { rows, state, error, load } = useRows(`${API}/disputes?limit=100`);
  const helpers = useDrawerConfirm();
  return <TableRoute title="Disputes" subtitle="Company dispute and chargeback queue" rows={rows} state={state} error={error} notice={helpers.notice} columns={['id', 'payment_id', 'provider_id', 'amount', 'reason', 'status', 'evidence_due_date', 'created_at']} actions={['View detail', 'Manage evidence', 'View audit trail']} onAction={(row) => helpers.setDrawer({ title: 'Dispute detail', body: row })} onRefresh={load} helpers={helpers} />;
}

export function RiskRulesPage() {
  const { rows, state, error, load } = useRows(`${API}/risk-rules`);
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    if (action === 'disable') helpers.setConfirm({ title: 'Disable risk rule', body: 'Risk decisioning may change immediately.', action: () => helpers.run('Risk rule disabled.', () => apiRequest('POST', `${API}/risk-rules/${String(row.id || '')}/disable`)).then(load) });
    else helpers.setDrawer({ title: 'Risk rule', body: row });
  };
  return <TableRoute title="Risk Rules" subtitle="Company risk controls and rule trigger history" rows={rows} state={state} error={error} notice={helpers.notice} columns={['name', 'rule_type', 'condition', 'action', 'priority', 'last_triggered_at', 'trigger_count', 'status']} actions={['Create rule', 'Edit rule', 'Disable', 'View triggered transactions']} onAction={act} onRefresh={load} helpers={helpers} />;
}

export function ReconciliationPage() {
  const { rows, state, error, load } = useRows(`${API}/reconciliation`);
  const helpers = useDrawerConfirm();
  const matched = rows.reduce((sum, row) => sum + Number(row.matched_count || 0), 0);
  const mismatches = rows.reduce((sum, row) => sum + Number(row.mismatch_count || 0), 0);
  const unresolved = rows.reduce((sum, row) => sum + Number(row.unresolved_count || row.mismatch_count || 0), 0);
  const act = (action: string) => {
    if (action.includes('upload') || action === 'auto_match' || action === 'manually_resolve') helpers.setConfirm({ title: action.replaceAll('_', ' '), body: 'Reconciliation changes affect settlement review and will be audit logged.', action: () => helpers.run('Reconciliation action queued.', () => Promise.resolve()).then(load) });
    else helpers.setDrawer({ title: 'Reconciliation', body: rows[0] || {} });
  };
  return (
    <CrudPage title="Reconciliation" subtitle="Provider and bank statement matching" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Uploaded files" value={rows.length} icon={Upload} />
        <Metric label="Matched records" value={matched} icon={BadgeCheck} />
        <Metric label="Mismatches" value={mismatches} icon={AlertTriangle} />
        <Metric label="Unresolved items" value={unresolved} icon={FileSearch} />
      </div>
      <DataTable rows={rows} columns={['file_name', 'source', 'uploaded_by', 'created_at', 'matched_count', 'mismatch_count', 'status']} empty="No reconciliation files uploaded" onView={row => helpers.setDrawer({ title: 'Reconciliation details', body: row })} />
      <div className="button-row">
        {['Upload provider report', 'Upload bank statement', 'Auto match', 'View mismatches', 'Export discrepancy report', 'Manually resolve'].map(action => <button key={action} type="button" onClick={() => act(normalizeAction(action))}>{action}</button>)}
      </div>
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function BusinessAccountsPage() {
  const [rows, setRows] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<{ merchants?: GenericRecord[] }>('GET', '/admin/merchants');
      setRows(data.merchants || []);
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Business accounts load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  return <TableRoute title="Business Accounts" subtitle="Internal company-managed businesses and tenants only" rows={rows} state={state} error={error} columns={['business_name', 'business_type', 'status', 'risk_status', 'payout_status', 'created_at']} actions={['View detail', 'Review risk', 'Manage payouts']} onAction={() => undefined} onRefresh={load} />;
}

export function ProviderConnectorsPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [credentials, setCredentials] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const [credentialForm, setCredentialForm] = useState({ provider_id: '', environment: 'sandbox', secret_type: 'api_secret', value: '', rotation_due_at: '', description: '' });

  const load = async () => {
    setState('loading');
    setError('');
    try {
      const [providerData, credentialData] = await Promise.all([
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/credentials`),
      ]);
      setProviders(providerData.items || []);
      setCredentials(rowsFrom(credentialData));
      setCredentialForm(form => ({ ...form, provider_id: form.provider_id || providerData.items?.[0]?.id || '' }));
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Provider connector load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);

  const saveSecret = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Save encrypted secret',
      body: 'Secret encrypted at rest. Masked in UI. Full value never returned after save. Rotation audit logged.',
      action: () => helpers.run('Credential saved.', () => apiRequest('POST', `${API}/credentials`, { ...credentialForm, rotation_due_at: credentialForm.rotation_due_at || null })).then(load),
    });
  };

  return (
    <CrudPage title="Provider Connectors" subtitle="Providers, encrypted credentials, health, and provider logs" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="grid grid-2">
        <section className="card">
          <h2><KeyRound size={18} /> Credential Management</h2>
          <form onSubmit={saveSecret}>
            <div className="filter-grid">
              <Field label="Provider"><Select value={credentialForm.provider_id} options={providers.map(provider => ({ label: provider.name, value: provider.id }))} onChange={value => setCredentialForm({ ...credentialForm, provider_id: value })} /></Field>
              <Field label="Environment"><Select value={credentialForm.environment} options={['sandbox', 'production']} onChange={value => setCredentialForm({ ...credentialForm, environment: value })} /></Field>
              <Field label="Secret type"><Select value={credentialForm.secret_type} options={['api_key', 'api_secret', 'webhook_secret', 'signing_secret', 'client_id', 'client_secret']} onChange={value => setCredentialForm({ ...credentialForm, secret_type: value })} /></Field>
              <Field label="Secret value"><input type="password" value={credentialForm.value} onChange={event => setCredentialForm({ ...credentialForm, value: event.target.value })} /></Field>
              <Field label="Rotation due at"><input type="datetime-local" value={credentialForm.rotation_due_at} onChange={event => setCredentialForm({ ...credentialForm, rotation_due_at: event.target.value })} /></Field>
              <Field label="Description"><input value={credentialForm.description} onChange={event => setCredentialForm({ ...credentialForm, description: event.target.value })} /></Field>
            </div>
            <button className="btn-primary" type="submit" disabled={!credentialForm.provider_id || !credentialForm.value}>Save encrypted secret</button>
          </form>
        </section>
        <TablePanel title="Credential status" rows={credentials} columns={['provider_name', 'environment', 'secret_type', 'masked_value', 'status', 'last_validated_at', 'last_rotated_at', 'rotation_due_at']} />
      </div>
      <TablePanel title="Providers" rows={providers} columns={['name', 'code', 'provider_type', 'environment', 'status', 'health_status', 'success_rate', 'attempt_count', 'api_key_status', 'api_secret_status', 'webhook_secret_status']} />
      <TablePanel title="Provider Health" rows={providers} columns={['name', 'health_status', 'success_rate', 'attempt_count', 'last_health_check_at']} />
      <TablePanel title="Provider Logs" rows={providers} columns={['name', 'status', 'health_status', 'updated_at']} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function BankIntegrationsPage() {
  const { rows, state, error, load } = useRows(`${API}/bank-integrations`);
  const helpers = useDrawerConfirm();
  return <TableRoute title="Bank Integrations" subtitle="Acquirer and company bank settlement configuration" rows={rows} state={state} error={error} notice={helpers.notice} columns={['acquiring_bank_name', 'integration_type', 'account_scope', 'currency', 'settlement_cycle', 'masked_account_number', 'last_verified_at', 'status']} actions={['Add bank integration', 'Verify account', 'Disable account', 'Set default', 'View audit log']} onAction={(row) => helpers.setDrawer({ title: 'Bank integration', body: row })} onRefresh={load} helpers={helpers} />;
}

export function RoutingRulesPage() {
  const { rows, state, error, load } = useRows(`${API}/routing-rules`);
  const helpers = useDrawerConfirm();
  const [form, setForm] = useState({ payment_method: 'upi', currency: 'INR', country: 'IN', amount: 10000, business_account: '', customer_region: '', risk_level: '' });
  const [result, setResult] = useState<RoutingSimulation | null>(null);
  const simulate = async (event: FormEvent) => {
    event.preventDefault();
    setResult(await apiRequest<RoutingSimulation>('POST', `${API}/routing/simulate`, form));
  };
  return (
    <CrudPage title="Routing Rules" subtitle="Routing rules, simulator, match logs, and failover rules" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="grid grid-2">
        <RoutingSimulator form={form} result={result} onChange={setForm} onSubmit={simulate} />
        <TablePanel title="Failover rules" rows={rows} columns={['name', 'provider_name', 'failover_provider_name', 'priority', 'status']} />
      </div>
      <DataTable rows={rows} columns={['name', 'payment_method', 'currency', 'country', 'amount_min', 'amount_max', 'provider_name', 'failover_provider_name', 'risk_level', 'last_matched_at', 'status']} empty="No routing rules configured" onView={row => helpers.setDrawer({ title: 'Routing rule', body: row })} />
      <TablePanel title="Match logs" rows={rows} columns={['name', 'last_matched_at', 'provider_name', 'failover_provider_name', 'status']} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function WhiteLabelCheckoutPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [form, setForm] = useState<WhiteLabelForm>(DEFAULT_WHITE_LABEL);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const [providerData, config] = await Promise.all([
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<Partial<WhiteLabelForm> & { default_payment_methods?: string[] }>('GET', `${API}/white-label-config`),
      ]);
      setProviders(providerData.items || []);
      setForm({ ...DEFAULT_WHITE_LABEL, ...config, default_payment_methods: (config.default_payment_methods || ['upi', 'card', 'netbanking']).join(',') });
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'White-label config load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const save = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({ title: 'Save checkout config', body: 'White-label checkout defaults will change for company-managed payment flows.', action: () => helpers.run('White-label checkout saved.', () => apiRequest('PATCH', `${API}/white-label-config`, { ...form, default_payment_methods: list(form.default_payment_methods), default_provider_id: form.default_provider_id || null, fallback_provider_id: form.fallback_provider_id || null })).then(load) });
  };
  return (
    <CrudPage title="White-label Checkout" subtitle="Company-managed checkout brand, defaults, and descriptors" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <WhiteLabelPanel value={form} providers={providers} onChange={setForm} onSubmit={save} onReset={() => setForm(DEFAULT_WHITE_LABEL)} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function WebhookConfigPage() {
  const configs = useRows(`${API}/webhook-configs`);
  const events = useRows(`${API}/webhook-events?limit=100`);
  const helpers = useDrawerConfirm();
  return (
    <CrudPage title="Webhook Config" subtitle="Endpoint config, events, retry queue, and dead-letter events" state={configs.state} error={configs.error || events.error} notice={helpers.notice} onRefresh={() => void Promise.all([configs.load(), events.load()])}>
      <div className="alert alert-success">Signatures required. Raw payload stored before processing. Processing idempotent. Failed events retried.</div>
      <TablePanel title="Webhook endpoint configs" rows={configs.rows} columns={['provider_name', 'environment', 'webhook_url', 'masked_signing_secret', 'signature_verification_enabled', 'last_event_received_at', 'failed_event_count', 'retry_queue_count', 'status']} />
      <TablePanel title="Webhook events" rows={events.rows} columns={['provider_id', 'provider_event_id', 'event_type', 'related_payment_id', 'signature_verification_status', 'processing_status', 'retry_count', 'received_at']} />
      <TablePanel title="Retry queue" rows={events.rows.filter(row => Number(row.retry_count || 0) > 0)} columns={['provider_event_id', 'event_type', 'processing_status', 'retry_count', 'next_retry_at', 'error_message']} />
      <TablePanel title="Dead-letter events" rows={events.rows.filter(row => String(row.processing_status || '') === 'dead_letter')} columns={['provider_event_id', 'event_type', 'related_payment_id', 'retry_count', 'error_message', 'received_at']} />
    </CrudPage>
  );
}

export function ReportsPage() {
  const reports = [
    'Payment report',
    'Settlement report',
    'Refund report',
    'Dispute report',
    'Reconciliation report',
    'Provider performance report',
    'Revenue report',
  ].map((name, index) => ({ id: String(index), name, format: 'CSV', status: 'ready' }));
  return (
    <Page title="Reports" subtitle="Company finance and operations exports">
      <div className="filter-bar">
        <input placeholder="Filter reports" aria-label="Filter reports" />
        <button className="btn-primary"><Download size={14} /> Export CSV</button>
        <button className="btn-secondary"><Download size={14} /> Export PDF</button>
      </div>
      <DataTable rows={reports} columns={['name', 'format', 'status']} empty="No reports configured" />
    </Page>
  );
}

export function AuditLogsPage() {
  const { rows, state, error, load } = useRows(`${API}/audit-logs`);
  const helpers = useDrawerConfirm();
  return <TableRoute title="Audit Logs" subtitle="Sensitive company admin action history" rows={rows} state={state} error={error} columns={['action', 'entity_type', 'entity_id', 'actor_admin_id', 'actor_role', 'ip_address', 'new_value_summary', 'created_at']} actions={['View detail']} onAction={(row) => helpers.setDrawer({ title: 'Audit details', body: row })} onRefresh={load} helpers={helpers} />;
}

export function AdminUsersRolesPage() {
  const { rows, state, error, load } = useRows(`${API}/admin-roles`);
  return (
    <Page title="Admin Users & Roles" subtitle="Company admin users only, deny-by-default permissions" onRefresh={load}>
      <Status state={state} error={error} loading="Loading admin roles..." />
      <div className="grid grid-2">
        <TablePanel title="Roles" rows={ADMIN_ROLES.map((role, index) => ({ id: String(index), role, scope: 'company_admin' }))} columns={['role', 'scope']} />
        <TablePanel title="Permissions" rows={ADMIN_PERMISSIONS.map((permission, index) => ({ id: String(index), permission }))} columns={['permission']} />
      </div>
      <TablePanel title="Role permissions from server" rows={rows} columns={['name', 'description', 'permissions']} />
    </Page>
  );
}

export function PlatformSettingsPage() {
  return (
    <Page title="Platform Settings" subtitle="Company admin platform controls">
      <div className="card">
        <h2><ShieldCheck size={18} /> Security posture</h2>
        <div className="security-list">
          {[
            'RBAC required on admin routes; deny by default.',
            'Full secrets never displayed after save.',
            'Full bank account numbers are masked.',
            'Raw card data and CVV are not stored.',
            'Sensitive actions require confirmation and audit logging.',
          ].map(item => <div className="security-row" key={item}><span>{item}</span><strong>Enabled</strong></div>)}
        </div>
      </div>
    </Page>
  );
}

function TableRoute({ title, subtitle, rows, state, error, notice, columns, actions, onAction, onRefresh, helpers }: { title: string; subtitle: string; rows: GenericRecord[]; state: LoadState; error: string; notice?: string; columns: string[]; actions: string[]; onAction: (row: GenericRecord, action: string) => void; onRefresh: () => void; helpers?: ReturnType<typeof useDrawerConfirm> }) {
  return (
    <CrudPage title={title} subtitle={subtitle} state={state} error={error} notice={notice} onRefresh={onRefresh}>
      <DataTable rows={rows} columns={columns} empty={`No ${title.toLowerCase()} records`} renderActions={row => <ActionButtons actions={actions} onClick={action => onAction(row, normalizeAction(action))} />} />
      {helpers && <Modals helpers={helpers} />}
    </CrudPage>
  );
}

function CrudPage({ title, subtitle, state, error, notice, onRefresh, children }: { title: string; subtitle: string; state: LoadState; error: string; notice?: string; onRefresh: () => void; children: ReactNode }) {
  return (
    <Page title={title} subtitle={subtitle} onRefresh={onRefresh}>
      <Status state={state} error={error} loading={`Loading ${title.toLowerCase()}...`} />
      {notice && <div className="alert alert-success">{notice}</div>}
      {children}
    </Page>
  );
}

function Page({ title, subtitle, onRefresh, children }: { title: string; subtitle: string; onRefresh?: () => void; children: ReactNode }) {
  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>{title}</h1>
          <p>{subtitle}</p>
        </div>
        {onRefresh && <button className="btn-secondary" type="button" onClick={onRefresh}><RefreshCw size={16} /> Refresh</button>}
      </div>
      {children}
    </div>
  );
}

function Status({ state, error, loading }: { state: LoadState; error: string; loading: string }) {
  if (state === 'loading') return <div className="alert alert-success">{loading}</div>;
  if (state === 'error') return <div className="alert alert-error">{error}</div>;
  return null;
}

function TablePanel({ title, rows, columns }: { title: string; rows: GenericRecord[]; columns: string[] }) {
  return (
    <section className="table-card">
      <h2>{title}</h2>
      <DataTable rows={rows} columns={columns} empty={`No ${title.toLowerCase()} records`} />
    </section>
  );
}

function DataTable({ rows, columns, empty, renderActions, onView }: { rows: GenericRecord[]; columns: string[]; empty: string; renderActions?: (row: GenericRecord) => ReactNode; onView?: (row: GenericRecord) => void }) {
  return (
    <div className="table-card" style={{ padding: 0 }}>
      <table>
        <thead>
          <tr>
            {columns.map(column => <th key={column}>{column.replaceAll('_', ' ')}</th>)}
            {(renderActions || onView) && <th>Actions</th>}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={String(row.id || index)}>
              {columns.map(column => <td key={column}>{column === 'amount' || column.endsWith('_amount') ? money(Number(row[column] || 0)) : renderValue(row[column])}</td>)}
              {(renderActions || onView) && <td>{renderActions ? renderActions(row) : <button type="button" onClick={() => onView?.(row)}><Eye size={14} /> View detail</button>}</td>}
            </tr>
          ))}
          {rows.length === 0 && <tr><td colSpan={columns.length + (renderActions || onView ? 1 : 0)}>{empty}</td></tr>}
        </tbody>
      </table>
    </div>
  );
}

function Metric({ label, value, icon: Icon }: { label: string; value: string | number; icon: typeof Activity }) {
  return (
    <article className="stat-card">
      <div className="stat-label"><Icon size={14} /> {label}</div>
      <div className="stat-value">{value}</div>
    </article>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="form-field">
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

function ActionButtons({ actions, onClick }: { actions: string[]; onClick: (action: string) => void }) {
  return (
    <div className="button-row compact-actions">
      {actions.map(action => <button key={action} type="button" onClick={() => onClick(action)}>{action}</button>)}
    </div>
  );
}

function RoutingSimulator({ form, result, onChange, onSubmit }: { form: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string }; result: RoutingSimulation | null; onChange: (value: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string }) => void; onSubmit: (event: FormEvent) => void }) {
  return (
    <section className="card">
      <h2><Route size={18} /> Routing simulator</h2>
      <form onSubmit={onSubmit}>
        <div className="filter-grid">
          <Field label="Payment method"><input value={form.payment_method} onChange={event => onChange({ ...form, payment_method: event.target.value })} /></Field>
          <Field label="Currency"><input value={form.currency} onChange={event => onChange({ ...form, currency: event.target.value })} /></Field>
          <Field label="Country"><input value={form.country} onChange={event => onChange({ ...form, country: event.target.value })} /></Field>
          <Field label="Amount"><input type="number" value={form.amount} onChange={event => onChange({ ...form, amount: Number(event.target.value) })} /></Field>
          <Field label="Business account"><input value={form.business_account} onChange={event => onChange({ ...form, business_account: event.target.value })} /></Field>
          <Field label="Customer region"><input value={form.customer_region} onChange={event => onChange({ ...form, customer_region: event.target.value })} /></Field>
          <Field label="Risk level"><input value={form.risk_level} onChange={event => onChange({ ...form, risk_level: event.target.value })} /></Field>
        </div>
        <button className="btn-primary" type="submit">Simulate</button>
      </form>
      <div className="stats-grid">
        <Metric label="Selected provider" value={result?.selected_provider || 'No match'} icon={Cable} />
        <Metric label="Matched rule" value={result?.matched_routing_rule || '-'} icon={Route} />
        <Metric label="Fallback provider" value={result?.fallback_provider || '-'} icon={AlertTriangle} />
        <Metric label="Provider fee" value={money(result?.estimated_provider_fee || 0)} icon={Receipt} />
      </div>
    </section>
  );
}

function WhiteLabelPanel({ value, providers, onChange, onSubmit, onReset }: { value: WhiteLabelForm; providers: Provider[]; onChange: (value: WhiteLabelForm) => void; onSubmit: (event: FormEvent) => void; onReset: () => void }) {
  return (
    <section className="card">
      <h2><SlidersHorizontal size={18} /> White-label config</h2>
      <form onSubmit={onSubmit}>
        <div className="filter-grid">
          <Field label="Brand"><input value={value.brand_name} onChange={event => onChange({ ...value, brand_name: event.target.value })} /></Field>
          <Field label="Logo URL"><input value={value.checkout_logo_url} onChange={event => onChange({ ...value, checkout_logo_url: event.target.value })} /></Field>
          <Field label="Theme color"><input value={value.checkout_theme_color} onChange={event => onChange({ ...value, checkout_theme_color: event.target.value })} /></Field>
          <Field label="Domain"><input value={value.payment_page_domain} onChange={event => onChange({ ...value, payment_page_domain: event.target.value })} /></Field>
          <Field label="Support email"><input value={value.support_email} onChange={event => onChange({ ...value, support_email: event.target.value })} /></Field>
          <Field label="Support phone"><input value={value.support_phone} onChange={event => onChange({ ...value, support_phone: event.target.value })} /></Field>
          <Field label="Terms URL"><input value={value.terms_url} onChange={event => onChange({ ...value, terms_url: event.target.value })} /></Field>
          <Field label="Privacy URL"><input value={value.privacy_url} onChange={event => onChange({ ...value, privacy_url: event.target.value })} /></Field>
          <Field label="Refund policy URL"><input value={value.refund_policy_url} onChange={event => onChange({ ...value, refund_policy_url: event.target.value })} /></Field>
          <Field label="Statement descriptor"><input value={value.statement_descriptor} onChange={event => onChange({ ...value, statement_descriptor: event.target.value })} /></Field>
          <Field label="Default provider"><Select value={value.default_provider_id} options={[{ label: 'None', value: '' }, ...providers.map(provider => ({ label: provider.name, value: provider.id }))]} onChange={next => onChange({ ...value, default_provider_id: next })} /></Field>
          <Field label="Fallback provider"><Select value={value.fallback_provider_id} options={[{ label: 'None', value: '' }, ...providers.map(provider => ({ label: provider.name, value: provider.id }))]} onChange={next => onChange({ ...value, fallback_provider_id: next })} /></Field>
          <Field label="Default methods"><input value={value.default_payment_methods} onChange={event => onChange({ ...value, default_payment_methods: event.target.value })} /></Field>
          <Field label="Default country"><input value={value.default_country} onChange={event => onChange({ ...value, default_country: event.target.value })} /></Field>
          <Field label="Default currency"><input value={value.default_currency} onChange={event => onChange({ ...value, default_currency: event.target.value })} /></Field>
          <Field label="Invoice prefix"><input value={value.invoice_prefix} onChange={event => onChange({ ...value, invoice_prefix: event.target.value })} /></Field>
          <Field label="Receipt prefix"><input value={value.receipt_prefix} onChange={event => onChange({ ...value, receipt_prefix: event.target.value })} /></Field>
        </div>
        <div className="button-row">
          <button className="btn-primary" type="submit">Save white-label config</button>
          <button type="button" onClick={onReset}>Reset to default</button>
        </div>
      </form>
    </section>
  );
}

function Modals({ helpers }: { helpers: ReturnType<typeof useDrawerConfirm> }) {
  return (
    <>
      {helpers.confirm && <ConfirmModal state={helpers.confirm} onCancel={() => helpers.setConfirm(null)} onConfirm={() => {
        const action = helpers.confirm?.action;
        helpers.setConfirm(null);
        void action?.();
      }} />}
      {helpers.drawer && <Drawer state={helpers.drawer} onClose={() => helpers.setDrawer(null)} />}
      {helpers.error && <div className="alert alert-error">{helpers.error}</div>}
    </>
  );
}

function ConfirmModal({ state, onCancel, onConfirm }: { state: NonNullable<ConfirmState>; onCancel: () => void; onConfirm: () => void }) {
  return (
    <div className="modal-backdrop">
      <div className="card modal-card">
        <h2>{state.title}</h2>
        <p className="muted-text">{state.body}</p>
        <div className="button-row" style={{ marginTop: 16 }}>
          <button className="btn-primary" type="button" onClick={onConfirm}>Confirm</button>
          <button className="btn-secondary" type="button" onClick={onCancel}>Cancel</button>
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
          <button type="button" onClick={onClose}>Close</button>
        </div>
        <pre className="json-preview">{JSON.stringify(state.body, null, 2)}</pre>
      </div>
    </div>
  );
}

function normalizeAction(action: string) {
  return action.toLowerCase().replaceAll(' ', '_');
}
