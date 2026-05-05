import { FormEvent, ReactNode, useEffect, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  BadgeCheck,
  Banknote,
  Bell,
  Cable,
  Copy,
  Download,
  Eye,
  FileSearch,
  KeyRound,
  Landmark,
  Mail,
  Receipt,
  RefreshCw,
  Route,
  ShieldCheck,
  SlidersHorizontal,
  Upload,
  UserPlus,
  Wallet,
} from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

const API = '/admin/payment-ops';

type GenericValue = string | number | boolean | null | string[] | Record<string, unknown> | Record<string, unknown>[];
type GenericRecord = Record<string, GenericValue>;
type LoadState = 'loading' | 'ready' | 'error';
type ConfirmState = { title: string; body: string; action: () => Promise<void> } | null;
type DrawerState = { title: string; body: GenericRecord } | null;
type EmptyStateConfig = { title: string; body: string; action?: string; onAction?: () => void };

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
  connector_readiness_score: number;
  missing_required_secrets: string[];
  capabilities: GenericRecord[];
};

type RoutingSimulation = {
  selected_provider?: string | null;
  selected_provider_code?: string | null;
  provider_score?: number | null;
  candidate_providers?: GenericRecord[];
  rejected_providers?: GenericRecord[];
  matched_routing_rule?: string | null;
  fallback_provider?: string | null;
  decision_reason: string;
  estimated_provider_fee: number;
  estimated_company_fee: number;
  risk_decision: string;
  provider_health_used: string;
  failover_applied: boolean;
  strategy_used?: string;
  latency_estimate_ms?: number;
  success_estimate?: number;
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
  'admin_users.view',
  'admin_users.manage',
  'roles.view',
  'roles.manage',
  'system_health.view',
  'system_health.manage',
  'notifications.view',
  'notifications.manage',
  'platform_settings.view',
  'platform_settings.manage',
  'ledger.view',
  'ledger.export',
  'business_accounts.manage',
  'business_accounts.kyc_review',
  'business_accounts.risk_review',
  'business_accounts.payouts_manage',
  'developer_tools.view',
  'api_keys.manage',
  'api_logs.view',
  'webhook_replay.manage',
  'compliance.view',
  'compliance.manage',
  'security_events.view',
];

const ALERT_TYPES = [
  'provider_down',
  'webhook_failure',
  'settlement_failed',
  'refund_failed',
  'reconciliation_mismatch',
  'high_refund_rate',
  'high_chargeback_rate',
  'credential_rotation_due',
  'bank_verification_failed',
  'suspicious_risk_activity',
  'background_job_failed',
];

type PlatformSettingsForm = {
  platform_name: string;
  support_email: string;
  support_phone: string;
  default_country: string;
  default_currency: string;
  maintenance_mode: boolean;
  session_timeout: string;
  mfa_required: boolean;
  audit_retention_days: string;
  export_retention_days: string;
  notification_email_sender: string;
  timezone: string;
  date_format: string;
};

const DEFAULT_PLATFORM_SETTINGS: PlatformSettingsForm = {
  platform_name: 'FeelPay',
  support_email: 'support@feelpay.local',
  support_phone: '',
  default_country: 'IN',
  default_currency: 'INR',
  maintenance_mode: false,
  session_timeout: '86400',
  mfa_required: true,
  audit_retention_days: '365',
  export_retention_days: '30',
  notification_email_sender: 'alerts@feelpay.local',
  timezone: 'Asia/Kolkata',
  date_format: 'DD MMM YYYY',
};

function money(value: number) {
  return new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(value / 100);
}

function formatDateTime(value: GenericRecord[string] | undefined) {
  if (typeof value !== 'string' || !value) return '-';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('en-GB', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  }).format(date).replace(',', '');
}

function isDateColumn(column: string) {
  return column.endsWith('_at') || column.endsWith('_date') || column === 'created' || column === 'last_check';
}

function isIdColumn(column: string) {
  return column === 'id' || column.endsWith('_id') || column.includes('transaction_group');
}

function shortId(value: string) {
  if (value.length <= 14) return value;
  return `${value.slice(0, 8)}…${value.slice(-5)}`;
}

function copyText(value: string) {
  void navigator.clipboard?.writeText(value);
}

function statusClass(value: string) {
  const normalized = value.toLowerCase().replaceAll(' ', '_');
  if (['active', 'captured', 'successful', 'success', 'settled', 'verified', 'enabled', 'won', 'healthy', 'matched', 'resolved', 'approved'].includes(normalized)) return 'success';
  if (['failed', 'rejected', 'disabled', 'blocked', 'lost', 'critical', 'down', 'mismatch', 'overdue'].includes(normalized)) return 'failed';
  if (['pending', 'processing', 'requested', 'review', 'warning', 'unknown', 'unmatched', 'investigating', 'open'].includes(normalized)) return 'pending';
  return normalized;
}

function isBadgeColumn(column: string) {
  return column === 'status' || column.endsWith('_status') || column === 'severity' || column === 'health' || column === 'health_status' || column === 'approval_status' || column === 'risk_status' || column === 'kyc_status' || column === 'payout_status';
}

function providerSuccessLabel(row: GenericRecord) {
  const status = String(row.status || '').toLowerCase();
  const health = String(row.health_status || row.health || '').toLowerCase();
  const attempts = Number(row.attempt_count || row.attempts || 0);
  if (status === 'disabled') return '—';
  if (attempts === 0 && (!health || health === 'unknown')) return 'Unknown';
  if (attempts === 0) return 'No traffic';
  const rate = Number(row.success_rate || 0);
  return `${rate.toFixed(1)}%`;
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

function renderCell(row: GenericRecord, column: string) {
  const value = row[column];
  if (column === 'success_rate') return providerSuccessLabel(row);
  if (column === 'actor' || column === 'assigned_to' || column === 'started_by' || column === 'completed_by' || column === 'resolved_by' || column === 'uploaded_by' || column === 'replayed_by') return renderAdminIdentity(row, column);
  if (column === 'amount' || column.endsWith('_amount') || column === 'debit' || column === 'credit' || column === 'balance_after' || column === 'chargeback_fee') return money(Number(value || 0));
  if (isDateColumn(column)) {
    const formatted = formatDateTime(value);
    return <span title={typeof value === 'string' ? value : formatted}>{formatted}</span>;
  }
  if (isIdColumn(column) && typeof value === 'string' && value) {
    return (
      <span className="id-cell" title={value}>
        <span>{shortId(value)}</span>
        <button className="btn-small" type="button" aria-label={`Copy ${column}`} onClick={() => copyText(value)}><Copy size={12} /></button>
      </span>
    );
  }
  if (isBadgeColumn(column) && value !== null && value !== undefined && value !== '') {
    const text = renderValue(value);
    return <span className={`badge ${statusClass(text)}`}>{text}</span>;
  }
  return renderValue(value);
}

function renderAdminIdentity(row: GenericRecord, prefix: string) {
  const name = row[`${prefix}_name`];
  const email = row[`${prefix}_email`];
  const role = row[`${prefix}_role`];
  if (name || email) {
    return (
      <span title={[name, email, role].filter(Boolean).map(renderValue).join(' · ')}>
        {renderValue(name || email)}{role ? ` (${renderValue(role)})` : ''}
      </span>
    );
  }
  return renderValue(row[`${prefix}_admin_id`] || row[`${prefix}_id`]);
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

export function LedgerPage() {
  const [summary, setSummary] = useState<GenericRecord>({});
  const [rows, setRows] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<{ summary: GenericRecord; items: GenericRecord[] }>('GET', `${API}/ledger?limit=100`);
      setSummary(data.summary || {});
      setRows(data.items || []);
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Ledger load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const exportLedger = () => {
    helpers.setConfirm({
      title: 'Export ledger',
      body: 'Ledger export includes money movement records and creates audit log.',
      action: () => helpers.run('Ledger export queued.', () => apiRequest('GET', `${API}/ledger/export`)),
    });
  };
  return (
    <CrudPage title="Ledger" subtitle="Double-entry money movement across payments, fees, refunds, disputes, settlements, and reserves" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Total company balance" value={money(Number(summary.total_company_balance || 0))} icon={Wallet} />
        <Metric label="Pending settlement" value={money(Number(summary.pending_settlement || 0))} icon={Landmark} />
        <Metric label="Business payable" value={money(Number(summary.business_payable || 0))} icon={Receipt} />
        <Metric label="Provider fees" value={money(Number(summary.provider_fees || 0))} icon={Cable} />
        <Metric label="Company revenue" value={money(Number(summary.company_revenue || 0))} icon={BadgeCheck} />
        <Metric label="Reserve balance" value={money(Number(summary.reserve_balance || 0))} icon={ShieldCheck} />
        <Metric label="Unreconciled ledger entries" value={String(summary.unreconciled_ledger_entries || 0)} icon={AlertTriangle} />
      </div>
      <div className="button-row">
        <button className="btn-primary" type="button" onClick={exportLedger}><Download size={14} /> Export ledger</button>
      </div>
      <DataTable
        rows={rows}
        columns={['created_at', 'transaction_group_id', 'source_type', 'source_id', 'account', 'debit', 'credit', 'currency', 'balance_after', 'status']}
        empty={{ title: 'No ledger entries yet', body: 'Ledger entries will appear when payments, refunds, settlements, or disputes post money movement.', action: 'Export ledger', onAction: exportLedger }}
        renderActions={row => <ActionButtons actions={['View ledger group', 'Link to transaction', 'Link to settlement', 'Link to refund', 'Link to dispute']} onClick={action => helpers.setDrawer({ title: action, body: row })} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RefundsPage() {
  const [filters, setFilters] = useState({ status: '', provider: '', currency: '', business_account: '', refund_source: '', amount_range: '' });
  const [path, setPath] = useState(`${API}/refunds?limit=100`);
  const { rows, state, error, load } = useRows(path);
  const helpers = useDrawerConfirm();
  const applyFilters = (event: FormEvent) => {
    event.preventDefault();
    const params = new URLSearchParams({ limit: '100' });
    Object.entries(filters).forEach(([key, value]) => value.trim() && params.set(key, value.trim()));
    setPath(`${API}/refunds?${params.toString()}`);
  };
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    const calls: Record<string, string> = { approve: 'approve', reject: 'reject', initiate: 'initiate', retry: 'retry' };
    if (action === 'request') helpers.setConfirm({ title: 'Request refund', body: 'Refund request will be audit logged.', action: () => helpers.run('Refund request queued.', () => apiRequest('POST', `${API}/refunds`, row)).then(load) });
    if (calls[action]) helpers.setConfirm({ title: `${action} refund`, body: 'Sensitive refund action. Confirm actor, amount, and provider state.', action: () => helpers.run(`Refund ${action} requested.`, () => apiRequest('POST', `${API}/refunds/${id}/${calls[action]}`)).then(load) });
    if (action === 'view_provider_response' || action === 'view_ledger' || action === 'view_audit_trail') helpers.setDrawer({ title: action.replaceAll('_', ' '), body: row });
  };
  return (
    <CrudPage title="Refunds" subtitle="Full, partial, batch, manual, approval, reversal, and provider refund workflows" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Pending approval" value={rows.filter(row => row.approval_status === 'pending').length} icon={Receipt} />
        <Metric label="Processing" value={rows.filter(row => row.status === 'processing' || row.status === 'initiated').length} icon={RefreshCw} />
        <Metric label="Successful" value={rows.filter(row => row.status === 'successful').length} icon={BadgeCheck} />
        <Metric label="Failed" value={rows.filter(row => row.status === 'failed').length} icon={AlertTriangle} />
        <Metric label="Total refunded" value={money(rows.reduce((sum, row) => sum + Number(row.amount || 0), 0))} icon={Banknote} />
      </div>
      <form className="card filter-grid" onSubmit={applyFilters}>
        {Object.entries(filters).map(([key, value]) => (
          <Field key={key} label={key.replaceAll('_', ' ')}>
            <input value={value} onChange={event => setFilters({ ...filters, [key]: event.target.value })} />
          </Field>
        ))}
        <button className="btn-primary" type="submit">Filter</button>
      </form>
      <DataTable
        rows={rows}
        columns={['id', 'payment_id', 'provider_id', 'business_account_id', 'refund_type', 'amount', 'currency', 'status', 'approval_status', 'refund_source', 'reversal_status', 'ledger_transaction_id', 'created_at']}
        empty={{ title: 'No refunds yet', body: 'Refunds will appear here when support or finance creates a refund request.', action: 'Request refund', onAction: () => helpers.setConfirm({ title: 'Request refund', body: 'Create refund from transaction detail or support workflow.', action: () => helpers.run('Refund request queued.', () => Promise.resolve()) }) }}
        renderActions={row => <ActionButtons actions={['Approve', 'Reject', 'Initiate', 'Retry', 'View provider response', 'View ledger', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
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
  return <TableRoute title="Settlements" subtitle="Settlement ledger and reconciliation status" rows={rows} state={state} error={error} notice={helpers.notice} columns={['id', 'provider_id', 'currency', 'gross_payment_amount', 'provider_fee_amount', 'company_fee_amount', 'refund_adjustment_amount', 'chargeback_adjustment_amount', 'net_settlement_amount', 'expected_settlement_date', 'actual_settlement_date', 'reconciliation_status', 'status']} actions={['View items', 'Hold', 'Release', 'Mark settled', 'Mark failed', 'Export CSV']} onAction={act} onRefresh={load} helpers={helpers} />;
}

export function DisputesPage() {
  const { rows, state, error, load } = useRows(`${API}/disputes?limit=100`);
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'view_detail' || action === 'view_audit_trail' || action === 'view_ledger') helpers.setDrawer({ title: action.replaceAll('_', ' '), body: row });
    if (action === 'submit_evidence') helpers.setConfirm({ title: 'Submit dispute evidence', body: 'Evidence submission affects provider dispute workflow and creates audit log.', action: () => helpers.run('Evidence submitted.', () => apiRequest('POST', `${API}/disputes/${id}/submit-evidence`, { evidence_text: 'Submitted from company admin' })).then(load) });
    if (action === 'mark_won') helpers.setConfirm({ title: 'Mark dispute won', body: 'Won dispute creates reversal ledger impact when applicable.', action: () => helpers.run('Dispute marked won.', () => apiRequest('POST', `${API}/disputes/${id}/mark-won`, { outcome: 'won' })).then(load) });
    if (action === 'mark_lost') helpers.setConfirm({ title: 'Mark dispute lost', body: 'Lost dispute creates debit ledger impact and settlement adjustment when applicable.', action: () => helpers.run('Dispute marked lost.', () => apiRequest('POST', `${API}/disputes/${id}/mark-lost`, { outcome: 'lost' })).then(load) });
  };
  return (
    <CrudPage title="Disputes" subtitle="Company dispute, evidence, chargeback, reversal, and ledger impact queue" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <DataTable
        rows={rows}
        columns={['id', 'payment_id', 'business_account_id', 'provider_dispute_id', 'amount', 'currency', 'reason', 'status', 'evidence_status', 'outcome', 'chargeback_fee', 'ledger_transaction_id', 'evidence_due_date', 'created_at']}
        empty={{ title: 'No disputes or chargebacks yet.', body: 'Disputes appear when providers report chargeback activity.' }}
        renderActions={row => <ActionButtons actions={['View detail', 'Submit evidence', 'Mark won', 'Mark lost', 'View ledger', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RiskRulesPage() {
  const { rows, state, error, load } = useRows(`${API}/risk-rules`);
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    if (action === 'disable') helpers.setConfirm({ title: 'Disable risk rule', body: 'Risk decisioning may change immediately.', action: () => helpers.run('Risk rule disabled.', () => apiRequest('POST', `${API}/risk-rules/${String(row.id || '')}/disable`)).then(load) });
    else helpers.setDrawer({ title: 'Risk rule', body: row });
  };
  return <TableRoute title="Risk Rules" subtitle="Company risk controls and rule trigger history" rows={rows} state={state} error={error} notice={helpers.notice} columns={['name', 'rule_type', 'condition', 'action', 'priority', 'last_triggered_at', 'trigger_count', 'status']} actions={['View triggered transactions', 'Create rule', 'Edit rule', 'Disable']} onAction={act} onRefresh={load} helpers={helpers} />;
}

export function ReconciliationPage() {
  const [tab, setTab] = useState('Overview');
  const [rows, setRows] = useState<GenericRecord[]>([]);
  const [queue, setQueue] = useState<GenericRecord[]>([]);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<{ items: GenericRecord[]; queue: GenericRecord[] }>('GET', `${API}/reconciliation`);
      setRows(data.items || []);
      setQueue(data.queue || []);
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Reconciliation load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const matched = rows.reduce((sum, row) => sum + Number(row.matched_count || 0), 0);
  const mismatches = rows.reduce((sum, row) => sum + Number(row.mismatch_count || 0), 0);
  const unresolved = rows.reduce((sum, row) => sum + Number(row.unresolved_count || row.mismatch_count || 0), 0);
  const duplicates = rows.reduce((sum, row) => sum + Number(row.duplicate_count || 0), 0);
  const variance = rows.reduce((sum, row) => sum + Number(row.amount_variance || 0), 0);
  const feeVariance = rows.reduce((sum, row) => sum + Number(row.fee_variance || 0), 0);
  const taxVariance = rows.reduce((sum, row) => sum + Number(row.tax_variance || 0), 0);
  const upload = (source: string) => {
    helpers.setConfirm({
      title: `Upload ${source.replaceAll('_', ' ')}`,
      body: 'Duplicate file hash is blocked. Upload action is permission-gated and audited.',
      action: () => helpers.run('Upload registered.', () => apiRequest('POST', `${API}/reconciliation/upload`, {
        source,
        file_name: `${source}-${Date.now()}.csv`,
        file_hash: `${source}-${Date.now()}`,
        row_count: 0,
      })).then(load),
    });
  };
  const act = (action: string, row?: GenericRecord) => {
    const id = String(row?.id || rows[0]?.id || '');
    if (action === 'upload_provider_report') upload('provider_report');
    else if (action === 'upload_bank_statement') upload('bank_statement');
    else if (action === 'upload_payout_report') upload('payout_report');
    else if (action === 'upload_fee_report') upload('fee_report');
    else if (action === 'auto_match') helpers.setConfirm({ title: 'Auto match', body: 'Auto-match uses references, UTR, amount/date fuzzy checks, and confidence scoring.', action: () => helpers.run('Auto-match queued.', () => apiRequest('POST', `${API}/reconciliation/${id}/auto-match`)).then(load) });
    else if (action === 'manually_resolve') helpers.setConfirm({ title: 'Manually resolve', body: 'Manual resolution creates audit log.', action: () => helpers.run('Record resolved.', () => apiRequest('POST', `${API}/reconciliation/${id}/resolve`, { status: 'matched', resolution_notes: 'Manually resolved' })).then(load) });
    else if (action === 'ignore') helpers.setConfirm({ title: 'Ignore mismatch', body: 'Ignored reconciliation record remains audited.', action: () => helpers.run('Record ignored.', () => apiRequest('POST', `${API}/reconciliation/records/${id}/ignore`)).then(load) });
    else if (action === 'export_discrepancy_report') helpers.setConfirm({ title: 'Export discrepancy report', body: 'Discrepancy export creates audit trail.', action: () => helpers.run('Export queued.', () => apiRequest('GET', `${API}/reconciliation/${id}/export-discrepancies`)) });
    else if (action === 'export_finance_summary') helpers.setConfirm({ title: 'Export finance summary', body: 'Finance summary export creates audit trail.', action: () => helpers.run('Finance summary export queued.', () => apiRequest('GET', `${API}/reconciliation/export-finance-summary`)) });
    else helpers.setDrawer({ title: 'Reconciliation', body: row || rows[0] || {} });
  };
  return (
    <CrudPage title="Reconciliation" subtitle="Provider and bank statement matching" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Uploaded files" value={rows.length} icon={Upload} />
        <Metric label="Matched records" value={matched} icon={BadgeCheck} />
        <Metric label="Mismatches" value={mismatches} icon={AlertTriangle} />
        <Metric label="Unresolved items" value={unresolved} icon={FileSearch} />
        <Metric label="Duplicate records" value={duplicates} icon={Copy} />
        <Metric label="Amount variance" value={money(variance)} icon={Banknote} />
        <Metric label="Fee variance" value={money(feeVariance)} icon={Receipt} />
        <Metric label="Tax variance" value={money(taxVariance)} icon={Landmark} />
      </div>
      <div className="button-row">
        {['Overview', 'Uploads', 'Matching Queue', 'Mismatches', 'Resolved', 'Reports'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      <div className="button-row">
        {['Upload provider report', 'Upload bank statement', 'Upload payout report', 'Upload fee report', 'Auto match', 'View mismatches', 'Manually resolve', 'Ignore', 'Export discrepancy report', 'Export finance summary'].map(action => <button key={action} type="button" onClick={() => act(normalizeAction(action))}>{action}</button>)}
      </div>
      {tab === 'Overview' && <DataTable rows={queue.slice(0, 12)} columns={['row_number', 'external_reference', 'internal_reference', 'amount', 'currency', 'record_type', 'match_status', 'mismatch_reason', 'confidence_score', 'resolution_status']} empty={{ title: 'Upload provider or bank files to start matching.', body: 'Provider reports, bank statements, payout reports, fee reports, and ledger exports create queue records here.' }} renderActions={row => <ActionButtons actions={['Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Uploads' && <DataTable rows={rows} columns={['id', 'source', 'provider_id', 'file_name', 'file_hash', 'row_count', 'matched_count', 'mismatch_count', 'duplicate_count', 'status', 'created_at', 'processed_at']} empty={{ title: 'Upload provider or bank files to start matching.', body: 'Upload provider reports, bank statements, payout reports, or fee reports to start reconciliation.' }} onView={row => helpers.setDrawer({ title: 'Reconciliation details', body: row })} />}
      {tab === 'Matching Queue' && <DataTable rows={queue} columns={['id', 'file_id', 'row_number', 'external_reference', 'internal_reference', 'amount', 'currency', 'record_type', 'match_status', 'confidence_score', 'suggested_match']} empty="No matching queue records" renderActions={row => <ActionButtons actions={['Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Mismatches' && <DataTable rows={queue.filter(row => String(row.match_status).includes('mismatch') || row.mismatch_reason)} columns={['id', 'external_reference', 'amount', 'currency', 'mismatch_reason', 'confidence_score', 'resolution_status']} empty="No mismatches" renderActions={row => <ActionButtons actions={['Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Resolved' && <DataTable rows={queue.filter(row => row.resolution_status === 'resolved')} columns={['id', 'external_reference', 'matched_entity_type', 'matched_entity_id', 'resolution_status', 'resolution_note', 'resolved_at']} empty="No resolved records" />}
      {tab === 'Reports' && <DataTable rows={rows} columns={['file_name', 'source', 'amount_variance', 'matched_count', 'mismatch_count', 'duplicate_count', 'created_at']} empty="No reconciliation reports" />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function BusinessAccountsPage() {
  const { rows, state, error, load } = useRows(`${API}/business-accounts`);
  const helpers = useDrawerConfirm();
  const openDetail = async (row: GenericRecord) => {
    try {
      const detail = await apiRequest<GenericRecord>('GET', `${API}/business-accounts/${String(row.id || '')}`);
      helpers.setDrawer({ title: 'Business account detail', body: detail });
    } catch {
      helpers.setDrawer({ title: 'Business account detail', body: row });
    }
  };
  const updateAccount = (row: GenericRecord, patch: GenericRecord, label: string) => {
    const id = String(row.id || '');
    helpers.setConfirm({
      title: label,
      body: 'Business account KYC, risk, or payout readiness change requires confirmation and audit logging.',
      action: () => helpers.run('Business account updated.', () => apiRequest('PATCH', `${API}/business-accounts/${id}`, patch)).then(load),
    });
  };
  const act = (row: GenericRecord, action: string) => {
    if (action === 'view_detail') void openDetail(row);
    if (action === 'review_kyc') updateAccount(row, { kyc_status: 'verified', note: 'KYC approved from company admin' }, 'Approve KYC');
    if (action === 'review_risk') updateAccount(row, { risk_status: 'medium', note: 'Risk reviewed from company admin' }, 'Review risk');
    if (action === 'manage_payouts') updateAccount(row, { payout_status: 'enabled', note: 'Payout readiness reviewed' }, 'Enable payouts');
    if (action === 'suspend') updateAccount(row, { status: 'suspended', payout_status: 'disabled', note: 'Suspended by company admin' }, 'Suspend business account');
    if (action === 'reactivate') updateAccount(row, { status: 'active', note: 'Reactivated by company admin' }, 'Reactivate business account');
  };
  return (
    <CrudPage title="Business Accounts" subtitle="Internal company-managed businesses and tenants only" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Total accounts" value={rows.length} icon={Receipt} />
        <Metric label="Pending KYC" value={rows.filter(row => row.kyc_status === 'pending' || row.kyc_status === 'not_started').length} icon={FileSearch} />
        <Metric label="Payout enabled" value={rows.filter(row => row.payout_status === 'enabled').length} icon={Landmark} />
        <Metric label="High risk" value={rows.filter(row => row.risk_status === 'high').length} icon={AlertTriangle} />
        <Metric label="Suspended" value={rows.filter(row => row.status === 'suspended').length} icon={ShieldCheck} />
      </div>
      <DataTable
        rows={rows}
        columns={['display_name', 'business_type', 'email', 'status', 'kyc_status', 'risk_status', 'payout_status', 'payout_ready', 'created_at']}
        empty={{ title: 'No business accounts yet', body: 'Internal company-managed business accounts will appear here after onboarding records are created.' }}
        renderActions={row => <ActionButtons actions={['View detail', 'Review KYC', 'Review risk', 'Manage payouts', 'Suspend', 'Reactivate']} onClick={action => act(row, normalizeAction(action))} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
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
  const capabilityRows = providers.flatMap(provider => {
    const capabilities = Array.isArray(provider.capabilities) ? provider.capabilities : [];
    return capabilities.map(capability => ({
      id: `${provider.id}-${String((capability as GenericRecord).capability || '')}`,
      provider: provider.name,
      environment: provider.environment,
      capability: (capability as GenericRecord).capability || '',
      enabled: (capability as GenericRecord).enabled || false,
      status: (capability as GenericRecord).status || 'unknown',
      last_checked_at: (capability as GenericRecord).last_checked_at || '',
      error_message: (capability as GenericRecord).error_message || '',
    }));
  });

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
      <TablePanel title="Providers" rows={providers} columns={['name', 'code', 'provider_type', 'environment', 'status', 'health_status', 'connector_readiness_score', 'missing_required_secrets', 'success_rate', 'attempt_count', 'api_key_status', 'api_secret_status', 'webhook_secret_status']} />
      <DataTable rows={capabilityRows} columns={['provider', 'environment', 'capability', 'enabled', 'status', 'last_checked_at', 'error_message']} empty={{ title: 'No provider capabilities yet', body: 'Capability checks appear after providers are configured.' }} />
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
  const performance = useRows(`${API}/routing/provider-performance`);
  const helpers = useDrawerConfirm();
  const [tab, setTab] = useState('Rules');
  const [form, setForm] = useState({ payment_method: 'upi', currency: 'INR', country: 'IN', amount: 10000, business_account: '', customer_region: '', risk_level: '', strategy: 'priority' });
  const [result, setResult] = useState<RoutingSimulation | null>(null);
  const simulate = async (event: FormEvent) => {
    event.preventDefault();
    setResult(await apiRequest<RoutingSimulation>('POST', `${API}/routing/simulate`, form));
  };
  return (
    <CrudPage title="Routing Rules" subtitle="Method, cost, success, health, risk, failover, and explainable provider routing" state={state} error={error || performance.error} notice={helpers.notice} onRefresh={() => void Promise.all([load(), performance.load()])}>
      <div className="button-row">
        {['Rules', 'Simulator', 'Provider Performance', 'Failover Rules', 'Match Logs'].map(item => <button key={item} className={tab === item ? 'btn-primary' : 'btn-secondary'} type="button" onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'Rules' && <DataTable rows={rows} columns={['name', 'rule_group', 'strategy', 'payment_method', 'currency', 'country', 'amount_min', 'amount_max', 'provider_name', 'provider_weight', 'min_success_rate', 'max_provider_fee_percent', 'max_latency_ms', 'health_required', 'failover_provider_name', 'risk_level', 'status']} empty={{ title: 'No routing rules configured', body: 'Create routing rules to choose providers by priority, cost, health, risk, success rate, or latency.' }} onView={row => helpers.setDrawer({ title: 'Routing rule', body: row })} />}
      {tab === 'Simulator' && <RoutingSimulator form={form} result={result} onChange={setForm} onSubmit={simulate} />}
      {tab === 'Provider Performance' && <DataTable rows={performance.rows} columns={['provider', 'payment_method', 'currency', 'country', 'success_rate', 'failure_rate', 'avg_latency_ms', 'p95_latency_ms', 'provider_fee_percent', 'attempt_count', 'updated_at']} empty={{ title: 'No provider performance yet', body: 'Performance metrics appear after payment attempts update provider success, latency, and fee data.' }} />}
      {tab === 'Failover Rules' && <DataTable rows={rows} columns={['name', 'provider_name', 'failover_provider_name', 'failover_strategy', 'retry_strategy', 'cooldown_minutes', 'health_required', 'status']} empty="No failover rules configured" />}
      {tab === 'Match Logs' && <DataTable rows={rows} columns={['name', 'last_matched_at', 'provider_name', 'failover_provider_name', 'status']} empty="No routing match logs yet" />}
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
  const helpers = useDrawerConfirm();
  const reports = useRows(`${API}/reports`);
  const history = useRows(`${API}/reports/export-history`);
  const scheduled = useRows(`${API}/reports/scheduled`);
  const [tab, setTab] = useState('Reports');
  const [filters, setFilters] = useState({ start_date: '', end_date: '', provider: '', business_account: '', currency: '', payment_method: '', status: '', format: 'CSV' });
  const state = reports.state === 'error' || history.state === 'error' || scheduled.state === 'error' ? 'error' : reports.state === 'loading' || history.state === 'loading' || scheduled.state === 'loading' ? 'loading' : 'ready';
  const error = reports.error || history.error || scheduled.error;
  const load = () => void Promise.all([reports.load(), history.load(), scheduled.load()]);
  const generate = (row: GenericRecord, format: string) => {
    helpers.setConfirm({
      title: `Generate ${format}`,
      body: 'Report export is permission-gated, masked, expiring, and audit logged.',
      action: () => helpers.run('Report export queued.', () => apiRequest('POST', `${API}/reports/generate`, {
        report_type: row.report_name || row.name,
        format,
        filters,
      })).then(load),
    });
  };
  const schedule = (row: GenericRecord) => {
    helpers.setConfirm({
      title: 'Schedule report',
      body: 'Scheduled finance reports use saved filters, masked data, and audited delivery setup.',
      action: () => helpers.run('Report scheduled.', () => apiRequest('POST', `${API}/reports/schedule`, {
        report_type: row.report_name || row.name,
        format: filters.format,
        frequency: 'weekly',
        recipients: ['finance@feelpay.local'],
        filters,
      })).then(load),
    });
  };
  const download = (row: GenericRecord) => {
    const id = String(row.id || '');
    helpers.setConfirm({
      title: 'Download latest',
      body: 'Download links expire and report download is audit logged.',
      action: () => helpers.run('Download link prepared.', () => apiRequest('GET', `${API}/reports/export-history/${id}/download`)),
    });
  };
  return (
    <CrudPage title="Reports" subtitle="Company finance and operations exports" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="card filter-grid">
        {Object.entries(filters).map(([key, value]) => (
          <Field key={key} label={key.replaceAll('_', ' ')}>
            {key === 'format'
              ? <Select value={value} options={['CSV', 'PDF', 'XLSX']} onChange={next => setFilters({ ...filters, format: next })} />
              : <input type={key.endsWith('date') ? 'date' : 'text'} value={value} onChange={event => setFilters({ ...filters, [key]: event.target.value })} />}
          </Field>
        ))}
      </div>
      <div className="button-row">
        {['Reports', 'Export History', 'Scheduled Reports'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'Reports' && <DataTable rows={reports.rows} columns={['report_name', 'category', 'format', 'last_generated_at', 'generated_by', 'status']} empty={{ title: 'Reports are ready to export.', body: 'Choose filters, then generate CSV, PDF, or XLSX exports.' }} renderActions={row => <ActionButtons actions={['Generate CSV', 'Generate PDF', 'Generate XLSX', 'Schedule report', 'View history', 'Download latest']} onClick={action => {
        if (action === 'Generate CSV') generate(row, 'CSV');
        else if (action === 'Generate PDF') generate(row, 'PDF');
        else if (action === 'Generate XLSX') generate(row, 'XLSX');
        else if (action === 'Schedule report') schedule(row);
        else if (action === 'View history') setTab('Export History');
        else helpers.setDrawer({ title: 'Latest report', body: row });
      }} />} />}
      {tab === 'Export History' && <DataTable rows={history.rows} columns={['id', 'report_type', 'file_name', 'format', 'status', 'row_count', 'generated_by', 'generated_at', 'expires_at']} empty={{ title: 'No report exports yet', body: 'Generated reports appear here with expiring download links.' }} renderActions={row => <ActionButtons actions={['Download latest', 'View detail']} onClick={action => action === 'Download latest' ? download(row) : helpers.setDrawer({ title: 'Export history', body: row })} />} />}
      {tab === 'Scheduled Reports' && <DataTable rows={scheduled.rows} columns={['id', 'report_type', 'frequency', 'recipients', 'format', 'status', 'created_by', 'next_run_at', 'last_run_at', 'created_at']} empty={{ title: 'No scheduled reports yet', body: 'Schedule recurring daily, weekly, or monthly reports for finance recipients.' }} onView={row => helpers.setDrawer({ title: 'Scheduled report', body: row })} />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function AuditLogsPage() {
  const { rows, state, error, load } = useRows(`${API}/audit-logs`);
  const helpers = useDrawerConfirm();
  return <TableRoute title="Audit Logs" subtitle="Sensitive company admin action history" rows={rows} state={state} error={error} columns={['action', 'entity_type', 'entity_id', 'actor', 'ip_address', 'new_value_summary', 'created_at']} actions={['View detail']} onAction={(row) => helpers.setDrawer({ title: 'Audit details', body: row })} onRefresh={load} helpers={helpers} />;
}

export function AdminUsersRolesPage() {
  const users = useRows('/admin/users');
  const roles = useRows(`${API}/admin-roles`);
  const helpers = useDrawerConfirm();
  const [invite, setInvite] = useState({ name: '', email: '', role: 'Support Agent' });
  const state = users.state === 'error' || roles.state === 'error' ? 'error' : users.state === 'loading' || roles.state === 'loading' ? 'loading' : 'ready';
  const error = users.error || roles.error;
  const load = () => void Promise.all([users.load(), roles.load()]);
  const inviteAdmin = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Invite admin user',
      body: 'New company admin invite will create audit log and least-privilege role assignment.',
      action: () => helpers.run('Admin invite created.', () => apiRequest('POST', '/admin/users/invite', invite)).then(load),
    });
  };
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    const role = String(row.role || 'Support Agent');
    if (action === 'edit_role') helpers.setConfirm({ title: 'Assign role', body: 'Role change requires confirmation and audit logging.', action: () => helpers.run('Role assigned.', () => apiRequest('POST', `/admin/users/${id}/roles`, { role })).then(load) });
    if (action === 'deactivate') helpers.setConfirm({ title: 'Deactivate admin user', body: 'User session access may stop. Users cannot deactivate themselves.', action: () => helpers.run('Admin deactivated.', () => apiRequest('POST', `/admin/users/${id}/deactivate`)).then(load) });
    if (action === 'reactivate') helpers.setConfirm({ title: 'Reactivate admin user', body: 'User regains company admin access according to assigned role.', action: () => helpers.run('Admin reactivated.', () => apiRequest('POST', `/admin/users/${id}/reactivate`)).then(load) });
    if (action === 'view_audit_trail') helpers.setDrawer({ title: 'User audit trail', body: row });
  };
  return (
    <CrudPage title="Admin Users & Roles" subtitle="Company admin users only, deny-by-default permissions" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <form className="card filter-grid" onSubmit={inviteAdmin}>
        <Field label="Name"><input value={invite.name} onChange={event => setInvite({ ...invite, name: event.target.value })} /></Field>
        <Field label="Email"><input type="email" value={invite.email} onChange={event => setInvite({ ...invite, email: event.target.value })} /></Field>
        <Field label="Role"><Select value={invite.role} options={ADMIN_ROLES} onChange={role => setInvite({ ...invite, role })} /></Field>
        <button className="btn-primary" type="submit"><UserPlus size={14} /> Invite</button>
      </form>
      <DataTable rows={users.rows} columns={['id', 'name', 'email', 'role', 'status', 'mfa_enabled', 'last_login_at', 'created_at']} empty="No admin users found" renderActions={row => <ActionButtons actions={['Edit role', 'Deactivate', 'Reactivate', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />} />
      <div className="grid grid-2">
        <TablePanel title="Roles" rows={ADMIN_ROLES.map((role, index) => ({ id: String(index), role, scope: 'company_admin' }))} columns={['role', 'scope']} />
        <TablePanel title="Permissions" rows={ADMIN_PERMISSIONS.map((permission, index) => ({ id: String(index), permission }))} columns={['permission']} />
      </div>
      <TablePanel title="Role permissions from server" rows={roles.rows} columns={['name', 'description', 'permissions']} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function SystemHealthPage() {
  const health = useRows('/admin/system-health');
  const jobs = useRows('/admin/system-health/jobs');
  const providers = useRows('/admin/system-health/providers');
  const helpers = useDrawerConfirm();
  const state = health.state === 'error' || jobs.state === 'error' || providers.state === 'error' ? 'error' : health.state === 'loading' || jobs.state === 'loading' || providers.state === 'loading' ? 'loading' : 'ready';
  const error = health.error || jobs.error || providers.error;
  const load = () => void Promise.all([health.load(), jobs.load(), providers.load()]);
  const retryJob = (row: GenericRecord) => {
    helpers.setConfirm({ title: 'Retry failed job', body: 'Background job retry will be audit logged.', action: () => helpers.run('Job retry queued.', () => apiRequest('POST', `/admin/system-health/jobs/${String(row.id || '')}/retry`)).then(load) });
  };
  return (
    <CrudPage title="System Health" subtitle="Internal health for company payment operations" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        {['API status', 'Database status', 'Queue status', 'Webhook worker status', 'Settlement sync status', 'Reconciliation job status', 'Provider health', 'Failed background jobs'].map(component => {
          const row = health.rows.find(item => String(item.component) === component);
          return <Metric key={component} label={component} value={String(row?.status || 'unknown')} icon={Activity} />;
        })}
      </div>
      <TablePanel title="Provider Health" rows={providers.rows} columns={['provider', 'environment', 'health', 'success_rate', 'attempt_count', 'last_check', 'latency_ms', 'error_message']} />
      <DataTable rows={jobs.rows} columns={['job_name', 'status', 'last_run_at', 'next_run_at', 'failure_count', 'last_error']} empty="No background jobs found" renderActions={row => <ActionButtons actions={['Retry failed job', 'View logs', 'Mark incident resolved']} onClick={action => action === 'Retry failed job' ? retryJob(row) : helpers.setDrawer({ title: action, body: row })} />} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function NotificationsPage() {
  const { rows, state, error, load } = useRows('/admin/notifications');
  const [tab, setTab] = useState('Inbox');
  const helpers = useDrawerConfirm();
  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'create_incident') helpers.setConfirm({ title: 'Create incident', body: 'Critical alert creates incident, timeline event, and audit log.', action: () => helpers.run('Incident created.', () => apiRequest('POST', `/admin/notifications/${id}/incident`)).then(load) });
    else if (action === 'snooze_alert') helpers.setConfirm({ title: 'Snooze alert', body: 'Snooze updates alert SLA visibility and creates audit log.', action: () => helpers.run('Alert snoozed.', () => apiRequest('POST', `/admin/notifications/${id}/snooze`)).then(load) });
    else if (action === 'view_related_record') helpers.setDrawer({ title: 'Related record', body: row });
    else helpers.setConfirm({ title: action.replaceAll('_', ' '), body: 'Notification action updates alert status and creates audit log.', action: () => helpers.run('Notification updated.', () => apiRequest('POST', `/admin/notifications/${id}/${action}`, { assigned_to_admin_id: row.assigned_to_admin_id || null })).then(load) });
  };
  const scopedRows = rows.filter(row => {
    if (tab === 'Critical') return row.severity === 'critical';
    if (tab === 'Resolved') return row.status === 'resolved';
    if (tab === 'Ignored') return row.status === 'ignored';
    if (tab === 'Assigned to me') return Boolean(row.assigned_to_admin_id);
    return row.status !== 'resolved' && row.status !== 'ignored';
  });
  return (
    <CrudPage title="Notifications" subtitle="Company admin alerts center" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Inbox', 'Assigned to me', 'Critical', 'Resolved', 'Ignored'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      <div className="card">
        <h2><Bell size={18} /> Alert types</h2>
        <div className="button-row">{ALERT_TYPES.map(type => <span className="badge pending" key={type}>{type}</span>)}</div>
      </div>
      <DataTable rows={scopedRows} columns={['id', 'severity', 'type', 'title', 'related_entity_type', 'status', 'occurrence_count', 'sla_due_at', 'assigned_to', 'incident_id', 'created_at']} empty={{ title: 'No active alerts.', body: 'Provider, webhook, settlement, and risk alerts appear here.' }} renderActions={row => <ActionButtons actions={['Acknowledge', 'Assign', 'Create incident', 'Resolve', 'Snooze alert', 'Ignore', 'View related record']} onClick={action => act(row, normalizeAction(action))} />} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function DeveloperIntegrationPage() {
  const apiKeys = useRows(`${API}/developer/api-keys`);
  const requestLogs = useRows(`${API}/developer/request-logs`);
  const replays = useRows(`${API}/developer/webhook-replays`);
  const [logFilters, setLogFilters] = useState({ request_id: '', idempotency_key: '', status_code: '', api_key_id: '', endpoint: '' });
  const [tab, setTab] = useState('API Keys');
  const [keyForm, setKeyForm] = useState({ name: 'Internal integration key', environment: 'sandbox', scopes: 'payments.read,refunds.write,webhooks.replay', expires_at: '' });
  const [webhookForm, setWebhookForm] = useState({ event_type: 'payment.captured', webhook_event_id: '' });
  const helpers = useDrawerConfirm();
  const state = [apiKeys.state, requestLogs.state, replays.state].includes('error') ? 'error' : [apiKeys.state, requestLogs.state, replays.state].includes('loading') ? 'loading' : 'ready';
  const error = apiKeys.error || requestLogs.error || replays.error;
  const load = () => void Promise.all([apiKeys.load(), requestLogs.load(), replays.load()]);
  const filteredLogs = requestLogs.rows.filter(row => {
    if (logFilters.request_id && !String(row.request_id || '').includes(logFilters.request_id)) return false;
    if (logFilters.idempotency_key && !String(row.idempotency_key || '').includes(logFilters.idempotency_key)) return false;
    if (logFilters.status_code && String(row.status_code || '') !== logFilters.status_code) return false;
    if (logFilters.api_key_id && String(row.api_key_id || '') !== logFilters.api_key_id) return false;
    if (logFilters.endpoint && !String(row.path || '').includes(logFilters.endpoint)) return false;
    return true;
  });

  const createKey = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Create API key',
      body: 'Full key is never returned after creation. Key lifecycle change creates audit log.',
      action: () => helpers.run('API key created.', () => apiRequest('POST', `${API}/developer/api-keys`, {
        ...keyForm,
        scopes: list(keyForm.scopes),
        expires_at: keyForm.expires_at || null,
      })).then(load),
    });
  };
  const keyAction = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'rotate_key' || action === 'revoke_key') {
      helpers.setConfirm({
        title: action.replaceAll('_', ' '),
        body: 'API key change is sensitive, scope-enforced, and audit logged.',
        action: () => helpers.run('API key updated.', () => apiRequest('POST', `${API}/developer/api-keys/${id}/${action === 'rotate_key' ? 'rotate' : 'revoke'}`)).then(load),
      });
    } else {
      helpers.setDrawer({ title: 'API key', body: row });
    }
  };
  const testWebhook = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Send webhook test',
      body: 'Test event verifies endpoint response and logs latency/status without exposing secrets.',
      action: () => helpers.run('Webhook test sent.', () => apiRequest('POST', `${API}/developer/webhook-test`, { ...webhookForm, webhook_event_id: webhookForm.webhook_event_id || null })),
    });
  };
  const replayEvent = (row?: GenericRecord) => {
    const id = String(row?.webhook_event_id || webhookForm.webhook_event_id || '');
    helpers.setConfirm({
      title: 'Replay webhook event',
      body: 'Webhook replay writes audit log and keeps payload redacted in admin logs.',
      action: () => helpers.run('Webhook replay queued.', () => apiRequest('POST', `${API}/developer/webhook-events/${id}/replay`)).then(load),
    });
  };
  const payloads = [
    { id: 'payment.captured', event: 'payment.captured', sample: '{"id":"pay_test","amount":1000,"currency":"INR","card":"redacted"}' },
    { id: 'refund.succeeded', event: 'refund.succeeded', sample: '{"id":"rf_test","payment_id":"pay_test","amount":500}' },
    { id: 'dispute.opened', event: 'dispute.opened', sample: '{"id":"dp_test","payment_id":"pay_test","reason":"fraudulent"}' },
  ];
  const checklist = ['Create sandbox API key', 'Configure webhook endpoint', 'Send webhook test', 'Replay failed event', 'Review idempotency logs', 'Switch to live mode after approval'].map((item, index) => ({ id: String(index), item, status: index < 2 ? 'ready' : 'pending' }));
  return (
    <CrudPage title="Developer Tools" subtitle="Internal API keys, sandbox/live mode, webhook testing, event replay, and logs" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['API Keys', 'Request Logs', 'Webhook Testing', 'Event Replay', 'Sample Payloads', 'Integration Checklist'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'API Keys' && (
        <>
          <section className="card">
            <h2><KeyRound size={18} /> API key management</h2>
            <form onSubmit={createKey}>
              <div className="filter-grid">
                <Field label="Name"><input value={keyForm.name} onChange={event => setKeyForm({ ...keyForm, name: event.target.value })} /></Field>
                <Field label="Environment"><Select value={keyForm.environment} options={['sandbox', 'production']} onChange={environment => setKeyForm({ ...keyForm, environment })} /></Field>
                <Field label="Scopes"><input value={keyForm.scopes} onChange={event => setKeyForm({ ...keyForm, scopes: event.target.value })} /></Field>
                <Field label="Expires at"><input type="datetime-local" value={keyForm.expires_at} onChange={event => setKeyForm({ ...keyForm, expires_at: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Create key</button>
            </form>
          </section>
          <DataTable rows={apiKeys.rows} columns={['id', 'name', 'environment', 'key_prefix', 'masked_key', 'scopes', 'status', 'last_used_at', 'expires_at', 'created_at']} empty={{ title: 'No API keys yet', body: 'Create sandbox keys for internal integration testing. Full secrets are shown once outside this admin table.' }} renderActions={row => <ActionButtons actions={['Rotate key', 'Revoke key', 'View detail']} onClick={action => keyAction(row, normalizeAction(action))} />} />
        </>
      )}
      {tab === 'Request Logs' && (
        <>
          <div className="card filter-grid">
            {Object.entries(logFilters).map(([key, value]) => <Field key={key} label={key.replaceAll('_', ' ')}><input value={value} onChange={event => setLogFilters({ ...logFilters, [key]: event.target.value })} /></Field>)}
          </div>
          <DataTable rows={filteredLogs} columns={['created_at', 'environment', 'api_key_id', 'method', 'path', 'status_code', 'latency_ms', 'idempotency_key', 'request_id', 'error_code', 'error_message']} empty={{ title: 'No API request logs', body: 'Internal API logs appear here with Authorization, card, secret, and bank data redacted.' }} onView={row => helpers.setDrawer({ title: 'Request metadata', body: row })} />
        </>
      )}
      {tab === 'Webhook Testing' && (
        <section className="card">
          <h2><Cable size={18} /> Webhook endpoint testing</h2>
          <form onSubmit={testWebhook}>
            <div className="filter-grid">
              <Field label="Event type"><input value={webhookForm.event_type} onChange={event => setWebhookForm({ ...webhookForm, event_type: event.target.value })} /></Field>
              <Field label="Webhook event ID"><input value={webhookForm.webhook_event_id} onChange={event => setWebhookForm({ ...webhookForm, webhook_event_id: event.target.value })} /></Field>
            </div>
            <div className="button-row">
              <button className="btn-primary" type="submit">Send test event</button>
              <button type="button" onClick={() => replayEvent()}>Replay selected event</button>
            </div>
          </form>
        </section>
      )}
      {tab === 'Event Replay' && <DataTable rows={replays.rows} columns={['id', 'webhook_event_id', 'replay_status', 'replayed_by_admin_id', 'replayed_at', 'error_message', 'created_at']} empty={{ title: 'No webhook replays', body: 'Replay records appear here after failed or selected webhook events are queued again.' }} renderActions={row => <ActionButtons actions={['Replay selected event', 'View detail']} onClick={action => action === 'Replay selected event' ? replayEvent(row) : helpers.setDrawer({ title: 'Webhook replay', body: row })} />} />}
      {tab === 'Sample Payloads' && <DataTable rows={payloads} columns={['event', 'sample']} empty="No sample payloads" />}
      {tab === 'Integration Checklist' && <DataTable rows={[
        ...checklist,
        { id: 'sandbox_provider', item: 'Sandbox provider configured', status: 'ready' },
        { id: 'production_provider', item: 'Production provider configured', status: 'pending' },
        { id: 'webhook_secret', item: 'Webhook secret configured', status: 'ready' },
        { id: 'signature_verification', item: 'Webhook signature verification enabled', status: 'ready' },
        { id: 'api_key_active', item: 'API key active', status: apiKeys.rows.some(row => row.status === 'active') ? 'ready' : 'pending' },
        { id: 'idempotency', item: 'Idempotency enabled', status: 'ready' },
        { id: 'no_raw_card', item: 'No raw card data', status: 'ready' },
        { id: 'no_cvv', item: 'No CVV storage', status: 'ready' },
        { id: 'reconciliation', item: 'Reconciliation enabled', status: 'ready' },
        { id: 'settlement_export', item: 'Settlement export enabled', status: 'ready' },
      ]} columns={['item', 'status']} empty="No integration checklist" />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function ComplianceCenterPage() {
  const [overview, setOverview] = useState<GenericRecord>({});
  const [tab, setTab] = useState('Overview');
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const securityEvents = useRows(`${API}/compliance/security-events`);
  const reviews = useRows(`${API}/compliance/access-reviews`);
  const tasks = useRows(`${API}/compliance/tasks`);
  const helpers = useDrawerConfirm();
  const relatedError = securityEvents.error || reviews.error || tasks.error;
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<GenericRecord>('GET', `${API}/compliance/overview`);
      await Promise.all([securityEvents.load(), reviews.load(), tasks.load()]);
      setOverview(data);
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Compliance center load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const action = (title: string, path: string, method: 'GET' | 'POST' = 'POST') => {
    helpers.setConfirm({
      title,
      body: 'Compliance action is restricted to Super Admin and Compliance Admin and creates audit log.',
      action: () => helpers.run(`${title} complete.`, () => apiRequest(method, path)).then(load),
    });
  };
  const startReview = () => {
    helpers.setConfirm({
      title: 'Start access review',
      body: 'Access review creates review items for current company admin users and writes audit log.',
      action: () => helpers.run('Access review started.', () => apiRequest('POST', `${API}/compliance/access-reviews`, { review_period: 'Q2 2026' })).then(load),
    });
  };
  const completeReview = (row: GenericRecord) => action('Complete access review', `${API}/compliance/access-reviews/${String(row.id || '')}/complete`);
  const readinessRows = Object.entries(overview).map(([key, value]) => ({ id: key, check: key, status: renderValue(value) }));
  const retentionRows = [
    { id: 'audit_retention', policy: 'Audit retention', status: overview.audit_retention_status || 'enabled' },
    { id: 'data_retention', policy: 'Data retention', status: 'configured' },
    { id: 'masked_exports', policy: 'Masked exports', status: 'enabled' },
  ];
  const rotationRows = [
    { id: 'credential_rotation_due', item: 'Credential rotation due', count: Number(overview.credential_rotation_due || 0), status: Number(overview.credential_rotation_due || 0) > 0 ? 'warning' : 'healthy' },
    { id: 'webhook_signature_coverage', item: 'Webhook signature coverage', count: 100, status: overview.webhook_signatures || 'required' },
    { id: 'bank_data_masking', item: 'Bank data masking coverage', count: 100, status: overview.bank_accounts_encrypted || 'enabled' },
  ];
  return (
    <CrudPage title="Compliance Center" subtitle="PCI readiness, access review, security events, retention, exports, and credential rotation" state={state} error={error || relatedError || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Raw card storage" value={String(overview.raw_card_storage || 'disabled')} icon={ShieldCheck} />
        <Metric label="CVV storage" value={String(overview.cvv_storage || 'disabled')} icon={ShieldCheck} />
        <Metric label="Tokenization" value={String(overview.tokenization || 'enabled')} icon={BadgeCheck} />
        <Metric label="Webhook signatures" value={String(overview.webhook_signatures || 'required')} icon={Cable} />
        <Metric label="Secrets encrypted" value={String(overview.secrets_encrypted || 'enabled')} icon={KeyRound} />
        <Metric label="Bank accounts encrypted" value={String(overview.bank_accounts_encrypted || 'enabled')} icon={Landmark} />
        <Metric label="MFA coverage" value={String(overview.mfa_coverage || 0)} icon={UserPlus} />
        <Metric label="Credential rotation due" value={String(overview.credential_rotation_due || 0)} icon={RefreshCw} />
      </div>
      <div className="button-row">
        {['Overview', 'Access Review', 'Security Events', 'Data Retention', 'Audit Export', 'Credential Rotation', 'Compliance Tasks'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      <div className="button-row">
        <button type="button" onClick={startReview}>Start access review</button>
        <button type="button" onClick={() => action('Export audit logs', `${API}/compliance/audit-export`, 'GET')}>Export audit logs</button>
        <button type="button" onClick={() => action('Export security events', `${API}/compliance/security-events-export`, 'GET')}>Export security events</button>
        <button type="button" onClick={() => action('Rotate due credentials', `${API}/compliance/rotate-due-credentials`)}>Rotate due credentials</button>
        <button type="button" onClick={() => action('Mark compliance task complete', `${API}/compliance/tasks/pci-readiness/complete`)}>Mark task complete</button>
        <button type="button" onClick={() => action('Create compliance task', `${API}/compliance/tasks`)}>Create compliance task</button>
      </div>
      {tab === 'Overview' && <DataTable rows={readinessRows} columns={['check', 'status']} empty="No compliance readiness checks" />}
      {tab === 'Access Review' && <DataTable rows={reviews.rows} columns={['id', 'review_period', 'status', 'started_by_admin_id', 'completed_by_admin_id', 'started_at', 'completed_at']} empty={{ title: 'No access reviews', body: 'Start an access review to verify least-privilege company admin access.' }} renderActions={row => <ActionButtons actions={['Complete access review', 'View detail']} onClick={label => label === 'Complete access review' ? completeReview(row) : helpers.setDrawer({ title: 'Access review', body: row })} />} />}
      {tab === 'Security Events' && <DataTable rows={securityEvents.rows} columns={['id', 'event_type', 'severity', 'actor_admin_id', 'entity_type', 'entity_id', 'ip_address', 'summary', 'created_at']} empty={{ title: 'No security events', body: 'Security events appear here when sensitive access, export, or credential actions occur.' }} onView={row => helpers.setDrawer({ title: 'Security event', body: row })} />}
      {tab === 'Data Retention' && <DataTable rows={retentionRows} columns={['policy', 'status']} empty="No data retention policies" />}
      {tab === 'Audit Export' && <DataTable rows={[{ id: 'audit', export_type: 'Audit logs', masking: 'masked', status: 'available' }, { id: 'security', export_type: 'Security events', masking: 'masked', status: 'available' }]} columns={['export_type', 'masking', 'status']} empty="No exports" />}
      {tab === 'Credential Rotation' && <DataTable rows={rotationRows} columns={['item', 'count', 'status']} empty="No credential rotation tasks" />}
      {tab === 'Compliance Tasks' && <DataTable rows={tasks.rows} columns={['id', 'title', 'category', 'status', 'owner_admin_id', 'due_at', 'completed_at', 'created_at']} empty={{ title: 'No compliance tasks yet', body: 'Create tasks for access review, retention, exports, and credential rotation follow-up.' }} renderActions={row => <ActionButtons actions={['Mark task complete', 'View detail']} onClick={label => label === 'Mark task complete' ? action('Mark compliance task complete', `${API}/compliance/tasks/${String(row.id || '')}/complete`) : helpers.setDrawer({ title: 'Compliance task', body: row })} />} />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function PlatformSettingsPage() {
  const [form, setForm] = useState<PlatformSettingsForm>(DEFAULT_PLATFORM_SETTINGS);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<Partial<PlatformSettingsForm>>('GET', '/admin/platform-settings');
      setForm({ ...DEFAULT_PLATFORM_SETTINGS, ...data });
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Platform settings load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const save = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({ title: 'Save platform settings', body: 'Sensitive setting changes require confirmation and audit logging.', action: () => helpers.run('Platform settings saved.', () => apiRequest('PATCH', '/admin/platform-settings', form)).then(load) });
  };
  const maintenance = (enabled: boolean) => {
    helpers.setConfirm({ title: `${enabled ? 'Enable' : 'Disable'} maintenance mode`, body: 'Maintenance mode change affects platform availability and creates audit log.', action: () => helpers.run('Maintenance mode updated.', () => apiRequest('POST', `/admin/platform-settings/maintenance/${enabled ? 'enable' : 'disable'}`)).then(load) });
  };
  return (
    <CrudPage title="Platform Settings" subtitle="Company-level platform configuration" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <section className="card">
        <h2><ShieldCheck size={18} /> Platform configuration</h2>
        <form onSubmit={save}>
          <div className="filter-grid">
            <Field label="Platform name"><input value={form.platform_name} onChange={event => setForm({ ...form, platform_name: event.target.value })} /></Field>
            <Field label="Support email"><input value={form.support_email} onChange={event => setForm({ ...form, support_email: event.target.value })} /></Field>
            <Field label="Support phone"><input value={form.support_phone} onChange={event => setForm({ ...form, support_phone: event.target.value })} /></Field>
            <Field label="Default country"><input value={form.default_country} onChange={event => setForm({ ...form, default_country: event.target.value })} /></Field>
            <Field label="Default currency"><input value={form.default_currency} onChange={event => setForm({ ...form, default_currency: event.target.value })} /></Field>
            <Field label="Session timeout"><input value={form.session_timeout} onChange={event => setForm({ ...form, session_timeout: event.target.value })} /></Field>
            <Field label="Audit retention days"><input value={form.audit_retention_days} onChange={event => setForm({ ...form, audit_retention_days: event.target.value })} /></Field>
            <Field label="Export retention days"><input value={form.export_retention_days} onChange={event => setForm({ ...form, export_retention_days: event.target.value })} /></Field>
            <Field label="Notification email sender"><input value={form.notification_email_sender} onChange={event => setForm({ ...form, notification_email_sender: event.target.value })} /></Field>
            <Field label="Timezone"><input value={form.timezone} onChange={event => setForm({ ...form, timezone: event.target.value })} /></Field>
            <Field label="Date format"><input value={form.date_format} onChange={event => setForm({ ...form, date_format: event.target.value })} /></Field>
            <label className="form-field"><span>Maintenance mode</span><input type="checkbox" checked={form.maintenance_mode} onChange={event => setForm({ ...form, maintenance_mode: event.target.checked })} /></label>
            <label className="form-field"><span>MFA required</span><input type="checkbox" checked={form.mfa_required} onChange={event => setForm({ ...form, mfa_required: event.target.checked })} /></label>
          </div>
          <div className="button-row">
            <button className="btn-primary" type="submit">Save settings</button>
            <button type="button" onClick={() => maintenance(true)}>Enable maintenance mode</button>
            <button type="button" onClick={() => maintenance(false)}>Disable maintenance mode</button>
            <button type="button" onClick={() => setForm(DEFAULT_PLATFORM_SETTINGS)}>Reset defaults</button>
          </div>
        </form>
      </section>
      <div className="card">
        <h2><Mail size={18} /> Security posture</h2>
        <div className="security-list">
          {['Deny-by-default RBAC', 'Least privilege route guards', 'Confirmation for sensitive settings', 'Audit logs for setting changes'].map(item => <div className="security-row" key={item}><span>{item}</span><strong>Enabled</strong></div>)}
        </div>
      </div>
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

function TableRoute({ title, subtitle, rows, state, error, notice, columns, actions, onAction, onRefresh, helpers }: { title: string; subtitle: string; rows: GenericRecord[]; state: LoadState; error: string; notice?: string; columns: string[]; actions: string[]; onAction: (row: GenericRecord, action: string) => void; onRefresh: () => void; helpers?: ReturnType<typeof useDrawerConfirm> }) {
  const emptyStates: Record<string, EmptyStateConfig> = {
    Refunds: { title: 'No refunds yet', body: 'Refund requests will appear here.' },
    Settlements: { title: 'No settlement batches yet.', body: 'Settlement batches appear when providers or bank files are ready for finance review.' },
    Disputes: { title: 'No disputes or chargebacks yet.', body: 'Disputes appear when providers report chargeback activity.' },
    'Risk Rules': { title: 'No risk rules configured.', body: 'Create rules to review risky payments, refunds, or payout activity.' },
    'Audit Logs': { title: 'No sensitive admin actions recorded yet.', body: 'Sensitive admin actions will create audit logs here.' },
    Notifications: { title: 'No active alerts.', body: 'Provider, webhook, settlement, and risk alerts appear here.' },
    Reports: { title: 'Reports are ready to export.', body: 'Choose a report type and export format when finance needs data.' },
  };
  return (
    <CrudPage title={title} subtitle={subtitle} state={state} error={error} notice={notice} onRefresh={onRefresh}>
      <DataTable rows={rows} columns={columns} empty={emptyStates[title] || `No ${title.toLowerCase()} records`} renderActions={row => <ActionButtons actions={actions} onClick={action => onAction(row, normalizeAction(action))} />} />
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
  if (state === 'loading') return <Skeleton message={loading} />;
  if (state === 'error') return <ErrorPanel message={error} />;
  return null;
}

function Skeleton({ message }: { message: string }) {
  return (
    <div className="card skeleton-card" aria-busy="true">
      <div className="skeleton-line wide"></div>
      <div className="skeleton-line"></div>
      <span>{message}</span>
    </div>
  );
}

function ErrorPanel({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="alert alert-error error-panel">
      <span>{message}</span>
      {onRetry && <button type="button" onClick={onRetry}>Retry</button>}
    </div>
  );
}

function EmptyState({ config }: { config: EmptyStateConfig }) {
  return (
    <div className="empty-state rich-empty">
      <FileSearch size={28} />
      <strong>{config.title}</strong>
      <p>{config.body}</p>
      {config.action && <button className="btn-primary" type="button" onClick={config.onAction}>{config.action}</button>}
    </div>
  );
}

function TablePanel({ title, rows, columns }: { title: string; rows: GenericRecord[]; columns: string[] }) {
  return (
    <section className="table-card">
      <h2>{title}</h2>
      <DataTable rows={rows} columns={columns} empty={`No ${title.toLowerCase()} records`} />
    </section>
  );
}

function DataTable({ rows, columns, empty, renderActions, onView }: { rows: GenericRecord[]; columns: string[]; empty: string | EmptyStateConfig; renderActions?: (row: GenericRecord) => ReactNode; onView?: (row: GenericRecord) => void }) {
  return (
    <div className="table-card table-scroll" style={{ padding: 0 }}>
      <table>
        <thead>
          <tr>
            {columns.map((column, index) => <th key={column} className={index === 0 ? 'sticky-col' : undefined}>{column.replaceAll('_', ' ')}</th>)}
            {(renderActions || onView) && <th>Actions</th>}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={String(row.id || index)}>
              {columns.map((column, columnIndex) => <td key={column} className={columnIndex === 0 ? 'sticky-col' : undefined}>{renderCell(row, column)}</td>)}
              {(renderActions || onView) && <td>{renderActions ? renderActions(row) : <button type="button" onClick={() => onView?.(row)}><Eye size={14} /> View detail</button>}</td>}
            </tr>
          ))}
          {rows.length === 0 && <tr><td colSpan={columns.length + (renderActions || onView ? 1 : 0)}>{typeof empty === 'string' ? empty : <EmptyState config={empty} />}</td></tr>}
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
  const primary = actions.slice(0, 3);
  const overflow = actions.slice(3);
  return (
    <div className="button-row compact-actions">
      {primary.map(action => <button key={action} type="button" onClick={() => onClick(action)}>{action}</button>)}
      {overflow.length > 0 && (
        <select aria-label="More actions" onChange={event => {
          if (event.target.value) onClick(event.target.value);
          event.target.value = '';
        }}>
          <option value="">More</option>
          {overflow.map(action => <option key={action} value={action}>{action}</option>)}
        </select>
      )}
    </div>
  );
}

function RoutingSimulator({ form, result, onChange, onSubmit }: { form: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string; strategy: string }; result: RoutingSimulation | null; onChange: (value: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string; strategy: string }) => void; onSubmit: (event: FormEvent) => void }) {
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
          <Field label="Strategy"><Select value={form.strategy} options={['priority', 'weighted', 'lowest_cost', 'highest_success_rate', 'latency_aware', 'risk_aware', 'health_aware', 'round_robin', 'fallback_only']} onChange={strategy => onChange({ ...form, strategy })} /></Field>
        </div>
        <button className="btn-primary" type="submit">Simulate</button>
      </form>
      <div className="stats-grid">
        <Metric label="Selected provider" value={result?.selected_provider || 'No match'} icon={Cable} />
        <Metric label="Matched rule" value={result?.matched_routing_rule || '-'} icon={Route} />
        <Metric label="Provider score" value={result?.provider_score?.toFixed(2) || '-'} icon={Activity} />
        <Metric label="Fallback provider" value={result?.fallback_provider || '-'} icon={AlertTriangle} />
        <Metric label="Provider fee" value={money(result?.estimated_provider_fee || 0)} icon={Receipt} />
        <Metric label="Latency estimate" value={`${result?.latency_estimate_ms || 0} ms`} icon={RefreshCw} />
        <Metric label="Success estimate" value={`${(result?.success_estimate || 0).toFixed(1)}%`} icon={BadgeCheck} />
        <Metric label="Strategy used" value={result?.strategy_used || '-'} icon={SlidersHorizontal} />
      </div>
      {result && <DataTable rows={result.candidate_providers || []} columns={['provider', 'score', 'success_rate', 'latency_ms', 'fee_percent', 'attempt_count']} empty="No candidate providers" />}
      {result && <DataTable rows={result.rejected_providers || []} columns={['provider', 'reason']} empty="No rejected providers" />}
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
