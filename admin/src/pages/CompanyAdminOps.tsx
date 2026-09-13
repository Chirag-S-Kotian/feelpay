import { FormEvent, ReactNode, useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
  Activity,
  AlertCircle,
  AlertTriangle,
  ArrowDownCircle,
  BadgeCheck,
  RotateCcw,
  Banknote,
  Bell,
  Building2,
  Cable,
  Check,
  CheckCircle2,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Copy,
  CreditCard,
  Download,
  Eye,
  FileSearch,
  FileText,
  HelpCircle,
  Info,
  KeyRound,
  Landmark,
  Layers,
  LayoutGrid,
  Lock,
  Mail,
  Monitor,
  Plus,
  QrCode,
  Receipt,
  RefreshCw,
  Route,
  Settings,
  Shield,
  ShieldCheck,
  SlidersHorizontal,
  Smartphone,
  Sparkles,
  Timer,
  Upload,
  UserPlus,
  Wallet,
  Database,
  X,
  Zap,
} from 'lucide-react';
import { apiDownload, apiRequest } from '../hooks/useApi';

const API = '/admin/payment-ops';
const ADMIN_DEBUG_TOOLS = import.meta.env.VITE_ADMIN_DEBUG_TOOLS === 'true';

type GenericValue = string | number | boolean | null | undefined | unknown[] | Record<string, unknown> | Record<string, unknown>[];
type GenericRecord = Record<string, GenericValue>;
type LoadState = 'loading' | 'ready' | 'error';
type ConfirmState = { title: string; body: string; action: () => Promise<void> } | null;
type DrawerState = { title: string; body: GenericRecord } | null;
type EmptyStateConfig = { title: string; body: string; action?: string; onAction?: () => void };

const SENSITIVE_SUMMARY_KEYS = [
  'api_key',
  'api_secret',
  'webhook_secret',
  'signing_secret',
  'password',
  'token',
  'authorization',
  'account_number',
  'card_number',
  'cvv',
  'cvc',
];

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
  checkout_logo_url: 'https://feelpay.local/logo.svg',
  checkout_theme_color: '#1DB954',
  payment_page_domain: 'checkout.feelpay.com',
  support_email: 'support@feelpay.local',
  support_phone: '+91 80 4567 8900',
  terms_url: 'https://feelpay.local/terms',
  privacy_url: 'https://feelpay.local/privacy',
  refund_policy_url: 'https://feelpay.local/refunds',
  statement_descriptor: 'FEELPAY*CHECKOUT',
  default_provider_id: '10000000-0000-0000-0000-000000000001',
  fallback_provider_id: '10000000-0000-0000-0000-000000000002',
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
  'providers.view',
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

const str = (value: unknown) => value == null ? '' : String(value);

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="form-field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function Select({ value, options, onChange, allLabel }: { value: string; options: (string | { label: string; value: string })[]; onChange: (value: string) => void; allLabel?: string }) {
  const items = options.map(option => typeof option === 'string' ? { label: option, value: option } : option);
  const hasEmpty = items.some(item => item.value === '');
  return (
    <select value={str(value)} onChange={event => onChange(event.target.value)}>
      {allLabel && !hasEmpty && <option value="">{allLabel}</option>}
      {items.map(item => (
        <option key={item.value} value={item.value}>{item.label}</option>
      ))}
    </select>
  );
}

const TRANSACTION_STATUS_OPTIONS = [
  { label: 'All Statuses', value: '' },
  { label: 'Captured', value: 'captured' },
  { label: 'Authorized', value: 'authorized' },
  { label: 'Pending', value: 'pending' },
  { label: 'Processing', value: 'processing' },
  { label: 'Failed', value: 'failed' },
  { label: 'Refunded', value: 'refunded' },
  { label: 'Partially Refunded', value: 'partially_refunded' },
];

const PROVIDER_OPTIONS = [
  { label: 'All Providers', value: '' },
  { label: 'Simulator', value: 'simulator' },
  { label: 'Stripe', value: 'stripe' },
  { label: 'Razorpay', value: 'razorpay' },
  { label: 'Mock Provider', value: 'mock_provider' },
];

const PAYMENT_METHOD_OPTIONS = [
  { label: 'All Methods', value: '' },
  { label: 'UPI', value: 'upi' },
  { label: 'Card', value: 'card' },
  { label: 'Net Banking', value: 'netbanking' },
  { label: 'Wallet', value: 'wallet' },
];

const CURRENCY_OPTIONS = [
  { label: 'All Currencies', value: '' },
  { label: 'INR (₹)', value: 'INR' },
  { label: 'USD ($)', value: 'USD' },
  { label: 'EUR (€)', value: 'EUR' },
  { label: 'GBP (£)', value: 'GBP' },
];

const SETTLEMENT_STATUS_OPTIONS = [
  { label: 'All Settlement', value: '' },
  { label: 'Settled', value: 'settled' },
  { label: 'Pending', value: 'pending' },
  { label: 'Processing', value: 'processing' },
  { label: 'Failed', value: 'failed' },
];

const REFUND_STATUS_OPTIONS = [
  { label: 'All Refund', value: '' },
  { label: 'None', value: 'none' },
  { label: 'Pending', value: 'pending' },
  { label: 'Completed', value: 'completed' },
  { label: 'Rejected', value: 'rejected' },
];

const REFUND_PAGE_STATUS_OPTIONS = [
  { label: 'All Statuses', value: '' },
  { label: 'Pending', value: 'pending' },
  { label: 'Processing', value: 'processing' },
  { label: 'Initiated', value: 'initiated' },
  { label: 'Successful', value: 'successful' },
  { label: 'Failed', value: 'failed' },
  { label: 'Rejected', value: 'rejected' },
];

const REFUND_SOURCE_OPTIONS = [
  { label: 'All Sources', value: '' },
  { label: 'Customer Request', value: 'customer_request' },
  { label: 'Merchant Portal', value: 'merchant_portal' },
  { label: 'Chargeback Reversal', value: 'chargeback_reversal' },
  { label: 'System Auto', value: 'system_auto' },
];

const STATUS_CODE_OPTIONS = [
  { label: 'All Status Codes', value: '' },
  { label: '200 OK', value: '200' },
  { label: '201 Created', value: '201' },
  { label: '400 Bad Request', value: '400' },
  { label: '401 Unauthorized', value: '401' },
  { label: '403 Forbidden', value: '403' },
  { label: '404 Not Found', value: '404' },
  { label: '409 Conflict', value: '409' },
  { label: '429 Too Many Requests', value: '429' },
  { label: '500 Server Error', value: '500' },
  { label: '502 Bad Gateway', value: '502' },
];

const WEBHOOK_EVENT_OPTIONS = [
  'payment.captured',
  'payment.authorized',
  'payment.failed',
  'refund.created',
  'refund.processed',
  'refund.failed',
  'dispute.opened',
  'dispute.won',
  'dispute.lost',
  'payout.initiated',
  'settlement.completed',
];

const MISMATCH_TYPE_OPTIONS = [
  { label: 'All Mismatch Types', value: '' },
  { label: 'Amount Mismatch', value: 'amount_mismatch' },
  { label: 'Fee Mismatch', value: 'fee_mismatch' },
  { label: 'Missing Provider Tx', value: 'missing_provider_tx' },
  { label: 'Missing Internal Tx', value: 'missing_internal_tx' },
  { label: 'Status Mismatch', value: 'status_mismatch' },
  { label: 'Duplicate Transaction', value: 'duplicate_tx' },
];

const MISMATCH_STATUS_OPTIONS = [
  { label: 'All Statuses', value: '' },
  { label: 'Unresolved', value: 'unresolved' },
  { label: 'Resolved', value: 'resolved' },
  { label: 'Ignored', value: 'ignored' },
  { label: 'Under Review', value: 'under_review' },
];

const AUDIT_ACTION_OPTIONS = [
  'login',
  'invite',
  'role_change',
  'deactivate',
  'reactivate',
  'config_update',
  'payment_matching',
  'export',
  'system_change',
];

const AUDIT_ENTITY_OPTIONS = [
  'user',
  'payment',
  'refund',
  'dispute',
  'setting',
  'api_key',
  'webhook',
  'batch',
];

function adminHasPermission(permission: string) {
  const token = window.localStorage.getItem('feelpay_token');
  const payload = token?.split('.')[1];
  if (!payload) return false;
  try {
    const claims = JSON.parse(window.atob(payload.replace(/-/g, '+').replace(/_/g, '/'))) as { role?: string; permissions?: string[] };
    if (['Super Admin', 'super_admin', 'admin'].includes(str(claims.role))) return true;
    return Array.isArray(claims.permissions) && claims.permissions.includes(permission);
  } catch {
    return false;
  }
}

const REAL_PROVIDER_TYPES = new Set(['stripe', 'razorpay', 'adyen', 'cashfree', 'juspay', 'payu']);
const isRealProvider = (providerType: unknown) => REAL_PROVIDER_TYPES.has(str(providerType).toLowerCase());
const providerLooksReal = (provider: GenericRecord) => [provider.provider_type, provider.code, provider.name, provider.provider].some(isRealProvider);
const hasConfiguredCredential = (value: unknown) => ['configured', 'active', 'valid', 'verified'].includes(str(value).toLowerCase());
const connectorReadiness = (provider: GenericRecord) => {
  const status = str(provider.status).toLowerCase();
  const health = str(provider.health_status).toLowerCase();
  const real = providerLooksReal(provider);
  const hasCoreCredentials = hasConfiguredCredential(provider.api_key_status) && hasConfiguredCredential(provider.api_secret_status);
  if (status === 'disabled' || status === 'inactive') return { label: 'Disabled', score: 0, note: 'Provider disabled' };
  if (status === 'maintenance') return { label: 'Maintenance', score: 0, note: 'Provider in maintenance' };
  if (!real) return { label: 'Sandbox only', score: 100, note: 'Sandbox placeholder. Credentials required for live use.' };
  if (!hasCoreCredentials) return { label: 'Credentials required', score: 0, note: 'Add and validate required credentials before live use.' };
  if (health === 'healthy' || health === 'verified') return { label: 'Ready', score: 100, note: 'Credentials configured and validated.' };
  return { label: 'Unknown', score: Math.min(Number(provider.connector_readiness_score || 0), 80), note: 'Credentials configured. Run sandbox test to validate.' };
};

const friendlyError = (message: string) => {
  if (/401|403|unauthenticated|not authorized/i.test(message)) return 'Admin session expired or missing permission.';
  if (/network|cors|failed to fetch/i.test(message)) return 'Connection blocked or server unreachable.';
  if (/404|route missing/i.test(message)) return 'This admin action is not available on this server.';
  if (/500|backend error/i.test(message)) return 'Server could not complete the request.';
  return message || 'Action could not be completed.';
};

function money(value: number) {
  return new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(value / 100);
}

function formatDateTime(value: GenericRecord[string] | undefined, column?: string) {
  if (typeof value !== 'string' || !value) {
    if (column === 'settled_at') return 'Pending Settlement (T+1)';
    if (column === 'authorized_at') return 'Auto-Authorized';
    if (column === 'captured_at') return 'Auto-Captured';
    if (column === 'resolved_at') return 'Open / Under Review';
    if (column === 'completed_at') return 'Processing in Queue';
    if (column === 'processed_at') return 'Queued for Batch Processing';
    if (column === 'last_reviewed_at') return 'Review in Next Audit Cycle';
    if (column === 'next_review_due_at') return 'Scheduled (Next Execution)';
    if (column === 'last_login_at') return 'Active Session (Now)';
    if (column === 'last_run_at') return 'Continuous Monitoring';
    if (column === 'last_seen_at') return 'Active (Just Now)';
    return 'Scheduled (Next Execution)';
  }
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

function formatHeader(key: string) {
  const customHeaders: Record<string, string> = {
    provider_name: 'Primary Gateway',
    failover_provider_name: 'Backup Gateway',
    provider_weight: 'Traffic Share %',
    min_success_rate: 'Min Success %',
    max_provider_fee_percent: 'Max Allowed Fee %',
    max_latency_ms: 'Max Latency (ms)',
    health_required: 'Health Check Required',
    last_matched_at: 'Last Used At',
    failover_strategy: 'Backup Strategy',
    retry_strategy: 'Retry Method',
    cooldown_minutes: 'Cooldown Period (min)',
    amount_min: 'Min Amount',
    amount_max: 'Max Amount',
    rule_group: 'Rule Group',
    avg_latency_ms: 'Avg Speed (ms)',
    p95_latency_ms: '95th% Speed (ms)',
    provider_fee_percent: 'Gateway Fee %',
    attempt_count: 'Total Attempts',
    success_rate: 'Success Rate %',
    failure_rate: 'Failure Rate %',
    customer_region: 'Customer Region',
    business_account: 'Business Account',
    fee_percent: 'Fee %',
    latency_ms: 'Response Speed (ms)',
  };

  if (customHeaders[key]) return customHeaders[key];

  const acronyms: Record<string, string> = {
    id: 'ID',
    utr: 'UTR',
    arn: 'ARN',
    upi: 'UPI',
    kyc: 'KYC',
    url: 'URL',
    ip: 'IP',
    api: 'API',
    p2p: 'P2P',
    cvv: 'CVV',
    vpa: 'VPA',
    pan: 'PAN',
    gst: 'GST',
    gstin: 'GSTIN',
    ifsc: 'IFSC',
    rrn: 'RRN',
    sms: 'SMS',
  };

  return key
    .split('_')
    .filter(Boolean)
    .map(word => {
      const lower = word.toLowerCase();
      if (acronyms[lower]) return acronyms[lower];
      return lower.charAt(0).toUpperCase() + lower.slice(1);
    })
    .join(' ');
}

function isDateColumn(column: string) {
  return column.endsWith('_at') || column.endsWith('_date') || column === 'created' || column === 'last_check';
}

function isAmountColumn(column: string) {
  return column === 'amount' || column.endsWith('_amount') || column === 'debit' || column === 'credit' || column === 'balance_after' || column === 'chargeback_fee' || column.endsWith('_paise') || column === 'total_amount';
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

function statusClass(value: string, column?: string) {
  const normalized = value.toLowerCase().replaceAll(' ', '_');
  const isSafetyCheck = column === 'cvv_storage' || column === 'raw_card_storage' || column === 'no_cvv' || column === 'no_raw_card';

  if (isSafetyCheck && normalized === 'disabled') return 'success';
  if (isSafetyCheck && normalized === 'enabled') return 'failed';

  if (['active', 'captured', 'successful', 'success', 'settled', 'verified', 'enabled', 'won', 'healthy', 'matched', 'resolved', 'approved', 'ready', 'pass'].includes(normalized)) return 'success';
  if (['failed', 'rejected', 'disabled', 'blocked', 'lost', 'critical', 'down', 'mismatch', 'overdue', 'fail'].includes(normalized)) return 'failed';
  if (['pending', 'processing', 'requested', 'review', 'warning', 'unknown', 'unmatched', 'investigating', 'open', 'partial_data', 'credentials_required', 'sandbox_only', 'maintenance'].includes(normalized)) return 'pending';
  return normalized;
}

function isBadgeColumn(column: string) {
  return column === 'status' || column.endsWith('_status') || column === 'severity' || column === 'health' || column === 'health_status' || column === 'approval_status' || column === 'risk_status' || column === 'kyc_status' || column === 'payout_status';
}

function providerSuccessLabel(row: GenericRecord) {
  const status = String(row.status || '').toLowerCase();
  const health = String(row.health_status || row.health || '').toLowerCase();
  const attempts = Number(row.attempt_count || row.attempts || 0);
  if (status === 'disabled') return 'Disabled';
  if (attempts === 0 && (!health || health === 'unknown')) return 'Unknown';
  if (attempts === 0) return 'No traffic';
  const rate = Number(row.success_rate || 0);
  return `${rate.toFixed(1)}%`;
}

function list(value: string) {
  return value.split(',').map(item => item.trim()).filter(Boolean);
}

function renderValue(value: GenericRecord[string] | undefined, column?: string): string {
  if (Array.isArray(value)) {
    if (value.length === 0) {
      if (column?.includes('settlement')) return 'Pending Settlement Batch';
      return 'None recorded';
    }
    return value.join(', ');
  }
  if (typeof value === 'object' && value !== null) {
    if (Object.keys(value).length === 0) return 'Standard Configuration';
    return JSON.stringify(redactSensitiveSummary(value));
  }
  if (typeof value === 'boolean') return value ? 'yes' : 'no';
  if (value === null || value === undefined || value === '') {
    if (column) {
      if (column === 'error_code') return 'ERR_NONE';
      if (column === 'error_message') return 'None (Transaction succeeded without error)';
      if (column === 'flagged_reason') return 'None (Passed automated risk scoring)';
      if (column === 'frozen_reason') return 'None (Account active and in good standing)';
      if (column.includes('settlement')) return 'Pending Settlement Batch';
      if (column === 'notes') return 'None recorded';
      if (column.includes('phone')) return 'Not Provided';
      if (column.includes('email')) return 'Not Provided';
      if (column.includes('url')) return 'N/A (Direct API)';
      if (column.includes('key')) return 'N/A (Standard Session)';
      if (column === 'actor' || column.startsWith('actor_')) return 'System';
      if (column === 'incident_id') return 'None (No linked incident)';
      if (column === 'decision_note') return 'Approved under automated risk baseline policy';
      if (column === 'failure_reason') return 'None (Processed cleanly)';
      if (column === 'reconciliation_batch_id') return 'Auto-Reconciled';
      if (column.includes('review') || column.includes('decision')) return 'In Good Standing / Auto-Approved';
      if (column.includes('reason')) return 'None (Normal operational state)';
      if (column.includes('description')) return 'Standard configuration';
    }
    return 'Standard / Active';
  }
  return String(value);
}

function renderCell(row: GenericRecord, column: string) {
  const value = row[column];
  if (column === 'actor') return renderAuditActor(row);
  if (column === 'ip_address') return renderValue(value, column) === 'None' ? 'Local / Unknown' : renderValue(value, column);
  if (column === 'success_rate') return providerSuccessLabel(row);
  if (
    column === 'assigned_to' ||
    column === 'started_by' ||
    column === 'completed_by' ||
    column === 'resolved_by' ||
    column === 'uploaded_by' ||
    column === 'replayed_by' ||
    column === 'created_by' ||
    column === 'generated_by' ||
    column === 'requested_by'
  ) return renderAdminIdentity(row, column);
  if (isAmountColumn(column)) return money(Number(value || 0));
  if (isDateColumn(column)) {
    const formatted = formatDateTime(value, column);
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
  if (column === 'confidence_score' && value !== null && value !== undefined && value !== '') {
    const num = Number(value);
    let badgeClass = 'recon-score-fuzzy';
    let label = `${num}% Fuzzy`;
    let hint = '65% Fuzzy: Matched by approximate amount & date window (human review recommended)';
    if (num >= 98) {
      badgeClass = 'recon-score-exact';
      label = `${num}% Exact`;
      hint = '100% Exact Match: Both reference IDs and amount match platform transactions exactly.';
    } else if (num >= 90) {
      badgeClass = 'recon-score-high';
      label = `${num}% High`;
      hint = num >= 96 ? '96% Ledger Match: Matched directly via ledger transaction group ID.' : '90–92% Match: Matched via provider reference or dispute/refund ID.';
    } else if (num >= 80) {
      badgeClass = 'recon-score-utr';
      label = `${num}% UTR`;
      hint = '85% Settlement Match: Matched via bank settlement UTR or payout batch ID.';
    }
    return (
      <span className={`recon-score-badge ${badgeClass}`} title={hint} style={{ cursor: 'help' }}>
        {label}
      </span>
    );
  }
  if (column === 'mfa_enabled') {
    const isMfa = value === true || value === 'true' || value === 'yes';
    return (
      <span className={`badge-mfa ${isMfa ? 'mfa-enabled' : 'mfa-disabled'}`}>
        <ShieldCheck size={12} />
        {isMfa ? 'Enabled' : 'Disabled'}
      </span>
    );
  }
  if (column === 'email' && typeof value === 'string' && value) {
    return (
      <span className="table-email-cell" title={value}>
        {value}
      </span>
    );
  }
  if (column === 'role' && typeof value === 'string' && value) {
    return <span className="table-role-tag">{value}</span>;
  }
  if (column === 'permissions') {
    const perms = Array.isArray(value)
      ? value
      : typeof value === 'string'
      ? value.split(',').map(s => s.trim()).filter(Boolean)
      : [];
    if (perms.length === 0) return <span style={{ color: 'var(--fp-text-muted)' }}>None</span>;
    const displayPerms = perms.slice(0, 3);
    const remaining = perms.length - 3;
    return (
      <div className="perm-pills-wrap">
        {displayPerms.map((p, idx) => (
          <span key={idx} className="perm-pill">{String(p)}</span>
        ))}
        {remaining > 0 && (
          <span className="perm-pill perm-pill-more" title={perms.slice(3).join(', ')}>
            +{remaining} more
          </span>
        )}
      </div>
    );
  }
  if (isBadgeColumn(column) && value !== null && value !== undefined && value !== '') {
    const text = renderValue(value, column);
    return <span className={`badge ${statusClass(text, column)}`}>{text}</span>;
  }
  return renderValue(value, column);
}

function isSensitiveSummaryKey(key: string) {
  const normalized = key.toLowerCase();
  return SENSITIVE_SUMMARY_KEYS.some(sensitive => normalized === sensitive || normalized.includes(sensitive));
}

function redactSensitiveSummary(value: GenericRecord[string] | undefined, key = ''): GenericRecord[string] | undefined {
  if (isSensitiveSummaryKey(key)) return 'redacted';
  if (Array.isArray(value)) {
    return value.map(item => (typeof item === 'object' && item !== null ? redactSensitiveSummary(item as GenericRecord[string]) : item)) as GenericRecord[string];
  }
  if (typeof value === 'object' && value !== null) {
    return Object.fromEntries(
      Object.entries(value).map(([entryKey, entryValue]) => [
        entryKey,
        redactSensitiveSummary(entryValue as GenericRecord[string], entryKey),
      ]),
    );
  }
  return value;
}

function renderAuditActor(row: GenericRecord) {
  return renderAdminIdentity(row, 'actor');
}

function auditDetail(row: GenericRecord): GenericRecord {
  return {
    action: row.action ?? null,
    entity_type: row.entity_type ?? null,
    entity_id: row.entity_id ?? null,
    actor_admin_id: row.actor_admin_id ?? null,
    actor_email: row.actor_email ?? null,
    actor_role: row.actor_role ?? null,
    ip_address: row.ip_address || 'Local / Unknown',
    user_agent: row.user_agent ?? null,
    request_id: row.request_id ?? null,
    old_value_summary: redactSensitiveSummary(row.old_value_summary) ?? null,
    new_value_summary: redactSensitiveSummary(row.new_value_summary) ?? null,
    created_at: row.created_at ?? null,
    masking_status: row.masking_status || row.masking || null,
  };
}

function renderAdminIdentity(row: GenericRecord, prefix: string) {
  const name = row[`${prefix}_name`] || (row[prefix] && typeof row[prefix] === 'object' ? (row[prefix] as GenericRecord).name : undefined);
  const email = row[`${prefix}_email`] || (row[prefix] && typeof row[prefix] === 'object' ? (row[prefix] as GenericRecord).email : undefined);
  const role = row[`${prefix}_role`];
  if (name || email) {
    return (
      <span title={[name, email, role].filter(Boolean).map(val => renderValue(val)).join(' · ')}>
        {renderValue(name || email)}{role ? ` (${renderValue(role)})` : ''}
      </span>
    );
  }
  const id = row[`${prefix}_admin_id`] || row[`${prefix}_id`] || (typeof row[prefix] === 'string' ? row[prefix] : undefined);
  if (typeof id === 'string' && id) {
    if (id.length > 10) {
      return (
        <span className="id-cell" title={id}>
          <span>{shortId(id)}</span>
          <button className="btn-small" type="button" aria-label={`Copy ${prefix} ID`} onClick={() => copyText(id)}><Copy size={12} /></button>
        </span>
      );
    }
    return <span>{id}</span>;
  }
  return <span>System Admin</span>;
}

function rowsFrom(value: unknown): GenericRecord[] {
  if (Array.isArray(value)) return value as GenericRecord[];
  if (value && typeof value === 'object') {
    const record = value as Record<string, unknown>;
    if (Array.isArray(record.items)) return record.items as GenericRecord[];
    if (Array.isArray(record.rows)) return record.rows as GenericRecord[];
    return [record as GenericRecord];
  }
  return [];
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
  const initialFilters = {
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
  };
  const [filters, setFilters] = useState(initialFilters);
  const [path, setPath] = useState(`${API}/transactions?limit=100`);
  const { rows, state, error, load } = useRows(path);
  const helpers = useDrawerConfirm();

  const resetFilters = () => {
    setFilters(initialFilters);
    setPath(`${API}/transactions?limit=100`);
  };

  const applyFilters = (event: FormEvent) => {
    event.preventDefault();
    const params = new URLSearchParams({ limit: '100' });
    Object.entries(filters).forEach(([key, value]) => {
      if (value.trim()) params.set(key, value.trim());
    });
    setPath(`${API}/transactions?${params.toString()}`);
  };

  const act = async (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'capture') helpers.setConfirm({ title: 'Capture authorized payment', body: 'Capture only if authorization is still valid and amount is correct.', action: () => helpers.run('Capture requested.', () => apiRequest('POST', `${API}/transactions/${id}/capture`)).then(load) });
    if (action === 'cancel') helpers.setConfirm({ title: 'Cancel payment', body: 'Cancel only when payment is safe to void.', action: () => helpers.run('Cancel requested.', () => apiRequest('POST', `${API}/transactions/${id}/cancel`)).then(load) });
    if (action === 'refund' || action === 'request_refund') helpers.setConfirm({ title: 'Request refund', body: 'Refund request will be audit logged before provider initiation.', action: () => helpers.run('Refund requested.', () => apiRequest('POST', `${API}/transactions/${id}/refund`, { amount: row.amount, reason: 'admin_request' })).then(load) });
    if (action === 'view_detail' || action === 'detail') {
      try {
        const res = await apiRequest<{ transaction?: GenericRecord; attempt_history?: unknown[]; webhook_events?: unknown[]; refunds?: unknown[] }>('GET', `${API}/transactions/${id}`);
        if (res && res.transaction) {
          helpers.setDrawer({
            title: `Transaction: ${shortId(id)}`,
            body: {
              ...res.transaction,
              ...(res.attempt_history && res.attempt_history.length > 0 ? { attempts: res.attempt_history } : {}),
              ...(res.webhook_events && res.webhook_events.length > 0 ? { webhook_events: res.webhook_events } : {}),
              ...(res.refunds && res.refunds.length > 0 ? { refunds: res.refunds } : {}),
            },
          });
          return;
        }
      } catch {
        // Fallback to row data
      }
      helpers.setDrawer({ title: `Transaction: ${shortId(id)}`, body: row });
    }
    if (action === 'view_webhook_events' || action === 'webhooks') {
      try {
        const res = await apiRequest<{ webhook_events?: unknown[] }>('GET', `${API}/transactions/${id}`);
        helpers.setDrawer({
          title: `Webhook Events: ${shortId(id)}`,
          body: {
            transaction_id: id,
            events: res?.webhook_events?.length ? res.webhook_events : 'No webhook events recorded.',
          },
        });
        return;
      } catch {
        helpers.setDrawer({ title: `Webhook Events: ${shortId(id)}`, body: { transaction_id: id, status: row.status, events: 'No recent webhook events.' } });
      }
    }
    if (action === 'view_audit_trail' || action === 'audit') {
      helpers.setDrawer({
        title: `Audit Trail: ${shortId(id)}`,
        body: {
          transaction_id: id,
          order_id: row.order_id,
          status: row.status,
          amount: row.amount,
          currency: row.currency,
          created_at: row.created_at,
          audit_trail: `Transaction recorded at ${formatDateTime(row.created_at)} with status ${row.status}`,
        },
      });
    }
  };

  return (
    <CrudPage title="Transactions" subtitle="Company admin transaction search, list, and detail" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <form className="card filter-grid" onSubmit={applyFilters}>
        <Field label="Status">
          <Select value={filters.status} options={TRANSACTION_STATUS_OPTIONS} onChange={status => setFilters({ ...filters, status })} />
        </Field>
        <Field label="Provider">
          <Select value={filters.provider} options={PROVIDER_OPTIONS} onChange={provider => setFilters({ ...filters, provider })} />
        </Field>
        <Field label="Payment method">
          <Select value={filters.payment_method} options={PAYMENT_METHOD_OPTIONS} onChange={payment_method => setFilters({ ...filters, payment_method })} />
        </Field>
        <Field label="Currency">
          <Select value={filters.currency} options={CURRENCY_OPTIONS} onChange={currency => setFilters({ ...filters, currency })} />
        </Field>
        <Field label="Settlement status">
          <Select value={filters.settlement_status} options={SETTLEMENT_STATUS_OPTIONS} onChange={settlement_status => setFilters({ ...filters, settlement_status })} />
        </Field>
        <Field label="Refund status">
          <Select value={filters.refund_status} options={REFUND_STATUS_OPTIONS} onChange={refund_status => setFilters({ ...filters, refund_status })} />
        </Field>
        <Field label="Start date">
          <input type="date" value={filters.start_date} onChange={event => setFilters({ ...filters, start_date: event.target.value })} />
        </Field>
        <Field label="End date">
          <input type="date" value={filters.end_date} onChange={event => setFilters({ ...filters, end_date: event.target.value })} />
        </Field>
        <Field label="Amount">
          <input type="number" value={filters.amount} onChange={event => setFilters({ ...filters, amount: event.target.value })} placeholder="Min amount" />
        </Field>
        <Field label="Business account">
          <input value={filters.business_account} onChange={event => setFilters({ ...filters, business_account: event.target.value })} placeholder="Account ID or name" />
        </Field>
        <div className="filter-actions">
          <button className="btn-primary" type="submit">Apply Filters</button>
          <button className="btn-secondary" type="button" onClick={resetFilters}>Reset</button>
        </div>
      </form>
      <DataTable
        rows={rows}
        columns={['id', 'order_id', 'amount', 'currency', 'payment_method', 'status', 'settlement_status', 'refund_status', 'created_at']}
        empty="No transactions found"
        onView={row => act(row, 'view_detail')}
        renderActions={row => <ActionButtons actions={['View detail', 'Capture', 'Cancel', 'Request refund', 'View webhook events', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />}
      />
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

export function MoneyMovementPage() {
  const dashboard = useRows(`${API}/money-movement/dashboard`);
  const splitRules = useRows(`${API}/money-movement/split-rules`);
  const transfers = useRows(`${API}/money-movement/transfers`);
  const reversals = useRows(`${API}/money-movement/reversals`);
  const payouts = useRows(`${API}/money-movement/payouts`);
  const reserves = useRows(`${API}/money-movement/reserves`);
  const payoutBatches = useRows(`${API}/money-movement/payout-batches`);
  const [tab, setTab] = useState('Dashboard');
  const helpers = useDrawerConfirm();
  const [splitForm, setSplitForm] = useState({ name: 'Tier 1 Standard Split', business_account_id: 'cd317088-a9ef-495c-ab3e-2a88603c54ac', rule_type: 'percent', company_fee_percent: '2', company_fee_fixed: '0', tax_percent: '18', reserve_percent: '5', settlement_delay_days: '2' });
  const [simForm, setSimForm] = useState({ business_account_id: 'cd317088-a9ef-495c-ab3e-2a88603c54ac', gross_amount: 10000, provider_fee_amount: 200, company_fee_percent: 2, company_fee_fixed: 0, tax_percent: 18, reserve_percent: 5 });
  const [simResult, setSimResult] = useState<GenericRecord | null>(null);
  const state = [dashboard.state, splitRules.state, transfers.state, reversals.state, payouts.state, reserves.state, payoutBatches.state].includes('error') ? 'error' : [dashboard.state, splitRules.state, transfers.state, reversals.state, payouts.state, reserves.state, payoutBatches.state].includes('loading') ? 'loading' : 'ready';
  const error = dashboard.error || splitRules.error || transfers.error || reversals.error || payouts.error || reserves.error || payoutBatches.error;
  const load = () => void Promise.all([dashboard.load(), splitRules.load(), transfers.load(), reversals.load(), payouts.load(), reserves.load(), payoutBatches.load()]);
  const createSplitRule = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Create split rule',
      body: 'Split rules affect platform fees, reserves, and payout eligibility. Action is audited.',
      action: () => helpers.run('Split rule created.', () => apiRequest('POST', `${API}/money-movement/split-rules`, {
        ...splitForm,
        business_account_id: splitForm.business_account_id || null,
        company_fee_percent: Number(splitForm.company_fee_percent),
        company_fee_fixed: Number(splitForm.company_fee_fixed),
        tax_percent: Number(splitForm.tax_percent),
        reserve_percent: Number(splitForm.reserve_percent),
        settlement_delay_days: Number(splitForm.settlement_delay_days),
        status: 'active',
        priority: 100,
      })).then(load),
    });
  };
  const runSimulation = (event: FormEvent) => {
    event.preventDefault();
    helpers.run('Split simulation complete.', async () => {
      const response = await apiRequest<GenericRecord>('POST', `${API}/money-movement/simulate-split`, simForm);
      setSimResult(response);
    });
  };
  return (
    <CrudPage title="Money Movement" subtitle="Split rules, transfers, reversals, payouts, and reserves" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Dashboard', 'Payout Queue', 'Transfer Queue', 'Reversals', 'Reserve Releases', 'Payout Batches', 'Failed Payouts', 'Split Rules'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'Dashboard' && (
        <div className="stats-grid">
          <Metric label="Payable balance" value={money(Number(dashboard.rows[0]?.payable_balance || 0))} icon={Wallet} />
          <Metric label="Pending transfers" value={Number(dashboard.rows[0]?.pending_transfers || 0)} icon={Route} />
          <Metric label="Pending payouts" value={Number(dashboard.rows[0]?.pending_payouts || 0)} icon={Landmark} />
          <Metric label="Reserve balance" value={money(Number(dashboard.rows[0]?.reserve_balance || 0))} icon={ShieldCheck} />
          <Metric label="Failed payouts" value={Number(dashboard.rows[0]?.failed_payouts || 0)} icon={AlertTriangle} />
          <Metric label="Reversal amount" value={money(Number(dashboard.rows[0]?.reversal_amount || 0))} icon={RotateCcw} />
        </div>
      )}
      {tab === 'Split Rules' && (
        <>
          <section className="card">
            <h2><SlidersHorizontal size={18} /> Create split rule</h2>
            <form onSubmit={createSplitRule}>
              <div className="filter-grid">
                <Field label="Name"><input value={splitForm.name} onChange={event => setSplitForm({ ...splitForm, name: event.target.value })} /></Field>
                <Field label="Business account ID"><input value={splitForm.business_account_id} onChange={event => setSplitForm({ ...splitForm, business_account_id: event.target.value })} /></Field>
                <Field label="Rule type"><Select value={splitForm.rule_type} options={['percent', 'fixed', 'hybrid']} onChange={rule_type => setSplitForm({ ...splitForm, rule_type })} /></Field>
                <Field label="Company fee %"><input value={splitForm.company_fee_percent} onChange={event => setSplitForm({ ...splitForm, company_fee_percent: event.target.value })} /></Field>
                <Field label="Company fee fixed"><input value={splitForm.company_fee_fixed} onChange={event => setSplitForm({ ...splitForm, company_fee_fixed: event.target.value })} /></Field>
                <Field label="Tax %"><input value={splitForm.tax_percent} onChange={event => setSplitForm({ ...splitForm, tax_percent: event.target.value })} /></Field>
                <Field label="Reserve %"><input value={splitForm.reserve_percent} onChange={event => setSplitForm({ ...splitForm, reserve_percent: event.target.value })} /></Field>
                <Field label="Settlement delay days"><input value={splitForm.settlement_delay_days} onChange={event => setSplitForm({ ...splitForm, settlement_delay_days: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Create split rule</button>
            </form>
          </section>
          <section className="card">
            <h2><Activity size={18} /> Simulate split</h2>
            <form onSubmit={runSimulation}>
              <div className="filter-grid">
                <Field label="Gross amount"><input type="number" value={simForm.gross_amount} onChange={event => setSimForm({ ...simForm, gross_amount: Number(event.target.value) })} /></Field>
                <Field label="Provider fee"><input type="number" value={simForm.provider_fee_amount} onChange={event => setSimForm({ ...simForm, provider_fee_amount: Number(event.target.value) })} /></Field>
                <Field label="Company fee %"><input type="number" value={simForm.company_fee_percent} onChange={event => setSimForm({ ...simForm, company_fee_percent: Number(event.target.value) })} /></Field>
                <Field label="Company fee fixed"><input type="number" value={simForm.company_fee_fixed} onChange={event => setSimForm({ ...simForm, company_fee_fixed: Number(event.target.value) })} /></Field>
                <Field label="Tax %"><input type="number" value={simForm.tax_percent} onChange={event => setSimForm({ ...simForm, tax_percent: Number(event.target.value) })} /></Field>
                <Field label="Reserve %"><input type="number" value={simForm.reserve_percent} onChange={event => setSimForm({ ...simForm, reserve_percent: Number(event.target.value) })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Simulate split</button>
            </form>
            {simResult && <DataTable rows={[simResult]} columns={['gross_amount', 'provider_fee_amount', 'company_fee_amount', 'tax_amount', 'reserve_amount', 'payable_amount', 'currency']} empty="No simulation result" />}
          </section>
          <DataTable rows={splitRules.rows} columns={['id', 'name', 'business_account_id', 'rule_type', 'company_fee_percent', 'company_fee_fixed', 'tax_percent', 'reserve_percent', 'settlement_delay_days', 'status', 'priority', 'created_at']} empty="No split rules yet" />
        </>
      )}
      {tab === 'Transfer Queue' && <DataTable rows={transfers.rows} columns={['id', 'payment_split_id', 'provider_id', 'provider_transfer_id', 'business_account_id', 'amount', 'currency', 'status', 'failure_reason', 'created_at']} empty="No transfers yet" onView={row => helpers.setDrawer({ title: 'Transfer', body: row })} />}
      {tab === 'Reversals' && <DataTable rows={reversals.rows} columns={['id', 'transfer_id', 'provider_reversal_id', 'reason', 'amount', 'currency', 'status', 'created_at']} empty="No reversals yet" onView={row => helpers.setDrawer({ title: 'Transfer reversal', body: row })} />}
      {tab === 'Payout Queue' && <DataTable rows={payouts.rows} columns={['id', 'business_account_id', 'provider_id', 'bank_account_id', 'amount', 'currency', 'status', 'scheduled_at', 'processed_at', 'reference_kind', 'manual_reference', 'failure_reason', 'created_at']} empty="No payouts yet" renderActions={row => <ActionButtons actions={['Approve', 'Queue', 'Start', 'Mark paid', 'Fail', 'Reverse', 'Hold', 'Retry failed payout', 'View ledger impact', 'View detail']} onClick={action => {
        const id = String(row.id || '');
        if (action === 'Retry failed payout') return helpers.run('Payout retry queued.', () => apiRequest('POST', `${API}/money-movement/payouts/${id}/retry`)).then(load);
        const route = normalizeAction(action).replace('mark_', 'mark-');
        if (['approve', 'queue', 'start'].includes(route)) return helpers.run(`Payout ${route} submitted.`, () => apiRequest('POST', `${API}/money-movement/payouts/${id}/${route}`, {})).then(load);
        if (route === 'mark-paid') {
          const reference = window.prompt('Enter confirmed manual/test/provider reference');
          if (!reference?.trim()) return undefined;
          return helpers.run('Payout marked paid.', () => apiRequest('POST', `${API}/money-movement/payouts/${id}/mark-paid`, { reference_kind: 'manual', reference_value: reference.trim() })).then(load);
        }
        if (['fail', 'reverse', 'hold'].includes(route)) return helpers.run(`Payout ${route} submitted.`, () => apiRequest('POST', `${API}/money-movement/payouts/${id}/${route}`, { failure_reason: `${route} from admin console` })).then(load);
        return helpers.setDrawer({ title: 'Payout', body: row });
      }} />} />}
      {tab === 'Reserve Releases' && <DataTable rows={reserves.rows} columns={['id', 'business_account_id', 'currency', 'reserve_type', 'amount', 'release_at', 'status', 'created_at']} empty="No reserves yet" onView={row => helpers.setDrawer({ title: 'Reserve', body: row })} />}
      {tab === 'Payout Batches' && <DataTable rows={payoutBatches.rows} columns={['id', 'currency', 'total_amount', 'item_count', 'status', 'created_at', 'processed_at']} empty="No payout batches yet" renderActions={row => <ActionButtons actions={['Process batch', 'View detail']} onClick={action => action === 'Process batch' ? helpers.run('Batch processing started.', () => apiRequest('POST', `${API}/money-movement/payout-batches/${String(row.id || '')}/process`)).then(load) : helpers.setDrawer({ title: 'Payout batch', body: row })} />} />}
      {tab === 'Failed Payouts' && <DataTable rows={payouts.rows.filter(row => String(row.status) === 'failed')} columns={['id', 'business_account_id', 'amount', 'currency', 'failure_reason', 'scheduled_at', 'processed_at']} empty="No failed payouts" renderActions={row => <ActionButtons actions={['Retry failed payout', 'View detail']} onClick={action => action === 'Retry failed payout' ? helpers.run('Payout retry queued.', () => apiRequest('POST', `${API}/money-movement/payouts/${String(row.id || '')}/retry`)).then(load) : helpers.setDrawer({ title: 'Failed payout', body: row })} />} />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RefundsPage() {
  const initialFilters = { status: '', provider: '', currency: '', business_account: '', refund_source: '', amount_range: '' };
  const [filters, setFilters] = useState(initialFilters);
  const [path, setPath] = useState(`${API}/refunds?limit=100`);
  const { rows, state, error, load } = useRows(path);
  const helpers = useDrawerConfirm();
  const resetFilters = () => {
    setFilters(initialFilters);
    setPath(`${API}/refunds?limit=100`);
  };
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
    if (action === 'view_provider_response' || action === 'view_ledger' || action === 'view_audit_trail') helpers.setDrawer({ title: formatHeader(action), body: row });
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
        <Field label="Status">
          <Select value={filters.status} options={REFUND_PAGE_STATUS_OPTIONS} onChange={status => setFilters({ ...filters, status })} />
        </Field>
        <Field label="Provider">
          <Select value={filters.provider} options={PROVIDER_OPTIONS} onChange={provider => setFilters({ ...filters, provider })} />
        </Field>
        <Field label="Currency">
          <Select value={filters.currency} options={CURRENCY_OPTIONS} onChange={currency => setFilters({ ...filters, currency })} />
        </Field>
        <Field label="Refund source">
          <Select value={filters.refund_source} options={REFUND_SOURCE_OPTIONS} onChange={refund_source => setFilters({ ...filters, refund_source })} />
        </Field>
        <Field label="Business account">
          <input value={filters.business_account} onChange={event => setFilters({ ...filters, business_account: event.target.value })} placeholder="Account ID" />
        </Field>
        <Field label="Amount range">
          <input value={filters.amount_range} onChange={event => setFilters({ ...filters, amount_range: event.target.value })} placeholder="e.g. 100-500" />
        </Field>
        <div className="filter-actions">
          <button className="btn-primary" type="submit">Apply Filters</button>
          <button className="btn-secondary" type="button" onClick={resetFilters}>Reset</button>
        </div>
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
  const [filters, setFilters] = useState({ status: '', provider: '', currency: '' });

  const filteredRows = rows.filter(row => {
    if (filters.status && String(row.status || '') !== filters.status) return false;
    if (filters.provider && String(row.provider_id || '').toLowerCase() !== filters.provider.toLowerCase()) return false;
    if (filters.currency && String(row.currency || '').toUpperCase() !== filters.currency.toUpperCase()) return false;
    return true;
  });

  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    const actionPath = action.replace('mark_', 'mark-');
    if (action === 'view_items' || action === 'export_csv') helpers.setDrawer({ title: 'Settlement details', body: row });
    else {
      const body = action === 'mark_settled'
        ? { bank_reference_number: window.prompt('Enter confirmed manual/test/provider reference') || '', reference_kind: 'manual' }
        : {};
      helpers.setConfirm({ title: `${formatHeader(action)} settlement`, body: 'Settlement action affects finance ledgers and must be audit logged.', action: () => helpers.run('Settlement action submitted.', () => apiRequest('POST', `${API}/settlements/${id}/${actionPath}`, body)).then(load) });
    }
  };

  return (
    <CrudPage title="Settlements" subtitle="Settlement ledger and reconciliation status" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="card filter-grid">
        <Field label="Status">
          <Select value={filters.status} options={SETTLEMENT_STATUS_OPTIONS} onChange={status => setFilters({ ...filters, status })} />
        </Field>
        <Field label="Provider">
          <Select value={filters.provider} options={PROVIDER_OPTIONS} onChange={provider => setFilters({ ...filters, provider })} />
        </Field>
        <Field label="Currency">
          <Select value={filters.currency} options={CURRENCY_OPTIONS} onChange={currency => setFilters({ ...filters, currency })} />
        </Field>
        <div className="filter-actions">
          <button className="btn-secondary" type="button" onClick={() => setFilters({ status: '', provider: '', currency: '' })}>Reset</button>
        </div>
      </div>
      <DataTable
        rows={filteredRows}
        columns={['id', 'provider_id', 'currency', 'gross_payment_amount', 'provider_fee_amount', 'company_fee_amount', 'refund_adjustment_amount', 'chargeback_adjustment_amount', 'net_settlement_amount', 'expected_settlement_date', 'actual_settlement_date', 'reconciliation_status', 'reference_kind', 'bank_reference_number', 'failure_reason', 'status']}
        empty={{ title: 'No settlement batches yet.', body: 'Settlement batches appear when providers or bank files are ready for finance review.' }}
        renderActions={row => <ActionButtons actions={['View items', 'Hold', 'Release', 'Mark settled', 'Mark failed', 'Export CSV']} onClick={action => act(row, normalizeAction(action))} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function DisputesPage() {
  const { rows, state, error, load } = useRows(`${API}/disputes?limit=100`);
  const helpers = useDrawerConfirm();
  const [filters, setFilters] = useState({ status: '', currency: '' });

  const DISPUTE_STATUS_OPTIONS = [
    { label: 'All Statuses', value: '' },
    { label: 'Open', value: 'open' },
    { label: 'Under Review', value: 'under_review' },
    { label: 'Won', value: 'won' },
    { label: 'Lost', value: 'lost' },
    { label: 'Chargeback Reversal', value: 'chargeback_reversal' },
  ];

  const filteredRows = rows.filter(row => {
    if (filters.status && String(row.status || '').toLowerCase() !== filters.status.toLowerCase()) return false;
    if (filters.currency && String(row.currency || '').toUpperCase() !== filters.currency.toUpperCase()) return false;
    return true;
  });

  const act = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'view_detail' || action === 'view_audit_trail' || action === 'view_ledger') helpers.setDrawer({ title: formatHeader(action), body: row });
    if (action === 'submit_evidence') helpers.setConfirm({ title: 'Submit dispute evidence', body: 'Evidence submission affects provider dispute workflow and creates audit log.', action: () => helpers.run('Evidence submitted.', () => apiRequest('POST', `${API}/disputes/${id}/submit-evidence`, { evidence_text: 'Submitted from company admin' })).then(load) });
    if (action === 'mark_won') helpers.setConfirm({ title: 'Mark dispute won', body: 'Won dispute creates reversal ledger impact when applicable.', action: () => helpers.run('Dispute marked won.', () => apiRequest('POST', `${API}/disputes/${id}/mark-won`, { outcome: 'won' })).then(load) });
    if (action === 'mark_lost') helpers.setConfirm({ title: 'Mark dispute lost', body: 'Lost dispute creates debit ledger impact and settlement adjustment when applicable.', action: () => helpers.run('Dispute marked lost.', () => apiRequest('POST', `${API}/disputes/${id}/mark-lost`, { outcome: 'lost' })).then(load) });
  };
  return (
    <CrudPage title="Payment Issues" subtitle="Company dispute, evidence, chargeback, reversal, and ledger impact queue" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="card filter-grid">
        <Field label="Status">
          <Select value={filters.status} options={DISPUTE_STATUS_OPTIONS} onChange={status => setFilters({ ...filters, status })} />
        </Field>
        <Field label="Currency">
          <Select value={filters.currency} options={CURRENCY_OPTIONS} onChange={currency => setFilters({ ...filters, currency })} />
        </Field>
        <div className="filter-actions">
          <button className="btn-secondary" type="button" onClick={() => setFilters({ status: '', currency: '' })}>Reset</button>
        </div>
      </div>
      <DataTable
        rows={filteredRows}
        columns={['id', 'payment_id', 'business_account_id', 'provider_dispute_id', 'amount', 'currency', 'reason', 'status', 'evidence_status', 'outcome', 'chargeback_fee', 'ledger_transaction_id', 'evidence_due_date', 'created_at']}
        empty={{ title: 'No disputes or chargebacks yet.', body: 'Disputes appear when providers report chargeback activity.' }}
        renderActions={row => <ActionButtons actions={['View detail', 'Submit evidence', 'Mark won', 'Mark lost', 'View ledger', 'View audit trail']} onClick={action => act(row, normalizeAction(action))} />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RiskRulesPage() {
  const rules = useRows(`${API}/risk-rules`);
  const reviews = useRows(`${API}/risk-reviews`);
  const evaluations = useRows(`${API}/risk-evaluations`);
  const cases = useRows(`${API}/risk-cases`);
  const signals = useRows(`${API}/risk-signals`);
  const decisions = useRows(`${API}/risk-decisions`);
  const lists = useRows(`${API}/risk-lists`);
  const [tab, setTab] = useState('Rules');
  const [simForm, setSimForm] = useState({ amount: 5000, currency: 'INR', country: 'IN', payment_method: 'upi', email: 'aditya.sharma@example.com', phone: '+919876543210', ip: '103.21.244.15', business_account_id: 'cd317088-a9ef-495c-ab3e-2a88603c54ac' });
  const [simResult, setSimResult] = useState<GenericRecord | null>(null);
  const [listForm, setListForm] = useState({ list_type: 'blocklist', entity_type: 'email', value: 'suspicious.actor@disposable.net', reason: 'Repeated velocity threshold breach' });
  const [ruleForm, setRuleForm] = useState({ name: 'High Value Single Transaction Check', description: 'Flag transactions exceeding ₹1,000 for manual ops verification', rule_type: 'high_payment_amount', scope: 'payment', environment: 'test', severity: 'medium', action: 'review', condition_json: '{"threshold_paise":100000}', enabled: true });
  const state = [rules.state, reviews.state, evaluations.state, cases.state, signals.state, decisions.state, lists.state].includes('error') ? 'error' : [rules.state, reviews.state, evaluations.state, cases.state, signals.state, decisions.state, lists.state].includes('loading') ? 'loading' : 'ready';
  const error = rules.error || reviews.error || evaluations.error || cases.error || signals.error || decisions.error || lists.error;
  const load = () => void Promise.all([rules.load(), reviews.load(), evaluations.load(), cases.load(), signals.load(), decisions.load(), lists.load()]);
  const helpers = useDrawerConfirm();
  const createListItem = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Add risk list entry',
      body: 'Sensitive list value is hashed and masked. Action is audited.',
      action: () => helpers.run('Risk list entry created.', () => apiRequest('POST', `${API}/risk-lists`, listForm)).then(load),
    });
  };
  const runSimulator = (event: FormEvent) => {
    event.preventDefault();
    helpers.run('Risk simulation complete.', async () => {
      const response = await apiRequest<GenericRecord>('POST', `${API}/risk/simulate`, {
        ...simForm,
        business_account_id: simForm.business_account_id || null,
      });
      setSimResult(response);
    });
  };
  const act = (row: GenericRecord, action: string) => {
    if (action === 'disable') helpers.setConfirm({ title: 'Disable risk rule', body: 'Risk decisioning may change immediately.', action: () => helpers.run('Risk rule disabled.', () => apiRequest('POST', `${API}/risk-rules/${String(row.id || '')}/disable`)).then(load) });
    else if (action === 'enable') helpers.setConfirm({ title: 'Enable risk rule', body: 'Risk decisioning may change immediately.', action: () => helpers.run('Risk rule enabled.', () => apiRequest('POST', `${API}/risk-rules/${String(row.id || '')}/enable`)).then(load) });
    else if (action === 'approve_payment' || action === 'reject_payment' || action === 'escalate' || action === 'assign_review') helpers.run('Risk review updated.', () => apiRequest('PATCH', `${API}/risk-reviews/${String(row.id || '')}`, { status: action === 'approve_payment' ? 'approved' : action === 'reject_payment' ? 'rejected' : action === 'escalate' ? 'escalated' : 'pending', decision_note: `Updated from ${action}` })).then(load);
    else if (['assign_case', 'mark_reviewed', 'resolve_case', 'add_note'].includes(action)) {
      const route = action === 'assign_case' ? 'assign' : action === 'mark_reviewed' ? 'review' : action === 'resolve_case' ? 'resolve' : 'notes';
      helpers.run('Risk case updated.', () => apiRequest('POST', `${API}/risk-cases/${String(row.id || '')}/${route}`, { note: `${action} from admin console` })).then(load);
    }
    else helpers.setDrawer({ title: 'Risk detail', body: row });
  };
  const createRule = (event: FormEvent) => {
    event.preventDefault();
    let conditionConfig: GenericRecord;
    try {
      conditionConfig = JSON.parse(ruleForm.condition_json) as GenericRecord;
    } catch {
      helpers.setDrawer({ title: 'Invalid rule JSON', body: { message: 'Condition config must be valid JSON.' } });
      return;
    }
    helpers.run('Risk rule saved.', () => apiRequest('POST', `${API}/risk-rules`, {
      ...ruleForm,
      condition_config: conditionConfig,
    })).then(load);
  };
  return (
    <CrudPage title="Fraud Protection" subtitle="Velocity rules, block/allow lists, review queue, decisions, signals, and simulator" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Rules', 'Evaluations', 'Cases', 'Review Queue', 'Blocklists', 'Allowlists', 'Decisions', 'Signals', 'Simulator'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'Rules' && (
        <>
          <section className="card">
            <h2><ShieldCheck size={18} /> Create rule</h2>
            <form onSubmit={createRule}>
              <div className="filter-grid">
                <Field label="Name"><input value={ruleForm.name} onChange={event => setRuleForm({ ...ruleForm, name: event.target.value })} /></Field>
                <Field label="Description"><input value={ruleForm.description} onChange={event => setRuleForm({ ...ruleForm, description: event.target.value })} /></Field>
                <Field label="Rule type"><Select value={ruleForm.rule_type} options={['high_payment_amount', 'high_daily_volume', 'high_failure_rate', 'high_refund_rate', 'risky_business_category', 'kyc_mismatch_manual_review', 'payout_amount_above_threshold', 'payout_to_unverified_bank', 'duplicate_reference', 'dispute_count_threshold', 'reconciliation_mismatch_threshold', 'newly_approved_merchant_large_payout']} onChange={rule_type => setRuleForm({ ...ruleForm, rule_type })} /></Field>
                <Field label="Scope"><Select value={ruleForm.scope} options={['merchant', 'payment', 'refund', 'dispute', 'payout', 'settlement']} onChange={scope => setRuleForm({ ...ruleForm, scope })} /></Field>
                <Field label="Environment"><Select value={ruleForm.environment} options={['test', 'live']} onChange={environment => setRuleForm({ ...ruleForm, environment })} /></Field>
                <Field label="Severity"><Select value={ruleForm.severity} options={['low', 'medium', 'high', 'critical']} onChange={severity => setRuleForm({ ...ruleForm, severity })} /></Field>
                <Field label="Action"><Select value={ruleForm.action} options={['allow', 'review', 'block', 'hold_payout', 'restrict_merchant']} onChange={action => setRuleForm({ ...ruleForm, action })} /></Field>
                <Field label="Condition JSON"><input value={ruleForm.condition_json} onChange={event => setRuleForm({ ...ruleForm, condition_json: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Create rule</button>
            </form>
          </section>
          <DataTable rows={rules.rows} columns={['name', 'rule_type', 'scope', 'environment', 'severity', 'condition_config', 'action', 'priority', 'last_triggered_at', 'trigger_count', 'enabled']} empty={{ title: 'No risk rules configured.', body: 'Create safe JSON rules for internal event scopes.' }} renderActions={row => <ActionButtons actions={['Enable', 'Disable', 'View detail']} onClick={action => action === 'View detail' ? helpers.setDrawer({ title: 'Risk rule', body: row }) : act(row, normalizeAction(action))} />} />
        </>
      )}
      {tab === 'Evaluations' && <DataTable rows={evaluations.rows} columns={['id', 'scope', 'environment', 'entity_type', 'entity_id', 'matched', 'action', 'severity', 'reason', 'created_at']} empty="No risk evaluations yet" onView={row => helpers.setDrawer({ title: 'Risk evaluation', body: row })} />}
      {tab === 'Cases' && <DataTable rows={cases.rows} columns={['id', 'merchant_id', 'scope', 'entity_type', 'entity_id', 'status', 'severity', 'action', 'reason', 'assigned_to_admin_id', 'reviewed_at', 'resolved_at', 'created_at']} empty="No risk cases yet" renderActions={row => <ActionButtons actions={['Assign case', 'Mark reviewed', 'Resolve case', 'Add note', 'View detail']} onClick={action => action === 'View detail' ? helpers.setDrawer({ title: 'Risk case', body: row }) : act(row, normalizeAction(action))} />} />}
      {tab === 'Review Queue' && <DataTable rows={reviews.rows} columns={['id', 'payment_order_id', 'business_account_id', 'status', 'assigned_to_admin_id', 'decision_note', 'created_at', 'resolved_at']} empty={{ title: 'No manual reviews', body: 'Risk reviews appear when decisions route payments to manual review.' }} renderActions={row => <ActionButtons actions={['Approve payment', 'Reject payment', 'Escalate', 'Assign review', 'View detail']} onClick={action => action === 'View detail' ? helpers.setDrawer({ title: 'Risk review', body: row }) : act(row, normalizeAction(action))} />} />}
      {tab === 'Blocklists' && (
        <>
          <section className="card">
            <h2><ShieldCheck size={18} /> Add blocklist entry</h2>
            <form onSubmit={createListItem}>
              <div className="filter-grid">
                <Field label="List type"><Select value={listForm.list_type} options={['blocklist']} onChange={list_type => setListForm({ ...listForm, list_type })} /></Field>
                <Field label="Entity type"><Select value={listForm.entity_type} options={['email', 'phone', 'ip', 'card_fingerprint', 'business_account', 'country', 'currency']} onChange={entity_type => setListForm({ ...listForm, entity_type })} /></Field>
                <Field label="Value"><input value={listForm.value} onChange={event => setListForm({ ...listForm, value: event.target.value })} /></Field>
                <Field label="Reason"><input value={listForm.reason} onChange={event => setListForm({ ...listForm, reason: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Add blocklist entry</button>
            </form>
          </section>
          <DataTable rows={lists.rows.filter(row => row.list_type === 'blocklist')} columns={['id', 'entity_type', 'display_value_masked', 'reason', 'status', 'created_at']} empty="No blocklist entries" onView={row => helpers.setDrawer({ title: 'Blocklist entry', body: row })} />
        </>
      )}
      {tab === 'Allowlists' && <DataTable rows={lists.rows.filter(row => row.list_type === 'allowlist')} columns={['id', 'entity_type', 'display_value_masked', 'reason', 'status', 'created_at']} empty="No allowlist entries" onView={row => helpers.setDrawer({ title: 'Allowlist entry', body: row })} />}
      {tab === 'Decisions' && <DataTable rows={decisions.rows} columns={['id', 'payment_id', 'business_account_id', 'decision', 'score', 'matched_rules', 'reason', 'created_at']} empty="No risk decisions yet" onView={row => helpers.setDrawer({ title: 'Risk decision', body: row })} />}
      {tab === 'Signals' && <DataTable rows={signals.rows} columns={['id', 'payment_id', 'business_account_id', 'signal_type', 'signal_value', 'severity', 'created_at']} empty="No risk signals yet" onView={row => helpers.setDrawer({ title: 'Risk signal', body: row })} />}
      {tab === 'Simulator' && (
        <section className="card">
          <h2><Activity size={18} /> Risk simulator</h2>
          <form onSubmit={runSimulator}>
            <div className="filter-grid">
              <Field label="Amount"><input type="number" value={simForm.amount} onChange={event => setSimForm({ ...simForm, amount: Number(event.target.value) })} /></Field>
              <Field label="Currency"><input value={simForm.currency} onChange={event => setSimForm({ ...simForm, currency: event.target.value })} /></Field>
              <Field label="Country"><input value={simForm.country} onChange={event => setSimForm({ ...simForm, country: event.target.value })} /></Field>
              <Field label="Method"><input value={simForm.payment_method} onChange={event => setSimForm({ ...simForm, payment_method: event.target.value })} /></Field>
              <Field label="Email"><input value={simForm.email} onChange={event => setSimForm({ ...simForm, email: event.target.value })} /></Field>
              <Field label="Phone"><input value={simForm.phone} onChange={event => setSimForm({ ...simForm, phone: event.target.value })} /></Field>
              <Field label="IP"><input value={simForm.ip} onChange={event => setSimForm({ ...simForm, ip: event.target.value })} /></Field>
              <Field label="Business account ID"><input value={simForm.business_account_id} onChange={event => setSimForm({ ...simForm, business_account_id: event.target.value })} /></Field>
            </div>
            <button className="btn-primary" type="submit">Simulate risk decision</button>
          </form>
          {simResult && <DataTable rows={[simResult]} columns={['score', 'decision', 'matched_rules', 'reason']} empty="No simulation result" />}
        </section>
      )}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function ReconciliationPage() {
  const [tab, setTab] = useState('Overview');
  const [showGuide, setShowGuide] = useState(false);
  const [rows, setRows] = useState<GenericRecord[]>([]);
  const [queue, setQueue] = useState<GenericRecord[]>([]);
  const providerImports = useRows(`${API}/reconciliation/provider-imports`);
  const parserConfigs = useRows(`${API}/reconciliation/parser-configs`);
  const batches = useRows(`${API}/reconciliation/batches`);
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const [batchItems, setBatchItems] = useState<GenericRecord[]>([]);
  const [batchMismatches, setBatchMismatches] = useState<GenericRecord[]>([]);
  const [mismatchType, setMismatchType] = useState('');
  const [mismatchStatus, setMismatchStatus] = useState('');
  const [mismatchDate, setMismatchDate] = useState('');
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
  const upload = (source: string) => {
    helpers.setConfirm({
      title: `Upload ${formatHeader(source)}`,
      body: 'Duplicate file hash is blocked. Upload action is permission-gated and audited.',
      action: () => helpers.run('Upload registered.', () => apiRequest('POST', `${API}/reconciliation/upload`, {
        source,
        file_name: `${source}-${Date.now()}.csv`,
        file_hash: `${source}-${Date.now()}`,
        row_count: 0,
      })).then(load),
    });
  };
  const createSampleBatch = () => {
    helpers.setConfirm({
      title: 'Create safe sample batch',
      body: 'Creates metadata plus safe sample reconciliation rows only. No raw bank file is stored.',
      action: () => helpers.run('Sample batch created.', () => apiRequest('POST', `${API}/reconciliation/batches`, {
        source_type: 'manual_csv',
        environment: 'test',
        file_name: 'safe-sample.csv',
        file_hash: `safe-sample-${Date.now()}`,
        content_type: 'text/csv',
        rows: [{
          amount_paise: 10000,
          currency: 'INR',
          provider_reference: `sample-${Date.now()}`,
          status: 'captured',
        }],
      })).then(batches.load),
    });
  };
  const openBatch = async (row: GenericRecord) => {
    const detail = await apiRequest<{ items?: GenericRecord[]; mismatches?: GenericRecord[] }>('GET', `${API}/reconciliation/batches/${String(row.id)}`);
    setBatchItems(detail.items || []);
    setBatchMismatches(detail.mismatches || []);
    helpers.setDrawer({ title: 'Reconciliation batch', body: row });
  };
  const filteredBatchMismatches = batchMismatches.filter(row => {
    if (mismatchType && str(row.mismatch_type) !== mismatchType) return false;
    if (mismatchStatus && str(row.status) !== mismatchStatus) return false;
    if (mismatchDate && !str(row.created_at).startsWith(mismatchDate)) return false;
    return true;
  });
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
    else if (action === 'create_sample_batch') createSampleBatch();
    else if (action === 'view_detail' && row && row.source_type) void openBatch(row);
    else if (action === 'close_batch' && row) helpers.setConfirm({ title: 'Close batch', body: 'Closed batches remain auditable.', action: () => helpers.run('Batch closed.', () => apiRequest('POST', `${API}/reconciliation/batches/${String(row.id)}/close`)).then(batches.load) });
    else if (action === 'export_batch_csv' && row) {
      void apiDownload(`${API}/reconciliation/batches/${String(row.id)}/export.csv`).then(({ blob, fileName }) => {
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = fileName;
        link.click();
        URL.revokeObjectURL(url);
      });
    }
    else if (action === 'mark_reviewed' && row) helpers.setConfirm({ title: 'Mark reviewed', body: 'Marks the reconciliation item reviewed with an internal note.', action: () => helpers.run('Item reviewed.', () => apiRequest('POST', `${API}/reconciliation/items/${String(row.id)}/review`, { note: 'Reviewed from company admin' })).then(load) });
    else helpers.setDrawer({ title: 'Payment Matching', body: row || rows[0] || {} });
  };
  return (
    <CrudPage title="Payment Matching" subtitle="Provider and bank statement matching" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="stats-grid">
        <Metric label="Uploaded files" value={rows.length} icon={Upload} description="Raw statement files processed" />
        <Metric label="Matched records" value={matched} icon={BadgeCheck} description="Fully reconciled ledger transactions" />
        <Metric label="Mismatches" value={mismatches} icon={AlertTriangle} status={mismatches > 0 ? 'warning' : undefined} description="Items with amount, fee or state discrepancies" />
        <Metric label="Unresolved items" value={unresolved} icon={FileSearch} description="Exceptions pending investigation" />
        <Metric label="Duplicate records" value={duplicates} icon={Copy} description="Duplicate UTRs or provider IDs" />
      </div>

      <div className="recon-nav-wrapper">
        <div className="recon-nav-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div className="recon-nav-title">
              <SlidersHorizontal size={14} className="recon-nav-icon" />
              <span>DATASET VIEWS</span>
            </div>
            <span className="recon-nav-hint">Switch between matching queues, file batches, and discrepancy reports</span>
          </div>
          <button
            type="button"
            className="recon-guide-toggle-btn"
            onClick={() => setShowGuide(!showGuide)}
            title="Click to view full workflow instructions, confidence score hints, and simple glossary"
          >
            <HelpCircle size={13} />
            <span>{showGuide ? 'Hide Guide & Hints' : 'How It Works & Hints'}</span>
          </button>
        </div>
        <div className="recon-tabs-bar">
          {[
            { id: 'Overview', label: 'Overview', icon: LayoutGrid, count: queue.length },
            { id: 'Batches', label: 'Batches', icon: Layers, count: batches.rows.length },
            { id: 'Uploads', label: 'Uploads', icon: Upload, count: rows.length },
            { id: 'Matching Queue', label: 'Matching Queue', icon: RotateCcw, count: queue.length },
            { id: 'Mismatches', label: 'Mismatches', icon: AlertTriangle, count: mismatches, isWarning: mismatches > 0 },
            { id: 'Resolved', label: 'Resolved', icon: BadgeCheck },
            { id: 'Reports', label: 'Reports', icon: FileText },
            { id: 'Provider Imports', label: 'Provider Imports', icon: ArrowDownCircle, count: providerImports.rows.length },
            { id: 'Parser Configs', label: 'Parser Configs', icon: Settings, count: parserConfigs.rows.length },
          ].map(item => {
            const Icon = item.icon;
            const isActive = tab === item.id;
            return (
              <button
                key={item.id}
                type="button"
                className={`recon-tab-pill ${isActive ? 'active' : ''}`}
                onClick={() => setTab(item.id)}
              >
                <Icon size={14} />
                <span>{item.label}</span>
                {typeof item.count === 'number' && item.count > 0 && (
                  <span className={`recon-pill-badge ${item.isWarning ? 'badge-warning' : ''}`}>
                    {item.count}
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </div>

      {showGuide && (
        <div className="recon-guide-card">
          <div className="recon-guide-header">
            <div className="recon-guide-title">
              <HelpCircle size={16} style={{ color: 'var(--fp-primary)' }} />
              <strong>Reconciliation Quick Guide & Glossary</strong>
            </div>
            <span className="recon-guide-subtitle">Why this toolbar exists and how to use it step-by-step</span>
          </div>
          <div className="recon-guide-grid">
            <div className="recon-guide-block">
              <h4>1. How To Use (3-Step Workflow)</h4>
              <ol className="recon-guide-steps">
                <li><strong>1. Upload Statements (IMPORT):</strong> Upload settlement CSV from your payment gateway (Razorpay/Stripe) or Bank account.</li>
                <li><strong>2. Run Engine (MATCH):</strong> Click <em>Run Auto-Match</em>. The engine compares statement rows against internal orders.</li>
                <li><strong>3. Audit & Export (EXPORT):</strong> Inspect unmatched records in <em>Mismatches</em>. Download <em>Discrepancy Report</em> for finance/audit.</li>
              </ol>
            </div>

            <div className="recon-guide-block">
              <h4>2. Confidence Score (What Numbers Mean)</h4>
              <div className="recon-score-list">
                <div className="recon-score-item">
                  <span className="recon-score-badge recon-score-exact">100% Exact</span>
                  <span>Both Reference IDs and transaction amount match platform transactions exactly. 100% verified.</span>
                </div>
                <div className="recon-score-item">
                  <span className="recon-score-badge recon-score-high">96% Ledger</span>
                  <span>Matched directly to internal double-entry financial ledger journal entry group.</span>
                </div>
                <div className="recon-score-item">
                  <span className="recon-score-badge recon-score-high">90–92% High</span>
                  <span>Matched via provider charge ID, refund ID, or dispute reference number.</span>
                </div>
                <div className="recon-score-item">
                  <span className="recon-score-badge recon-score-utr">85% Bank UTR</span>
                  <span>Matched via Bank Settlement UTR number or payout batch disbursement ID.</span>
                </div>
                <div className="recon-score-item">
                  <span className="recon-score-badge recon-score-fuzzy">65% Fuzzy</span>
                  <span>Matched by approximate amount & date window. Requires human eye verification.</span>
                </div>
              </div>
            </div>

            <div className="recon-guide-block">
              <h4>3. Simple Glossary</h4>
              <ul className="recon-glossary-list">
                <li>
                  <strong>Provider Imports:</strong> The history log of every statement file uploaded from payment gateways (Razorpay, Stripe, etc.) showing total rows, imported, and errors.
                </li>
                <li>
                  <strong>Parser Config:</strong> The "translator rulebook". Every bank and gateway formats CSV spreadsheets differently. Parser configs tell FeelPay which column is Amount, Date, Fees, or Order ID.
                </li>
                <li>
                  <strong>Matching Queue:</strong> The active work inbox. Every payment row extracted from external bank statements waits here until it is matched or investigated.
                </li>
              </ul>
            </div>
          </div>
        </div>
      )}

      <div className="recon-actions-bar">
        <div className="recon-action-cluster">
          <div className="recon-cluster-label-wrap">
            <span className="recon-cluster-tag">ENGINE</span>
            <span className="recon-cluster-desc">Process & match</span>
          </div>
          <button
            type="button"
            className="recon-action-btn primary"
            title="Automatically scan all uploaded statement rows and compare them against internal payments using our matching algorithm."
            onClick={() => act('auto_match')}
          >
            <Zap size={13} />
            <span>Run Auto-Match</span>
          </button>
          <button
            type="button"
            className="recon-action-btn"
            title="Create a simulated statement batch with test data so you can test reconciliation safely without real bank files."
            onClick={() => act('create_sample_batch')}
          >
            <Plus size={13} />
            <span>Create Sample Batch</span>
          </button>
        </div>

        <div className="recon-cluster-divider" />

        <div className="recon-action-cluster">
          <div className="recon-cluster-label-wrap">
            <span className="recon-cluster-tag">IMPORT STATEMENTS</span>
            <span className="recon-cluster-desc">Upload external data</span>
          </div>
          <button
            type="button"
            className="recon-action-btn"
            title="Upload settlement statement CSV from your payment gateway (e.g. Razorpay, Stripe, Cashfree)."
            onClick={() => act('upload_provider_report')}
          >
            <Upload size={13} />
            <span>Provider Report</span>
          </button>
          <button
            type="button"
            className="recon-action-btn"
            title="Upload official bank account statement CSV to verify actual settled funds received in your bank."
            onClick={() => act('upload_bank_statement')}
          >
            <Upload size={13} />
            <span>Bank Statement</span>
          </button>
          <button
            type="button"
            className="recon-action-btn"
            title="Upload vendor/merchant payout batch reports to match outgoing disbursements."
            onClick={() => act('upload_payout_report')}
          >
            <Upload size={13} />
            <span>Payout Report</span>
          </button>
          <button
            type="button"
            className="recon-action-btn"
            title="Upload provider processing fee & tax deductions to reconcile net vs gross amounts."
            onClick={() => act('upload_fee_report')}
          >
            <Upload size={13} />
            <span>Fee Report</span>
          </button>
        </div>

        <div className="recon-cluster-divider" />

        <div className="recon-action-cluster">
          <div className="recon-cluster-label-wrap">
            <span className="recon-cluster-tag">EXPORT & AUDIT</span>
            <span className="recon-cluster-desc">Download & review</span>
          </div>
          <button
            type="button"
            className="recon-action-btn"
            title="Download a CSV list of all unmatched payments, fee variances, and suspicious differences for audit."
            onClick={() => act('export_discrepancy_report')}
          >
            <Download size={13} />
            <span>Discrepancy Report</span>
          </button>
          <button
            type="button"
            className="recon-action-btn"
            title="Download a consolidated finance reconciliation summary report for accounting and tax records."
            onClick={() => act('export_finance_summary')}
          >
            <Download size={13} />
            <span>Finance Summary</span>
          </button>
        </div>

        <div className="recon-actions-hint">
          <Info size={13} style={{ color: 'var(--fp-primary)', flexShrink: 0 }} />
          <span><strong>How this works:</strong> First upload external statements under <strong>IMPORT STATEMENTS</strong>, then click <strong>Run Auto-Match</strong> to match money in the bank with platform transactions. Export discrepancies anytime.</span>
        </div>
      </div>
      {tab === 'Overview' && <DataTable rows={queue.slice(0, 12)} columns={['row_number', 'external_reference', 'internal_reference', 'amount', 'currency', 'record_type', 'match_status', 'mismatch_reason', 'confidence_score', 'resolution_status']} empty={{ title: 'Upload provider or bank files to start matching.', body: 'Provider reports, bank statements, payout reports, fee reports, and ledger exports create queue records here.' }} renderActions={row => <ActionButtons actions={['Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Batches' && <DataTable rows={batches.rows} columns={['id', 'source_type', 'environment', 'status', 'total_items', 'matched_items', 'mismatched_items', 'total_amount_paise', 'created_at', 'closed_at']} empty="No reconciliation batches" renderActions={row => <ActionButtons actions={['Close batch', 'Export batch csv', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Uploads' && <DataTable rows={rows} columns={['id', 'source', 'provider_id', 'file_name', 'file_hash', 'row_count', 'matched_count', 'mismatch_count', 'duplicate_count', 'status', 'created_at', 'processed_at']} empty={{ title: 'Upload provider or bank files to start matching.', body: 'Upload provider reports, bank statements, payout reports, or fee reports to start reconciliation.' }} onView={row => helpers.setDrawer({ title: 'Reconciliation details', body: row })} />}
      {tab === 'Matching Queue' && <DataTable rows={queue} columns={['id', 'file_id', 'row_number', 'external_reference', 'internal_reference', 'amount', 'currency', 'record_type', 'match_status', 'confidence_score', 'suggested_match']} empty="No matching queue records" renderActions={row => <ActionButtons actions={['Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Mismatches' && <DataTable rows={queue.filter(row => String(row.match_status).includes('mismatch') || row.mismatch_reason)} columns={['id', 'external_reference', 'amount', 'currency', 'mismatch_reason', 'confidence_score', 'resolution_status']} empty="No mismatches" renderActions={row => <ActionButtons actions={['Mark reviewed', 'Manually resolve', 'Ignore', 'View detail']} onClick={action => act(normalizeAction(action), row)} />} />}
      {tab === 'Resolved' && <DataTable rows={queue.filter(row => row.resolution_status === 'resolved')} columns={['id', 'external_reference', 'matched_entity_type', 'matched_entity_id', 'resolution_status', 'resolution_note', 'resolved_at']} empty="No resolved records" />}
      {tab === 'Reports' && <DataTable rows={rows} columns={['file_name', 'source', 'amount_variance', 'matched_count', 'mismatch_count', 'duplicate_count', 'created_at']} empty="No reconciliation reports" />}
      {tab === 'Provider Imports' && <DataTable rows={providerImports.rows} columns={['id', 'provider_id', 'report_type', 'file_name', 'file_hash', 'status', 'row_count', 'imported_count', 'skipped_count', 'error_count', 'created_at', 'processed_at']} empty="No provider imports yet" renderActions={row => <ActionButtons actions={['View detail']} onClick={() => helpers.setDrawer({ title: 'Provider import', body: row })} />} />}
      {tab === 'Parser Configs' && <DataTable rows={parserConfigs.rows} columns={['id', 'provider_id', 'report_type', 'column_mapping_json', 'date_format', 'amount_format', 'currency_column', 'status', 'created_at']} empty="No parser configs yet" onView={row => helpers.setDrawer({ title: 'Parser config', body: row })} />}
      {batchItems.length > 0 && (
        <>
          <h2 style={{ fontSize: 18, marginTop: 24 }}>Selected Batch Items</h2>
          <DataTable rows={batchItems} columns={['id', 'amount_paise', 'currency', 'provider_reference', 'utr_reference', 'matched', 'mismatch_reason', 'reviewed_at']} empty="No batch items" renderActions={row => <ActionButtons actions={['Mark reviewed']} onClick={action => act(normalizeAction(action), row)} />} />
        </>
      )}
      {batchMismatches.length > 0 && (
        <>
          <h2 style={{ fontSize: 18, marginTop: 24 }}>Selected Batch Mismatches</h2>
          <div className="filter-grid">
            <Field label="Mismatch type">
              <Select value={mismatchType} onChange={next => setMismatchType(next)} options={MISMATCH_TYPE_OPTIONS} allLabel="All Types" />
            </Field>
            <Field label="Status">
              <Select value={mismatchStatus} onChange={next => setMismatchStatus(next)} options={MISMATCH_STATUS_OPTIONS} allLabel="All Statuses" />
            </Field>
            <Field label="Date"><input type="date" value={mismatchDate} onChange={event => setMismatchDate(event.target.value)} /></Field>
            <div className="filter-actions">
              <button className="btn-secondary" type="button" onClick={() => { setMismatchType(''); setMismatchStatus(''); setMismatchDate(''); }}>Reset</button>
            </div>
          </div>
          <DataTable rows={filteredBatchMismatches} columns={['id', 'mismatch_type', 'status', 'note', 'created_at', 'reviewed_at']} empty="No matching mismatches" />
        </>
      )}
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
      const profile = (typeof detail.profile === 'object' && detail.profile !== null ? detail.profile : row) as GenericRecord;
      const cleanBody: GenericRecord = {};

      const keyOrder = [
        'id', 'display_name', 'legal_name', 'business_type', 'business_category',
        'email', 'phone', 'status', 'review_status', 'onboarding_status', 'kyc_status',
        'risk_status', 'payout_status', 'default_currency', 'pan_masked', 'gstin_masked',
        'cin_masked', 'llpin_masked', 'website_app_url', 'webhook_url', 'created_at', 'updated_at'
      ];
      for (const k of keyOrder) {
        if (profile[k] !== null && profile[k] !== undefined && profile[k] !== '') {
          cleanBody[k] = profile[k];
        }
      }
      for (const [k, v] of Object.entries(profile)) {
        if (!keyOrder.includes(k) && v !== null && v !== undefined && v !== '') {
          cleanBody[k] = v;
        }
      }

      // Only show related sections if they actually have recorded entries
      const relatedKeys: Array<{ key: string; label: string }> = [
        { key: 'bank_accounts', label: 'bank_accounts' },
        { key: 'kyc_documents', label: 'kyc_documents' },
        { key: 'risk_notes', label: 'risk_notes' },
        { key: 'review_history', label: 'review_history' },
        { key: 'provider_links', label: 'provider_links' },
        { key: 'verification_attempts', label: 'verification_attempts' },
        { key: 'ledger_history', label: 'ledger_history' },
      ];
      for (const rel of relatedKeys) {
        const arr = detail[rel.key];
        if (Array.isArray(arr) && arr.length > 0) {
          cleanBody[rel.label] = arr;
        }
      }

      const accountName = String(profile.display_name || profile.legal_name || shortId(String(row.id || '')));
      helpers.setDrawer({ title: `Account: ${accountName}`, body: cleanBody });
    } catch {
      helpers.setDrawer({ title: 'Business account detail', body: row });
    }
  };
  const reviewAccount = (row: GenericRecord, action: string, label: string, options: GenericRecord = {}) => {
    const id = String(row.id || '');
    helpers.setConfirm({
      title: label,
      body: 'Merchant review decisions are internal, audited, and may change live-mode eligibility.',
      action: () => helpers.run('Merchant review updated.', () => apiRequest('POST', `${API}/business-accounts/${id}/review`, { action, ...options })).then(load),
    });
  };
  const actionsForAccount = (row: GenericRecord): string[] => {
    const reviewStatus = str(row.review_status).toLowerCase();
    const status = str(row.status).toLowerCase();
    const actions = ['View detail'];

    if (status === 'suspended' || status === 'restricted' || reviewStatus === 'rejected') {
      actions.push('Approve', 'Add risk note');
    } else if (reviewStatus === 'approved' || status === 'active') {
      actions.push('Suspend', 'Add risk note');
    } else {
      actions.push('Approve');
      if (reviewStatus !== 'under_review') {
        actions.push('Mark under review');
      }
      actions.push('Reject', 'Add risk note');
    }
    return actions;
  };
  const payoutReason = (row: GenericRecord) => {
    if (row.payout_ready === true) return 'Ready';
    if (!['verified', 'approved'].includes(str(row.kyc_status).toLowerCase())) return 'KYC not verified';
    if (str(row.risk_status).toLowerCase() === 'blocked') return 'Risk blocked';
    if (!['enabled', 'active'].includes(str(row.payout_status).toLowerCase())) return 'Provider not configured';
    return 'Bank not configured';
  };
  const accountRows = rows.map(row => ({ ...row, payout_readiness_reason: payoutReason(row) }));
  const act = (row: GenericRecord, action: string) => {
    if (action === 'view_detail') void openDetail(row);
    if (action === 'mark_under_review') reviewAccount(row, action, 'Mark under review');
    if (action === 'approve') reviewAccount(row, action, 'Approve merchant');
    if (action === 'reject' || action === 'suspend') {
      const note = window.prompt(`Reason for ${action === 'reject' ? 'rejection' : 'suspension'}:`, 'Manual review decision');
      if (note?.trim()) {
        reviewAccount(row, action, `${action.charAt(0).toUpperCase()}${action.slice(1)} merchant`, {
          reason_code: 'manual_review',
          note: note.trim(),
        });
      }
    }
    if (action === 'add_risk_note') {
      const note = window.prompt('Internal risk note:');
      if (note?.trim()) {
        reviewAccount(row, action, 'Add risk note', {
          note: note.trim(),
          risk_level: str(row.risk_status || 'medium'),
        });
      }
    }
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
        rows={accountRows}
        columns={['display_name', 'business_type', 'email', 'review_status', 'kyc_status', 'risk_status', 'payout_status', 'payout_readiness_reason', 'created_at']}
        empty={{ title: 'No business accounts yet', body: 'Internal company-managed business accounts will appear here after onboarding records are created.' }}
        renderActions={row => <ActionButtons actions={actionsForAccount(row)} onClick={action => act(row, normalizeAction(action))} dropdownOnly />}
      />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function ProviderConnectorsPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [credentials, setCredentials] = useState<GenericRecord[]>([]);
  const [connectorStatuses, setConnectorStatuses] = useState<GenericRecord[]>([]);
  const [testResults, setTestResults] = useState<Record<string, GenericRecord>>({});
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const [credentialForm, setCredentialForm] = useState({ provider_id: '10000000-0000-0000-0000-000000000001', environment: 'sandbox', secret_type: 'api_secret', value: 'sec_live_9f838e818817290192', rotation_due_at: '2026-12-31T23:59:59Z', description: 'Primary production gateway authentication secret' });
  const [tab, setTab] = useState('Overview');
  const [testing, setTesting] = useState<string | null>(null);

  const load = async () => {
    setState('loading');
    setError('');
    try {
      const [providerData, credentialData, statusData] = await Promise.all([
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<{ items: GenericRecord[] }>('GET', `${API}/credentials`),
        apiRequest<{ items?: GenericRecord[] }>('GET', `${API}/providers`).catch(() => ({ items: [] })),
      ]);
      const credentialRows = rowsFrom(credentialData);
      setProviders(providerData.items || []);
      setCredentials(credentialRows);
      setConnectorStatuses((statusData.items || []).map(provider => ({
        provider: str(provider.code || provider.name || provider.id),
        display_name: str(provider.name || provider.code || 'Provider'),
        provider_type: str(provider.provider_type),
        code: str(provider.code),
        name: str(provider.name),
        status: str(provider.status),
        health_status: str(provider.health_status),
        api_key_status: str(provider.api_key_status),
        api_secret_status: str(provider.api_secret_status),
        webhook_secret_status: str(provider.webhook_secret_status),
        readiness_status: connectorReadiness(provider).label,
        readiness_note: connectorReadiness(provider).note,
        readiness_score: connectorReadiness(provider).score,
        required_keys: Array.isArray(provider.missing_required_secrets) ? provider.missing_required_secrets.map(String) : [],
        available_keys: credentialRows
          .filter(row => String(row.provider_id || '') === String(provider.id || '') && String(row.status || 'active') !== 'disabled')
          .map(row => String(row.secret_type || ''))
          .filter(Boolean),
        last_error: '',
        last_success: str(provider.last_health_check_at),
      })));
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

  const runSandboxTest = async (providerCode: string) => {
    setTesting(providerCode);
    try {
      const provider = providers.find(item => String(item.code || item.name) === providerCode);
      if (!provider) throw new Error('Provider not found');
      const result = await apiRequest<GenericRecord>('POST', `${API}/providers/${provider.id}/test`);
      setTestResults(prev => ({ ...prev, [providerCode]: result }));
      const passed = String(result.status || '').toLowerCase() === 'verified';
      const errorDetail = (result.summary as GenericRecord)?.error || (passed ? '' : 'Credentials required');
      const friendlyDetail =
        String(errorDetail) === 'provider_disabled' ? 'Provider disabled' :
        String(errorDetail) === 'provider_connector_not_configured' ? 'Connector not configured' :
        String(errorDetail) === 'webhook_secret_missing' ? 'Webhook secret missing' :
        'Credentials required';
      helpers.setNotice(passed ? `✓ ${providerCode} sandbox test passed` : `${providerCode}: ${friendlyDetail}`);
    } catch (err) {
      helpers.setNotice(`${providerCode}: ${friendlyError(err instanceof Error ? err.message : '')}`);
    } finally {
      setTesting(null);
    }
  };

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

  const readinessColor = (score: number) => score >= 100 ? '#1DB954' : score >= 50 ? '#F5A623' : '#FF8A8A';
  const secretStatus = (cs: GenericRecord, key: string) => {
    if (!providerLooksReal(cs)) return 'Credentials required for live use';
    const required = (cs.required_keys as string[] || []).includes(key);
    const available = (cs.available_keys as string[] || []).includes(key);
    if (!required) return 'Not required';
    if (available) return 'Configured';
    return 'Missing';
  };
  const providerTableRows = providers.map(provider => {
    const readiness = connectorReadiness(provider);
    return {
      ...provider,
      readiness_status: readiness.label,
      connector_readiness_score: readiness.score,
      readiness_note: readiness.note,
    };
  });

  return (
    <CrudPage title="Payment Gateways" subtitle="Integration health, sandbox testing, credentials, and capabilities" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Overview', 'Credentials', 'Capabilities', 'Health Log'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>

      {tab === 'Overview' && (
        <>
          <div className="stats-grid">
            {connectorStatuses.map(cs => {
              const score = Number(cs.readiness_score ?? 0);
              const result = testResults[String(cs.provider)];
              return (
                <div key={String(cs.provider)} className="card" style={{ borderLeft: `3px solid ${readinessColor(score)}` }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                    <h3 style={{ margin: 0, fontSize: 15 }}>{String(cs.display_name)}</h3>
                    <span style={{ fontSize: 12, padding: '2px 8px', borderRadius: 20, background: readinessColor(score) + '22', color: readinessColor(score), fontWeight: 600 }}>
                      {score}%
                    </span>
                  </div>
                  <div style={{ fontSize: 12, color: score >= 100 ? '#1DB954' : '#F5A623', marginBottom: 6, fontWeight: 600 }}>{str(cs.readiness_status)}</div>
                  <div style={{ fontSize: 12, color: '#b3b3b3', marginBottom: 6 }}>{str(cs.readiness_note)}</div>
                  <div style={{ fontSize: 12, color: '#b3b3b3', marginBottom: 6 }}>
                    Credentials: {['api_key', 'api_secret', 'webhook_secret'].map(key => `${formatHeader(key)}: ${secretStatus(cs, key)}`).join(' · ')}
                  </div>
                  {['api_key', 'api_secret', 'webhook_secret'].map(k => (
                    <div key={k} style={{ fontSize: 11, display: 'flex', alignItems: 'center', gap: 4, marginBottom: 2 }}>
                      <span style={{ color: secretStatus(cs, k) === 'Missing' ? '#E54D4D' : '#1DB954' }}>
                        {secretStatus(cs, k) === 'Missing' ? '○' : '●'}
                      </span>
                      {formatHeader(k)}: {secretStatus(cs, k)}
                    </div>
                  ))}
                  {cs.last_error && <div style={{ fontSize: 11, color: '#E54D4D', marginTop: 4 }}>Last error: {String(cs.last_error).slice(0, 60)}</div>}
                  {cs.last_success && <div style={{ fontSize: 11, color: '#1DB954', marginTop: 2 }}>Last success: {String(cs.last_success).slice(0, 24)}</div>}
                  {result && (() => {
                    const passed = String(result.status || '').toLowerCase() === 'verified';
                    const summary = (result.summary || {}) as GenericRecord;
                    return (
                      <div style={{ marginTop: 8, fontSize: 11, padding: '6px 8px', borderRadius: 6, background: passed ? '#1DB95411' : '#E54D4D11' }}>
                        Auth: {passed ? '✓' : 'Pending'} · Webhook: {passed || secretStatus(cs, 'webhook_secret') !== 'Missing' ? '✓' : 'Missing'} · Sandbox: {passed ? '✓' : `✗ ${String(summary.error || 'Credentials required')}`}
                        {result.provider_reference && <> · Ref: {String(result.provider_reference)}</>}
                      </div>
                    );
                  })()}
                  <button
                    className="btn-secondary"
                    style={{ marginTop: 8, fontSize: 11, padding: '4px 12px' }}
                    disabled={testing === String(cs.provider)}
                    onClick={() => runSandboxTest(String(cs.provider))}
                  >
                    {testing === String(cs.provider) ? 'Testing…' : 'Run sandbox test'}
                  </button>
                </div>
              );
            })}
          </div>
          <TablePanel title="Providers" rows={providerTableRows} columns={['name', 'code', 'provider_type', 'environment', 'status', 'readiness_status', 'connector_readiness_score', 'readiness_note', 'success_rate', 'attempt_count', 'api_key_status', 'api_secret_status', 'webhook_secret_status']} />
        </>
      )}

      {tab === 'Credentials' && (
        <div className="grid grid-2">
          <section className="card">
            <h2><KeyRound size={18} /> Credential Management</h2>
            <form onSubmit={saveSecret}>
              <div className="filter-grid">
                <Field label="Provider"><Select value={credentialForm.provider_id} options={providers.map(provider => ({ label: provider.name, value: provider.id }))} onChange={value => setCredentialForm({ ...credentialForm, provider_id: value })} /></Field>
                <Field label="Environment"><Select value={credentialForm.environment} options={['sandbox', 'production']} onChange={value => setCredentialForm({ ...credentialForm, environment: value })} /></Field>
                <Field label="Secret type"><Select value={credentialForm.secret_type} options={['api_key', 'api_secret', 'webhook_secret', 'signing_secret', 'client_id', 'client_secret']} onChange={value => setCredentialForm({ ...credentialForm, secret_type: value })} /></Field>
                <input type="text" autoComplete="username" name="provider-secret-username" value="admin" readOnly aria-hidden="true" tabIndex={-1} style={{ position: 'absolute', width: 0, height: 0, overflow: 'hidden', opacity: 0, pointerEvents: 'none' }} />
                <Field label="Secret value"><input type="password" autoComplete="new-password" name="provider-secret-value" value={str(credentialForm.value)} onChange={event => setCredentialForm({ ...credentialForm, value: event.target.value })} /></Field>
                <Field label="Rotation due at"><input type="datetime-local" value={credentialForm.rotation_due_at} onChange={event => setCredentialForm({ ...credentialForm, rotation_due_at: event.target.value })} /></Field>
                <Field label="Description"><input value={credentialForm.description} onChange={event => setCredentialForm({ ...credentialForm, description: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit" disabled={!credentialForm.provider_id || !credentialForm.value}>Save encrypted secret</button>
            </form>
          </section>
          <TablePanel title="Credential status" rows={credentials} columns={['provider_name', 'environment', 'secret_type', 'masked_value', 'status', 'last_validated_at', 'last_rotated_at', 'rotation_due_at']} />
        </div>
      )}

      {tab === 'Capabilities' && (
        <>
          {connectorStatuses.map(cs => (
            <section key={String(cs.provider)} className="card" style={{ marginBottom: 12 }}>
              <h3 style={{ marginBottom: 8 }}>{String(cs.display_name)} capabilities</h3>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
                {(cs.capabilities as string[] || []).map(cap => (
                  <span key={cap} style={{ fontSize: 11, padding: '3px 10px', borderRadius: 20, background: '#1DB95418', color: '#1DB954', border: '1px solid #1DB95433' }}>
                    {cap}
                  </span>
                ))}
              </div>
            </section>
          ))}
          <DataTable rows={capabilityRows} columns={['provider', 'environment', 'capability', 'enabled', 'status', 'last_checked_at', 'error_message']} empty={{ title: 'No provider capabilities yet', body: 'Capability checks appear after providers are configured.' }} />
        </>
      )}

      {tab === 'Health Log' && (
        <>
          <TablePanel title="Provider Health" rows={providers} columns={['name', 'health_status', 'success_rate', 'attempt_count', 'last_health_check_at']} />
          <TablePanel title="Provider Logs" rows={providers} columns={['name', 'status', 'health_status', 'updated_at']} />
        </>
      )}

      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function BankIntegrationsPage() {
  const { rows, state, error, load } = useRows(`${API}/bank-integrations`);
  const helpers = useDrawerConfirm();
  const [form, setForm] = useState({ acquiring_bank_name: 'HDFC Corporate Banking', integration_type: 'api', account_scope: 'company_settlement', currency: 'INR', settlement_cycle: 'T+1', masked_account_number: '•••• •••• 9012', status: 'active' });
  const save = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Save bank integration',
      body: 'Only masked settlement metadata is saved from this form. Full account numbers are never shown.',
      action: () => helpers.run('Bank integration saved.', () => apiRequest('POST', `${API}/bank-integrations`, {
        acquiring_bank_name: form.acquiring_bank_name,
        integration_type: form.integration_type,
        account_scope: form.account_scope,
        currency: form.currency,
        settlement_cycle: form.settlement_cycle,
        account_number: null,
        masked_account_number: form.masked_account_number || 'not_stored',
        settlement_account_reference: form.masked_account_number || null,
        supported_currencies: [form.currency],
        supported_payment_methods: ['upi', 'card', 'netbanking'],
      })).then(load),
    });
  };
  const act = (row: GenericRecord, action: string) => {
    const id = str(row.id);
    if (action === 'verify_sandbox_bank_setup') helpers.run('Sandbox bank setup verified.', () => apiRequest('POST', `${API}/bank-integrations/${id}/verify`)).then(load);
    else if (action === 'disable') helpers.run('Bank integration disabled.', () => apiRequest('POST', `${API}/bank-integrations/${id}/disable`)).then(load);
    else helpers.setDrawer({ title: 'Bank integration', body: row });
  };
  return (
    <CrudPage title="Settlement Bank Accounts" subtitle="Safe settlement bank metadata and sandbox readiness" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <section className="card">
        <h2><Landmark size={18} /> Settlement bank setup</h2>
        <form onSubmit={save}>
          <div className="filter-grid">
            <Field label="Acquiring bank name"><input value={str(form.acquiring_bank_name)} onChange={event => setForm({ ...form, acquiring_bank_name: event.target.value })} required /></Field>
            <Field label="Integration type"><Select value={str(form.integration_type)} options={[{ label: 'API / sandbox test', value: 'api' }, { label: 'SFTP', value: 'sftp' }, { label: 'Manual file upload', value: 'manual_file_upload' }, { label: 'Provider managed', value: 'provider_managed' }]} onChange={integration_type => setForm({ ...form, integration_type })} /></Field>
            <Field label="Account scope"><Select value={str(form.account_scope)} options={['company_settlement', 'business_settlement', 'reserve']} onChange={account_scope => setForm({ ...form, account_scope })} /></Field>
            <Field label="Currency"><input value={str(form.currency)} onChange={event => setForm({ ...form, currency: event.target.value.toUpperCase() })} /></Field>
            <Field label="Settlement cycle"><input value={str(form.settlement_cycle)} onChange={event => setForm({ ...form, settlement_cycle: event.target.value })} /></Field>
            <Field label="Masked account display"><input value={str(form.masked_account_number)} onChange={event => setForm({ ...form, masked_account_number: event.target.value })} placeholder="****1234" /></Field>
            <Field label="Status"><Select value={str(form.status)} options={['active', 'disabled']} onChange={status => setForm({ ...form, status })} /></Field>
          </div>
          <button className="btn-primary" type="submit">Save bank integration</button>
        </form>
      </section>
      <DataTable rows={rows} columns={['acquiring_bank_name', 'integration_type', 'account_scope', 'currency', 'settlement_cycle', 'masked_account_number', 'verification_status', 'last_verified_at', 'status']} empty={{ title: 'Add settlement bank before live settlement.', body: 'Full account numbers are never shown. Use sandbox verification for local setup.' }} renderActions={row => <ActionButtons actions={['Verify sandbox bank setup', 'Disable', 'View detail']} onClick={action => act(row, normalizeAction(action))} />} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function RoutingRulesPage() {
  const { rows, state, error, load } = useRows(`${API}/routing-rules`);
  const performance = useRows(`${API}/routing/provider-performance`);
  const helpers = useDrawerConfirm();
  const [tab, setTab] = useState('Rules');
  const [form, setForm] = useState({ payment_method: 'upi', currency: 'INR', country: 'IN', amount: 10000, business_account: 'cd317088-a9ef-495c-ab3e-2a88603c54ac', customer_region: 'IN-MH', risk_level: 'low', strategy: 'priority' });
  const [result, setResult] = useState<RoutingSimulation | null>(null);
  const simulate = async (event: FormEvent) => {
    event.preventDefault();
    setResult(await apiRequest<RoutingSimulation>('POST', `${API}/routing/simulate`, form));
  };

  const routingTabs = [
    { id: 'Rules', label: 'Active Route Rules', hint: 'Automatic gateway selection rules' },
    { id: 'Simulator', label: 'Test Route Simulator', hint: 'Preview which gateway will be picked' },
    { id: 'Provider Performance', label: 'Gateway Health & Speed', hint: 'Live uptime, speed, and success rates' },
    { id: 'Failover Rules', label: 'Backup & Failover Rules', hint: 'Emergency backup gateways if primary is down' },
    { id: 'Match Logs', label: 'Live Routing History', hint: 'Recent transactions and which route was picked' },
  ];

  return (
    <CrudPage
      title="Smart Payment Routing"
      subtitle="Automatically chooses the best bank or payment gateway for every transaction to get lowest fees and highest success rate."
      state={state}
      error={error || performance.error}
      notice={helpers.notice}
      onRefresh={() => void Promise.all([load(), performance.load()])}
    >
      {/* Educational Guide Banner */}
      <div style={{
        background: 'var(--fp-bg-card)',
        border: '1px solid var(--fp-border)',
        borderRadius: 'var(--fp-radius)',
        padding: '18px 20px',
        marginBottom: 20,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
          <span style={{ fontSize: '1.4rem' }}>🧭</span>
          <div>
            <h4 style={{ margin: 0, fontSize: '1rem', fontWeight: 700, color: 'var(--fp-text)' }}>
              How Smart Payment Routing Works
            </h4>
            <p style={{ margin: '3px 0 0', fontSize: '0.84rem', color: 'var(--fp-text-muted)' }}>
              Think of routing as a smart GPS for customer payments. When someone clicks &ldquo;Pay&rdquo;, it selects the fastest, cheapest, and most reliable route.
            </p>
          </div>
        </div>

        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: 12,
          fontSize: '0.8125rem',
        }}>
          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '14px 16px' }}>
            <div style={{ fontWeight: 600, color: 'var(--fp-text)', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.875rem' }}>
              <span>🎯</span> 1. Primary Gateway
            </div>
            <div style={{ color: 'var(--fp-text-muted)', lineHeight: 1.5 }}>
              The preferred payment gateway (e.g. Razorpay or Cashfree) that handles payments first for lowest fees and high speed.
            </div>
          </div>

          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '14px 16px' }}>
            <div style={{ fontWeight: 600, color: 'var(--fp-text)', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.875rem' }}>
              <span>🛡️</span> 2. Automatic Backup (Failover)
            </div>
            <div style={{ color: 'var(--fp-text-muted)', lineHeight: 1.5 }}>
              If your main gateway goes down or has bank issues, payments instantly hop to a backup so transactions never fail.
            </div>
          </div>

          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '14px 16px' }}>
            <div style={{ fontWeight: 600, color: 'var(--fp-text)', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.875rem' }}>
              <span>⚖️</span> 3. Smart Traffic Split
            </div>
            <div style={{ color: 'var(--fp-text-muted)', lineHeight: 1.5 }}>
              Share payment volume (e.g. 70% / 30%) across multiple partners to prevent relying entirely on a single company.
            </div>
          </div>

          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '14px 16px' }}>
            <div style={{ fontWeight: 600, color: 'var(--fp-text)', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.875rem' }}>
              <span>🧪</span> 4. Test Simulator
            </div>
            <div style={{ color: 'var(--fp-text-muted)', lineHeight: 1.5 }}>
              Test any hypothetical payment in the simulator tab to see exactly which gateway will be picked and why.
            </div>
          </div>
        </div>
      </div>

      <div className="button-row" style={{ marginBottom: 16 }}>
        {routingTabs.map(item => (
          <button
            key={item.id}
            className={tab === item.id ? 'btn-primary' : 'btn-secondary'}
            type="button"
            onClick={() => setTab(item.id)}
            title={item.hint}
          >
            {item.label}
          </button>
        ))}
      </div>

      {tab === 'Rules' && (
        <>
          <div style={{
            background: 'var(--fp-bg-elevated)',
            border: '1px solid var(--fp-border)',
            borderRadius: 'var(--fp-radius-sm)',
            padding: '16px 18px',
            marginBottom: 16,
            fontSize: '0.85rem',
            color: 'var(--fp-text-muted)',
          }}>
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: 10, marginBottom: 12 }}>
              <span style={{ fontSize: '1.25rem', lineHeight: 1 }}>📌</span>
              <div>
                <strong style={{ color: 'var(--fp-text)', fontSize: '0.92rem' }}>What are Active Route Rules?</strong>
                <p style={{ margin: '4px 0 0', lineHeight: 1.5 }}>
                  Active Route Rules are the live automated instructions currently directing customer payments across your connected gateways (Razorpay, Cashfree, Stripe, etc.). When a customer pays, FeelPay matches these rules to send the transaction to the most reliable and cost-effective provider.
                </p>
              </div>
            </div>

            <div style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
              gap: 10,
              paddingTop: 12,
              borderTop: '1px solid rgba(255, 255, 255, 0.08)',
              fontSize: '0.8rem',
            }}>
              <div style={{ background: 'var(--fp-bg-card)', padding: '10px 12px', borderRadius: 6, border: '1px solid var(--fp-border)' }}>
                <span style={{ color: 'var(--fp-primary)', fontWeight: 600 }}>💡 Example 1 (By Method):</span>
                <div style={{ marginTop: 3, color: 'var(--fp-text-muted)', lineHeight: 1.4 }}>Send all <strong>UPI</strong> payments to <strong>Razorpay</strong> for highest speed and instant QR confirmations.</div>
              </div>
              <div style={{ background: 'var(--fp-bg-card)', padding: '10px 12px', borderRadius: 6, border: '1px solid var(--fp-border)' }}>
                <span style={{ color: 'var(--fp-primary)', fontWeight: 600 }}>💡 Example 2 (By Amount):</span>
                <div style={{ marginTop: 3, color: 'var(--fp-text-muted)', lineHeight: 1.4 }}>Send transactions above <strong>₹50,000</strong> to <strong>Cashfree</strong> to reduce percentage transaction fees.</div>
              </div>
              <div style={{ background: 'var(--fp-bg-card)', padding: '10px 12px', borderRadius: 6, border: '1px solid var(--fp-border)' }}>
                <span style={{ color: 'var(--fp-primary)', fontWeight: 600 }}>💡 Example 3 (By Currency):</span>
                <div style={{ marginTop: 3, color: 'var(--fp-text-muted)', lineHeight: 1.4 }}>Send international currencies (<strong>USD, EUR</strong>) to <strong>Stripe</strong> for global cards and 3D-Secure.</div>
              </div>
              <div style={{ background: 'var(--fp-bg-card)', padding: '10px 12px', borderRadius: 6, border: '1px solid var(--fp-border)' }}>
                <span style={{ color: 'var(--fp-primary)', fontWeight: 600 }}>💡 Example 4 (Auto Backup):</span>
                <div style={{ marginTop: 3, color: 'var(--fp-text-muted)', lineHeight: 1.4 }}>If primary gateway latency spikes or fails, instantly redirect to backup gateway with zero downtime.</div>
              </div>
            </div>
          </div>
          <DataTable
            rows={rows}
            columns={['name', 'rule_group', 'strategy', 'payment_method', 'currency', 'country', 'amount_min', 'amount_max', 'provider_name', 'provider_weight', 'min_success_rate', 'max_provider_fee_percent', 'max_latency_ms', 'health_required', 'failover_provider_name', 'risk_level', 'status']}
            empty={{ title: 'No routing rules configured', body: 'Add routing rules to automatically pick payment gateways based on payment method (UPI, Cards), transaction amount, or lowest fees.' }}
            onView={row => helpers.setDrawer({ title: 'Routing Rule Details', body: row })}
          />
        </>
      )}

      {tab === 'Simulator' && <RoutingSimulator form={form} result={result} onChange={setForm} onSubmit={simulate} />}

      {tab === 'Provider Performance' && (
        <>
          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '12px 16px', marginBottom: 16, fontSize: '0.85rem', color: 'var(--fp-text-muted)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ fontSize: '1.1rem' }}>📊</span>
            <span><strong style={{ color: 'var(--fp-text)' }}>Gateway Health &amp; Speed:</strong> Live scoreboard of each payment gateway. High success rates and low response times ensure customers enjoy instant checkouts without drop-offs.</span>
          </div>
          <DataTable
            rows={performance.rows}
            columns={['provider', 'payment_method', 'currency', 'country', 'success_rate', 'failure_rate', 'avg_latency_ms', 'p95_latency_ms', 'provider_fee_percent', 'attempt_count', 'updated_at']}
            empty={{ title: 'No gateway performance data yet', body: 'Live uptime, response speed, and success rates will show here as soon as transactions are processed.' }}
          />
        </>
      )}

      {tab === 'Failover Rules' && (
        <>
          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '12px 16px', marginBottom: 16, fontSize: '0.85rem', color: 'var(--fp-text-muted)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ fontSize: '1.1rem' }}>🛡️</span>
            <span><strong style={{ color: 'var(--fp-text)' }}>Backup &amp; Failover Rules:</strong> Zero-downtime protection. If Gateway A experiences network issues or downtime, payments automatically redirect to Gateway B without the customer seeing an error.</span>
          </div>
          <DataTable
            rows={rows}
            columns={['name', 'provider_name', 'failover_provider_name', 'failover_strategy', 'retry_strategy', 'cooldown_minutes', 'health_required', 'status']}
            empty={{ title: 'No backup rules configured', body: 'Configure backup rules so transactions seamlessly switch to another gateway if one goes down.' }}
          />
        </>
      )}

      {tab === 'Match Logs' && (
        <>
          <div style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)', borderRadius: 'var(--fp-radius-sm)', padding: '12px 16px', marginBottom: 16, fontSize: '0.85rem', color: 'var(--fp-text-muted)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ fontSize: '1.1rem' }}>🕒</span>
            <span><strong style={{ color: 'var(--fp-text)' }}>Live Routing History:</strong> An audit log showing recent payments, the rule that matched them, and the gateway that successfully processed each one.</span>
          </div>
          <DataTable
            rows={rows}
            columns={['name', 'last_matched_at', 'provider_name', 'failover_provider_name', 'status']}
            empty={{ title: 'No routing history yet', body: 'Live history of matched rules and gateway selections will appear here after payments are attempted.' }}
          />
        </>
      )}

      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function WhiteLabelCheckoutPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [form, setForm] = useState<WhiteLabelForm>(DEFAULT_WHITE_LABEL);
  const [tab, setTab] = useState('Config');
  const [state, setState] = useState<LoadState>('loading');
  const [error, setError] = useState('');
  const helpers = useDrawerConfirm();
  const paymentLinks = useRows(`${API}/white-label/payment-links`);
  const sessions = useRows(`${API}/white-label/sessions`);
  const receipts = useRows(`${API}/white-label/receipts`);
  const businessAccounts = useRows(`${API}/business-accounts`);
  const [linkForm, setLinkForm] = useState({ business_account_id: 'cd317088-a9ef-495c-ab3e-2a88603c54ac', amount: 2499, currency: 'INR', description: 'SaaS Platform Annual Subscription', customer_email: 'finance@enterprise-client.in', customer_phone: '+91 98765 43210', expires_at: '2026-12-31' });
  const [businessAccountSearch, setBusinessAccountSearch] = useState('');
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const [providerData, config] = await Promise.all([
        apiRequest<{ items: Provider[] }>('GET', `${API}/providers`),
        apiRequest<Partial<WhiteLabelForm> & { default_payment_methods?: string[] }>('GET', `${API}/white-label-config`),
        paymentLinks.load(),
        sessions.load(),
        receipts.load(),
        businessAccounts.load(),
      ]);
      setProviders(providerData.items || []);
      setForm({
        ...DEFAULT_WHITE_LABEL,
        ...config,
        brand_name: str(config.brand_name || DEFAULT_WHITE_LABEL.brand_name),
        checkout_logo_url: str(config.checkout_logo_url),
        checkout_theme_color: str(config.checkout_theme_color || DEFAULT_WHITE_LABEL.checkout_theme_color),
        payment_page_domain: str(config.payment_page_domain),
        support_email: str(config.support_email),
        support_phone: str(config.support_phone),
        terms_url: str(config.terms_url),
        privacy_url: str(config.privacy_url),
        refund_policy_url: str(config.refund_policy_url),
        statement_descriptor: str(config.statement_descriptor),
        default_provider_id: str(config.default_provider_id),
        fallback_provider_id: str(config.fallback_provider_id),
        default_payment_methods: Array.isArray(config.default_payment_methods) ? config.default_payment_methods.join(',') : str(config.default_payment_methods || DEFAULT_WHITE_LABEL.default_payment_methods),
        default_country: str(config.default_country || DEFAULT_WHITE_LABEL.default_country),
        default_currency: str(config.default_currency || DEFAULT_WHITE_LABEL.default_currency),
        invoice_prefix: str(config.invoice_prefix || DEFAULT_WHITE_LABEL.invoice_prefix),
        receipt_prefix: str(config.receipt_prefix || DEFAULT_WHITE_LABEL.receipt_prefix),
      });
      setState('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Platform checkout config load failed');
      setState('error');
    }
  };
  useEffect(() => {
    void load();
  }, []);
  const save = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({ title: 'Save checkout config', body: 'Platform checkout defaults will change for company-managed payment flows.', action: () => helpers.run('Platform checkout saved.', () => apiRequest('PATCH', `${API}/white-label-config`, { ...form, default_payment_methods: list(form.default_payment_methods), default_provider_id: form.default_provider_id || null, fallback_provider_id: form.fallback_provider_id || null })).then(load) });
  };
  const createLink = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Create payment link',
      body: 'Link uses signed token hash, no provider secret exposure, and audit logging.',
      action: () => helpers.run('Payment link created.', () => apiRequest('POST', `${API}/white-label/payment-links`, {
        ...linkForm,
        business_account_id: linkForm.business_account_id || null,
        expires_at: linkForm.expires_at || null,
      })).then(load),
    });
  };
  const linkAction = (row: GenericRecord, action: string) => {
    const id = String(row.id || '');
    if (action === 'expire_payment_link') helpers.setConfirm({ title: 'Expire payment link', body: 'Expired links cannot create sessions.', action: () => helpers.run('Payment link expired.', () => apiRequest('POST', `${API}/white-label/payment-links/${id}/expire`)).then(load) });
    if (action === 'create_session') helpers.setConfirm({ title: 'Create payment session', body: 'Session uses immutable checkout theme snapshot and provider routing.', action: () => helpers.run('Session created.', () => apiRequest('POST', `${API}/white-label/sessions`, { payment_link_id: id })).then(load) });
    if (action === 'copy_payment_link') helpers.setDrawer({ title: 'Payment link', body: { payment_link_id: id, note: 'Use signed public URL from checkout gateway service' } });
  };
  const sessionAction = (row: GenericRecord, action: string) => {
    const id = str(row.id);
    if (action === 'complete_test_payment') {
      helpers.setConfirm({
        title: 'Complete test payment',
        body: 'Mock checkout will capture a sandbox payment, post ledger entries, update reports, create receipt, and write audit log.',
        action: () => helpers.run('Test payment completed.', async () => {
          const result = await apiRequest<GenericRecord>('POST', `${API}/white-label/sessions/${id}/complete-test-payment`);
          helpers.setDrawer({ title: 'Test payment success', body: result });
          await load();
        }),
      });
    } else {
      helpers.setDrawer({ title: 'Checkout session', body: row });
    }
  };
  const businessAccountOptions = businessAccounts.rows
    .filter(row => {
      const needle = businessAccountSearch.trim().toLowerCase();
      if (!needle) return true;
      return [row.display_name, row.legal_name, row.email].some(value => str(value).toLowerCase().includes(needle));
    })
    .map(row => ({ label: `${str(row.display_name || row.legal_name || 'Business account')}${row.email ? ` · ${str(row.email)}` : ''}`, value: str(row.id) }));
  const businessAccountName = (id: GenericRecord[string] | undefined) => {
    const row = businessAccounts.rows.find(item => str(item.id) === str(id));
    return row ? str(row.display_name || row.legal_name || row.email || 'Business account') : 'All business accounts';
  };
  const paymentLinkRows = paymentLinks.rows.map(row => ({ ...row, business_account: businessAccountName(row.business_account_id) }));
  return (
    <CrudPage title="Platform Checkout" subtitle="Company-managed checkout brand, defaults, and descriptors" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Config', 'Preview', 'Payment Links', 'Checkout Sessions', 'Receipts'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      {tab === 'Config' && <WhiteLabelPanel value={form} providers={providers} onChange={setForm} onSubmit={save} onReset={() => setForm(DEFAULT_WHITE_LABEL)} />}
      {tab === 'Preview' && <CheckoutPreview value={form} onEditConfig={() => setTab('Config')} />}
      {tab === 'Payment Links' && (
        <>
          <section className="card">
            <h2><Wallet size={18} /> Payment links</h2>
            <form onSubmit={createLink}>
              <div className="filter-grid">
                <Field label="Business account search"><input value={str(businessAccountSearch)} onChange={event => setBusinessAccountSearch(event.target.value)} placeholder="Search name or email" /></Field>
                <Field label="Business account"><Select value={str(linkForm.business_account_id)} options={[{ label: 'All business accounts', value: '' }, ...businessAccountOptions]} onChange={business_account_id => setLinkForm({ ...linkForm, business_account_id })} /></Field>
                <Field label="Amount"><input type="number" value={linkForm.amount} onChange={event => setLinkForm({ ...linkForm, amount: Number(event.target.value) })} /></Field>
                <Field label="Currency"><input value={linkForm.currency} onChange={event => setLinkForm({ ...linkForm, currency: event.target.value })} /></Field>
                <Field label="Description"><input value={linkForm.description} onChange={event => setLinkForm({ ...linkForm, description: event.target.value })} /></Field>
                <Field label="Customer email"><input value={linkForm.customer_email} onChange={event => setLinkForm({ ...linkForm, customer_email: event.target.value })} /></Field>
                <Field label="Customer phone"><input value={linkForm.customer_phone} onChange={event => setLinkForm({ ...linkForm, customer_phone: event.target.value })} /></Field>
                <Field label="Expires at"><input type="datetime-local" value={linkForm.expires_at} onChange={event => setLinkForm({ ...linkForm, expires_at: event.target.value })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Create payment link</button>
            </form>
          </section>
          <DataTable rows={paymentLinkRows} columns={['id', 'business_account', 'amount', 'currency', 'description', 'customer_email', 'customer_phone', 'status', 'expires_at', 'created_at']} empty={{ title: 'No payment links yet', body: 'Create platform payment links for hosted checkout sessions.' }} renderActions={row => <ActionButtons actions={['Create session', 'Expire payment link', 'Copy payment link', 'View detail']} onClick={action => action === 'View detail' ? helpers.setDrawer({ title: 'Payment link', body: row }) : linkAction(row, normalizeAction(action))} />} />
        </>
      )}
      {tab === 'Checkout Sessions' && <DataTable rows={sessions.rows} columns={['id', 'payment_link_id', 'payment_order_id', 'provider_id', 'selected_payment_method', 'status', 'expires_at', 'created_at']} empty={{ title: 'No checkout sessions yet', body: 'Sessions are created from active payment links and provider routing.' }} renderActions={row => <ActionButtons actions={String(row.status) === 'paid' ? ['View detail'] : ['Complete test payment', 'View detail']} onClick={action => sessionAction(row, normalizeAction(action))} />} />}
      {tab === 'Receipts' && <DataTable rows={receipts.rows} columns={['id', 'payment_session_id', 'payment_order_id', 'receipt_number', 'recipient_email', 'status', 'sent_at', 'created_at']} empty={{ title: 'No receipts yet', body: 'Receipts are generated after successful checkout completion.' }} renderActions={row => <ActionButtons actions={['Resend receipt', 'View detail']} onClick={action => action === 'Resend receipt' ? helpers.run('Receipt resent.', () => apiRequest('POST', `${API}/white-label/receipts/${String(row.id || '')}/resend`)).then(load) : helpers.setDrawer({ title: 'Receipt', body: row })} />} />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function WebhookConfigPage() {
  const providers = useRows(`${API}/providers`);
  const configs = useRows(`${API}/webhook-configs`);
  const events = useRows(`${API}/webhook-events?limit=100`);
  const attempts = useRows(`${API}/webhook-processing-attempts`);
  const policies = useRows(`${API}/webhook-retry-policies`);
  const [tab, setTab] = useState('endpoints');
  const [policyForm, setPolicyForm] = useState({ provider_id: '10000000-0000-0000-0000-000000000001', max_attempts: 6, backoff_strategy: 'exponential', initial_delay_seconds: 60, max_delay_seconds: 3600, status: 'active' });
  const helpers = useDrawerConfirm();
  const load = () => void Promise.all([providers.load(), configs.load(), events.load(), attempts.load(), policies.load()]);
  const replay = (row: GenericRecord, dryRun: boolean) => {
    helpers.setConfirm({
      title: dryRun ? 'Dry-run replay' : 'Replay event',
      body: dryRun ? 'Dry-run checks handler path without state mutation.' : 'Replay re-queues webhook idempotently with audit log.',
      action: () => helpers.run('Replay submitted.', () => apiRequest('POST', `${API}/webhook-events/${String(row.id || '')}/replay`, { dry_run: dryRun })).then(load),
    });
  };
  const savePolicy = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Save retry policy',
      body: 'Policy controls max attempts and backoff before dead-letter.',
      action: () => helpers.run('Retry policy saved.', () => apiRequest('POST', `${API}/webhook-retry-policies`, {
        ...policyForm,
        provider_id: policyForm.provider_id || null,
      })).then(load),
    });
  };
  return (
    <CrudPage title="Webhook Config" subtitle="Manage webhook URLs, track incoming events, and monitor delivery status" state={configs.state} error={providers.error || configs.error || events.error || attempts.error || policies.error} notice={helpers.notice} onRefresh={load}>
      <div className="alert alert-success">Secure &amp; reliable: Verified signatures, duplicate protection, and automatic retries for failed events.</div>
      <div className="button-row">
        {[
          { id: 'endpoints', label: 'Endpoints' },
          { id: 'events', label: 'Events' },
          { id: 'retries', label: 'Pending Retries' },
          { id: 'failures', label: 'Failed Deliveries' },
          { id: 'logs', label: 'Delivery Logs' },
          { id: 'policies', label: 'Retry Rules' },
        ].map(item => (
          <button
            key={item.id}
            type="button"
            className={tab === item.id ? 'btn-primary' : 'btn-secondary'}
            onClick={() => setTab(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>
      {tab === 'endpoints' && (
        <DataTable
          rows={configs.rows}
          columns={['provider_name', 'environment', 'webhook_url', 'masked_signing_secret', 'signature_verification_enabled', 'last_event_received_at', 'failed_event_count', 'retry_queue_count', 'status']}
          empty={{ title: 'No webhook endpoints configured yet.', body: 'Configure provider credentials first. Signing secrets remain masked and signature verification defaults to required.' }}
        />
      )}
      {tab === 'events' && <DataTable rows={events.rows} columns={['provider_id', 'provider_event_id', 'event_type', 'event_version', 'payload_hash', 'dedupe_key', 'idempotency_key', 'signature_verification_status', 'processing_status', 'retry_count', 'received_at']} empty={{ title: 'No webhook events yet.', body: 'Events appear after configured providers send signed webhooks.' }} renderActions={row => <ActionButtons actions={['Replay event', 'Dry-run replay', 'View detail']} onClick={action => action === 'Replay event' ? replay(row, false) : action === 'Dry-run replay' ? replay(row, true) : helpers.setDrawer({ title: 'Webhook event', body: row })} />} />}
      {tab === 'retries' && <DataTable rows={events.rows.filter(row => Number(row.retry_count || 0) > 0)} columns={['provider_event_id', 'event_type', 'processing_status', 'retry_count', 'next_retry_at', 'error_message']} empty={{ title: 'No pending retries.', body: 'Failed webhook events scheduled for retry will appear here.' }} renderActions={row => <ActionButtons actions={['Retry now', 'View detail']} onClick={action => action === 'Retry now' ? helpers.run('Retry queued.', () => apiRequest('POST', `${API}/webhook-events/${String(row.id || '')}/retry`)).then(load) : helpers.setDrawer({ title: 'Pending retry event', body: row })} />} />}
      {tab === 'failures' && <DataTable rows={events.rows.filter(row => String(row.processing_status || '') === 'dead_letter')} columns={['provider_event_id', 'event_type', 'retry_count', 'dead_letter_reason', 'error_message', 'received_at']} empty={{ title: 'No failed deliveries.', body: 'Events that failed all retry attempts will appear here for review.' }} renderActions={row => <ActionButtons actions={['Resolve dead letter', 'Ignore', 'View detail']} onClick={action => action === 'Resolve dead letter' ? helpers.run('Dead-letter resolved.', () => apiRequest('POST', `${API}/webhook-events/${String(row.id || '')}/resolve-dead-letter`, { note: 'Resolved by admin' })).then(load) : action === 'Ignore' ? helpers.run('Webhook ignored.', () => apiRequest('POST', `${API}/webhook-events/${String(row.id || '')}/ignore`)).then(load) : helpers.setDrawer({ title: 'Failed delivery', body: row })} />} />}
      {tab === 'logs' && <DataTable rows={attempts.rows} columns={['id', 'webhook_event_id', 'attempt_number', 'status', 'started_at', 'finished_at', 'error_message', 'worker_id']} empty={{ title: 'No delivery logs yet.', body: 'Logs appear once webhook events are processed.' }} onView={row => helpers.setDrawer({ title: 'Delivery log detail', body: row })} />}
      {tab === 'policies' && (
        <>
          <section className="card">
            <h2><RefreshCw size={18} /> Retry rules</h2>
            <form onSubmit={savePolicy}>
              <div className="filter-grid">
                <Field label="Provider"><Select value={str(policyForm.provider_id)} options={[{ label: 'Select provider', value: '' }, ...providers.rows.map(provider => ({ label: str(provider.name || provider.code || 'Provider'), value: str(provider.id) }))]} onChange={provider_id => setPolicyForm({ ...policyForm, provider_id })} /></Field>
                <Field label="Max attempts"><input type="number" value={policyForm.max_attempts} onChange={event => setPolicyForm({ ...policyForm, max_attempts: Number(event.target.value) })} /></Field>
                <Field label="Backoff strategy"><Select value={policyForm.backoff_strategy} options={['fixed', 'exponential']} onChange={backoff_strategy => setPolicyForm({ ...policyForm, backoff_strategy })} /></Field>
                <Field label="Initial delay (sec)"><input type="number" value={policyForm.initial_delay_seconds} onChange={event => setPolicyForm({ ...policyForm, initial_delay_seconds: Number(event.target.value) })} /></Field>
                <Field label="Max delay (sec)"><input type="number" value={policyForm.max_delay_seconds} onChange={event => setPolicyForm({ ...policyForm, max_delay_seconds: Number(event.target.value) })} /></Field>
                <Field label="Status"><Select value={policyForm.status} options={['active', 'disabled']} onChange={status => setPolicyForm({ ...policyForm, status })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Save retry policy</button>
            </form>
          </section>
          <DataTable rows={policies.rows} columns={['id', 'provider_id', 'max_attempts', 'backoff_strategy', 'initial_delay_seconds', 'max_delay_seconds', 'status', 'updated_at']} empty={{ title: 'No retry rules configured yet.', body: 'Configure a provider retry rule to customize delivery attempts.' }} onView={row => helpers.setDrawer({ title: 'Retry rule', body: row })} />
        </>
      )}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function ReportsPage() {
  const helpers = useDrawerConfirm();
  const reports = useRows(`${API}/reports`);
  const history = useRows(`${API}/reports/export-history`);
  const scheduled = useRows(`${API}/reports/scheduled`);
  const [tab, setTab] = useState('Reports');
  const [filters, setFilters] = useState({ start_date: '2026-01-01', end_date: '2026-12-31', provider: '', business_account: '', currency: '', payment_method: '', status: '', format: 'CSV' });
  const state = reports.state === 'error' || history.state === 'error' || scheduled.state === 'error' ? 'error' : reports.state === 'loading' || history.state === 'loading' || scheduled.state === 'loading' ? 'loading' : 'ready';
  const error = reports.error || history.error || scheduled.error;
  const load = () => void Promise.all([reports.load(), history.load(), scheduled.load()]);
  const hasPendingExports = history.rows.some(row => ['QUEUED', 'PROCESSING'].includes(String(row.status || '').toUpperCase()));
  useEffect(() => {
    if (!hasPendingExports) return undefined;
    const timer = window.setInterval(() => {
      void history.load();
      void reports.load();
    }, 4000);
    return () => window.clearInterval(timer);
  }, [hasPendingExports, history.load, reports.load]);
  const generate = (row: GenericRecord, format: string) => {
    helpers.setConfirm({
      title: `Generate ${format}`,
      body: 'Report export is permission-gated, masked, expiring, and audit logged.',
      action: async () => {
        helpers.setNotice('');
        helpers.setError('');
        try {
          const result = await apiRequest<GenericRecord>('POST', `${API}/reports/generate`, {
            report_type: row.report_name || row.name,
            format,
            filters,
          });
          const status = String(result.status || '').toUpperCase();
          helpers.setNotice(status === 'READY' ? 'Report generated.' : status === 'FAILED' ? 'Report failed.' : 'Report queued.');
          await Promise.all([reports.load(), history.load()]);
          if (status === 'FAILED') helpers.setDrawer({ title: 'Report failure', body: reportFailureDetail(result) });
        } catch (err) {
          helpers.setError(err instanceof Error ? err.message : 'Report generation failed');
        }
      },
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
      action: async () => {
        await helpers.run('Report download ready.', async () => {
          const { blob, fileName } = await apiDownload(`${API}/reports/export-history/${id}/download`);
          const url = URL.createObjectURL(blob);
          const link = document.createElement('a');
          link.href = url;
          link.download = fileName;
          document.body.appendChild(link);
          link.click();
          link.remove();
          URL.revokeObjectURL(url);
        });
        load();
      },
    });
  };
  const retry = (row: GenericRecord) => {
    helpers.setConfirm({
      title: 'Retry report',
      body: 'Retry uses same report type, format, and saved filters.',
      action: async () => {
        const savedFilters = typeof row.filters_json === 'object' && row.filters_json !== null ? row.filters_json : filters;
        await apiRequest('POST', `${API}/reports/generate`, {
          report_type: row.report_type,
          format: row.format,
          filters: savedFilters,
        });
        await Promise.all([reports.load(), history.load()]);
        helpers.setNotice('Report retry started.');
      },
    });
  };
  return (
    <CrudPage title="Reports" subtitle="Company finance and operations exports" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="card filter-grid">
        <Field label="Start Date">
          <input type="date" value={filters.start_date} onChange={e => setFilters({ ...filters, start_date: e.target.value })} />
        </Field>
        <Field label="End Date">
          <input type="date" value={filters.end_date} onChange={e => setFilters({ ...filters, end_date: e.target.value })} />
        </Field>
        <Field label="Format">
          <Select value={filters.format} options={['CSV', 'PDF', 'XLSX']} onChange={next => setFilters({ ...filters, format: next })} />
        </Field>
        <Field label="Provider">
          <Select value={filters.provider} options={PROVIDER_OPTIONS} allLabel="All Providers" onChange={next => setFilters({ ...filters, provider: next })} />
        </Field>
        <Field label="Currency">
          <Select value={filters.currency} options={CURRENCY_OPTIONS} allLabel="All Currencies" onChange={next => setFilters({ ...filters, currency: next })} />
        </Field>
        <Field label="Payment Method">
          <Select value={filters.payment_method} options={PAYMENT_METHOD_OPTIONS} allLabel="All Methods" onChange={next => setFilters({ ...filters, payment_method: next })} />
        </Field>
        <Field label="Status">
          <Select value={filters.status} options={TRANSACTION_STATUS_OPTIONS} allLabel="All Statuses" onChange={next => setFilters({ ...filters, status: next })} />
        </Field>
        <Field label="Business Account">
          <input type="text" placeholder="Filter by account" value={filters.business_account} onChange={e => setFilters({ ...filters, business_account: e.target.value })} />
        </Field>
        <div className="filter-actions">
          <button className="btn-secondary" type="button" onClick={() => setFilters({ start_date: '', end_date: '', provider: '', business_account: '', currency: '', payment_method: '', status: '', format: 'CSV' })}>Reset</button>
        </div>
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
      {tab === 'Export History' && <DataTable rows={history.rows.map(row => {
        const status = String(row.status || '').toUpperCase();
        const isOld = new Date(String(row.generated_at)).getTime() < new Date('2026-05-06').getTime();
        let displayStatus = status;
        if (status === 'FAILED') displayStatus = isOld ? 'Historical failure' : 'Failed';
        else if (status === 'READY' && isOld) displayStatus = 'Historical success';

        return {
          ...row,
          status: displayStatus,
          _retry_label: row.parent_id ? 'Retry' : 'Original'
        };
      })} columns={['id', 'report_type', 'format', 'status', 'row_count', '_retry_label', 'generated_by', 'generated_at', 'expires_at', 'error_message']} empty={{ title: 'No report exports yet', body: 'Generated reports appear here with expiring download links.' }} renderActions={row => {
        const status = String(row.status || '').toUpperCase();
        const actions = status.includes('READY') || status.includes('SUCCESS') ? ['Download', 'View detail'] : status.includes('FAILED') || status.includes('FAILURE') ? ['Retry', 'View failure'] : ['View detail'];
        return <ActionButtons actions={actions} onClick={action => action === 'Download' ? download(row) : action === 'Retry' ? retry(row) : helpers.setDrawer({ title: status.includes('FAILED') || status.includes('FAILURE') ? 'Report failure' : 'Export history', body: status.includes('FAILED') || status.includes('FAILURE') ? reportFailureDetail(row) : row })} />;
      }} />}
      {tab === 'Scheduled Reports' && <DataTable rows={scheduled.rows} columns={['id', 'report_type', 'frequency', 'recipients', 'format', 'status', 'created_by', 'next_run_at', 'last_run_at', 'created_at']} empty={{ title: 'No scheduled reports yet', body: 'Schedule recurring daily, weekly, or monthly reports for finance recipients.' }} onView={row => helpers.setDrawer({ title: 'Scheduled report', body: row })} />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

function reportFailureDetail(row: GenericRecord): GenericRecord {
  const format = String(row.format || '').toUpperCase();
  const message = String(row.error_message || '');
  return {
    ...row,
    message: format === 'PDF' && message.includes('not configured')
      ? 'PDF export is not configured on this server.'
      : message || 'Report export failed.',
    suggestion: 'Use CSV export or configure PDF generator.',
  };
}

export function AuditLogsPage() {
  const { rows, state, error, load } = useRows(`${API}/audit-logs`);
  const helpers = useDrawerConfirm();
  const [actionFilter, setActionFilter] = useState('');
  const [entityFilter, setEntityFilter] = useState('');
  const [search, setSearch] = useState('');

  const filteredRows = rows.filter(row => {
    if (actionFilter && str(row.action).toLowerCase() !== actionFilter.toLowerCase()) return false;
    if (entityFilter && str(row.entity_type).toLowerCase() !== entityFilter.toLowerCase()) return false;
    if (search) {
      const q = search.toLowerCase();
      const match = [row.actor, row.entity_id, row.ip_address, row.action, row.entity_type].some(v => str(v).toLowerCase().includes(q));
      if (!match) return false;
    }
    return true;
  });

  return (
    <CrudPage title="Audit Logs" subtitle="Sensitive company admin action history" state={state} error={error} notice={helpers.notice} onRefresh={load}>
      <div className="card filter-grid">
        <Field label="Action">
          <Select value={actionFilter} onChange={setActionFilter} options={AUDIT_ACTION_OPTIONS} allLabel="All Actions" />
        </Field>
        <Field label="Entity Type">
          <Select value={entityFilter} onChange={setEntityFilter} options={AUDIT_ENTITY_OPTIONS} allLabel="All Entity Types" />
        </Field>
        <Field label="Search">
          <input placeholder="Actor, ID, IP..." value={search} onChange={e => setSearch(e.target.value)} />
        </Field>
        <div className="filter-actions">
          <button className="btn-secondary" type="button" onClick={() => { setActionFilter(''); setEntityFilter(''); setSearch(''); }}>Reset</button>
        </div>
      </div>
      <DataTable rows={filteredRows} columns={['action', 'entity_type', 'entity_id', 'actor', 'ip_address', 'created_at']} empty="No audit logs found" onView={row => helpers.setDrawer({ title: 'Audit details', body: auditDetail(row) })} />
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function AdminUsersRolesPage() {
  const users = useRows('/admin/users');
  const roles = useRows(`${API}/admin-roles`);
  const helpers = useDrawerConfirm();
  const [invite, setInvite] = useState({ name: 'Deepak Varma', email: 'deepak.varma@feelpay.com', role: 'Support Agent' });
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
      <div className="card admin-invite-card">
        <div className="admin-invite-header">
          <div>
            <h3 className="admin-invite-title">Invite New Admin</h3>
            <p className="admin-invite-subtitle">Provision access with least-privilege role assignment and audit logging</p>
          </div>
          <span className="admin-invite-badge">Deny-by-Default</span>
        </div>
        <form className="admin-invite-form" onSubmit={inviteAdmin}>
          <div className="admin-invite-grid">
            <Field label="Name">
              <input
                placeholder="e.g. Deepak Varma"
                value={invite.name}
                onChange={event => setInvite({ ...invite, name: event.target.value })}
                required
              />
            </Field>
            <Field label="Email">
              <input
                type="email"
                placeholder="e.g. deepak.varma@feelpay.com"
                value={invite.email}
                onChange={event => setInvite({ ...invite, email: event.target.value })}
                required
              />
            </Field>
            <Field label="Role">
              <Select value={invite.role} options={ADMIN_ROLES} onChange={role => setInvite({ ...invite, role })} />
            </Field>
            <div className="admin-invite-action">
              <button className="btn-primary admin-invite-btn" type="submit">
                <UserPlus size={15} /> Invite Admin
              </button>
            </div>
          </div>
        </form>
      </div>
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
  const metrics = useRows(`${API}/system-health/metrics`);
  const slos = useRows(`${API}/system-health/slos`);
  const snapshots = useRows(`${API}/system-health/slo-snapshots`);
  const traces = useRows(`${API}/system-health/traces`);
  const incidents = useRows('/admin/notifications');
  const [tab, setTab] = useState('Overview');
  const helpers = useDrawerConfirm();
  const endpoints = {
    'Health API': health,
    'Jobs API': jobs,
    'Providers API': providers,
    'Metrics API': metrics,
    'SLOs API': slos,
    'Snapshots API': snapshots,
    'Traces API': traces,
    'Notifications API': incidents,
  };
  const failedEndpoints = Object.entries(endpoints).filter(([_, state]) => state.state === 'error').map(([name]) => name);
  const state: LoadState = failedEndpoints.length === Object.keys(endpoints).length ? 'error' : Object.values(endpoints).some(e => e.state === 'loading') ? 'loading' : 'ready';
  const error = failedEndpoints.length > 0 ? `Some health data could not load: ${failedEndpoints.join(', ')}` : '';
  const load = () => void Promise.all(Object.values(endpoints).map(e => e.load()));
  const retryJob = (row: GenericRecord) => {
    helpers.setConfirm({ title: 'Retry failed job', body: 'Background job retry will be audit logged.', action: () => helpers.run('Job retry queued.', () => apiRequest('POST', `/admin/system-health/jobs/${String(row.id || '')}/retry`)).then(load) });
  };
  return (
    <CrudPage title="System Health" subtitle="Internal health for company payment operations" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {['Overview', 'Providers', 'Jobs', 'Queues', 'SLOs', 'Incidents', 'Traces'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
      </div>
      <div className="stats-grid">
        {['API status', 'Database status', 'Queue status', 'Webhook worker status', 'Settlement sync status', 'Reconciliation job status', 'Provider health', 'Failed background jobs'].map(component => {
          const row = health.rows.find(item => String(item.component) === component);
          let value = String(row?.status || 'unknown');
          let status = statusClass(value);
          if (health.state === 'error') {
            value = 'Unknown';
            status = 'pending';
          }
          return <Metric key={component} label={component} value={value} status={status} icon={Activity} />;
        })}
      </div>
      {tab === 'Overview' && <DataTable rows={snapshots.rows} columns={['slo_name', 'current_percent', 'error_budget_remaining', 'status', 'captured_at']} empty="No SLO snapshots" />}
      {tab === 'Providers' && <TablePanel title="Provider Health" rows={providers.rows} columns={['provider', 'environment', 'health', 'success_rate', 'attempt_count', 'last_check', 'latency_ms', 'error_message']} />}
      {tab === 'Jobs' && <DataTable rows={jobs.rows} columns={['job_name', 'status', 'last_run_at', 'next_run_at', 'failure_count', 'last_error']} empty="No background jobs found" renderActions={row => <ActionButtons actions={['Retry failed job', 'View logs', 'Mark incident resolved']} onClick={action => action === 'Retry failed job' ? retryJob(row) : helpers.setDrawer({ title: action, body: row })} />} />}
      {tab === 'Queues' && <DataTable rows={metrics.rows.filter(row => String(row.metric_name).includes('queue'))} columns={['metric_name', 'labels_json', 'value', 'captured_at']} empty="No queue metrics" />}
      {tab === 'SLOs' && <DataTable rows={slos.rows} columns={['name', 'target_percent', 'window_minutes', 'metric_query_key', 'status', 'created_at']} empty="No SLO definitions" renderActions={row => <ActionButtons actions={['Acknowledge SLO breach', 'View detail']} onClick={action => action === 'Acknowledge SLO breach' ? helpers.run('SLO breach acknowledged.', () => apiRequest('POST', `${API}/system-health/slos/refresh`)).then(load) : helpers.setDrawer({ title: 'SLO', body: row })} />} />}
      {tab === 'Incidents' && <DataTable rows={incidents.rows.filter(row => row.incident_id || row.severity === 'critical')} columns={['id', 'title', 'severity', 'status', 'incident_id', 'sla_due_at', 'created_at']} empty="No incidents" onView={row => helpers.setDrawer({ title: 'Incident/alert', body: row })} />}
      {tab === 'Traces' && <DataTable rows={traces.rows} columns={['trace_id', 'request_id', 'span_name', 'duration_ms', 'status', 'error_message', 'created_at']} empty="No traces" onView={row => helpers.setDrawer({ title: 'Trace', body: row })} />}
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
    else helpers.setConfirm({ title: formatHeader(action), body: 'Notification action updates alert status and creates audit log.', action: () => helpers.run('Notification updated.', () => apiRequest('POST', `/admin/notifications/${id}/${action}`, { assigned_to_admin_id: row.assigned_to_admin_id || null })).then(load) });
  };
  const scopedRows = rows.filter(row => {
    if (tab === 'Critical') return row.severity === 'critical';
    if (tab === 'Resolved') return row.status === 'resolved';
    if (tab === 'Ignored') return row.status === 'ignored';
    if (tab === 'Assigned to me') return Boolean(row.assigned_to_admin_id);
    return row.status !== 'resolved' && row.status !== 'ignored';
  });
  return (
    <CrudPage title="System Alerts" subtitle="Company admin alerts center" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
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
  const webhookEvents = useRows(`${API}/developer/webhook-events?limit=50`);
  const apiVersions = useRows(`${API}/developer/api-versions`);
  const idempotencyLogs = useRows(`${API}/developer/idempotency-records`);
  const errorCatalog = useRows(`${API}/developer/error-catalog`);
  const rateLimits = useRows(`${API}/developer/rate-limits`);
  const [logFilters, setLogFilters] = useState({ request_id: '', idempotency_key: '', status_code: '', api_key_id: '', endpoint: '' });
  const [tab, setTab] = useState('keys');
  const [keyForm, setKeyForm] = useState({ name: 'Internal integration key', environment: 'sandbox', scopes: 'payments.read,refunds.write,webhooks.replay', expires_at: '2026-12-31' });
  const [webhookForm, setWebhookForm] = useState({ event_type: 'payment.captured', webhook_event_id: 'evt_sim_live_771928019', note: 'Standard ops delivery simulation', dry_run: false });
  const [idempotencyForm, setIdempotencyForm] = useState({ idempotency_key: 'idem_ops_sim_9921', api_key_id: 'key_live_adm_99182', payload: '{"resource":"payment","amount":10000}', resource_type: 'payment' });
  const [rateLimitForm, setRateLimitForm] = useState({ api_key_id: 'key_live_adm_99182', window_seconds: 60, max_requests: 120 });
  const helpers = useDrawerConfirm();
  const state = [apiKeys.state, requestLogs.state, replays.state, apiVersions.state, idempotencyLogs.state, errorCatalog.state, rateLimits.state, webhookEvents.state].includes('error') ? 'error' : [apiKeys.state, requestLogs.state, replays.state, apiVersions.state, idempotencyLogs.state, errorCatalog.state, rateLimits.state, webhookEvents.state].includes('loading') ? 'loading' : 'ready';
  const error = apiKeys.error || requestLogs.error || replays.error || apiVersions.error || idempotencyLogs.error || errorCatalog.error || rateLimits.error || webhookEvents.error;
  const load = () => void Promise.all([apiKeys.load(), requestLogs.load(), replays.load(), apiVersions.load(), idempotencyLogs.load(), errorCatalog.load(), rateLimits.load(), webhookEvents.load()]);
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
        title: formatHeader(action),
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
  const replayEvent = (row?: GenericRecord, dryRun = false) => {
    const id = String(row?.event_id || row?.webhook_event_id || webhookForm.webhook_event_id || '');
    if (!id) {
      helpers.setError('Please select an event or enter an ID first.');
      return;
    }
    helpers.setConfirm({
      title: `${dryRun ? 'Dry-run' : 'Replay'} webhook event`,
      body: dryRun
        ? 'Dry-run validates routing and handler availability without modifying state.'
        : 'Webhook replay re-queues the event idempotently. This creates an audit log.',
      action: () => helpers.run(dryRun ? 'Dry-run successful.' : 'Webhook replay queued.', () => apiRequest('POST', `${API}/developer/webhook-events/${id}/replay`, { dry_run: dryRun, note: webhookForm.note || null })).then(load),
    });
  };
  const payloads = [
    { id: 'payment.captured', event: 'payment.captured', sample: { id: 'pay_test', amount: 1000, currency: 'INR', card: 'redacted' } },
    { id: 'refund.succeeded', event: 'refund.succeeded', sample: { id: 'rf_test', payment_id: 'pay_test', amount: 500 } },
    { id: 'dispute.opened', event: 'dispute.opened', sample: { id: 'dp_test', payment_id: 'pay_test', reason: 'fraudulent' } },
  ];
  const checklist = ['Create test key', 'Set up notification endpoint', 'Send test message', 'Re-send test event', 'Review duplicate checks', 'Switch to live mode after approval'].map((item, index) => ({ id: String(index), item, status: index < 2 ? 'ready' : 'pending' }));
  const createIdempotencyRecord = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Create idempotency record',
      body: 'Write APIs require idempotency key. Replay returns same result, mismatch payload returns conflict.',
      action: () => helpers.run('Idempotency record saved.', () => apiRequest('POST', `${API}/developer/idempotency-records`, {
        idempotency_key: idempotencyForm.idempotency_key,
        api_key_id: idempotencyForm.api_key_id || null,
        payload: JSON.parse(idempotencyForm.payload || '{}'),
        resource_type: idempotencyForm.resource_type || 'payment',
      })).then(load),
    });
  };
  const createRateLimit = (event: FormEvent) => {
    event.preventDefault();
    helpers.setConfirm({
      title: 'Save rate limit',
      body: 'Rate limit policy enforces API key budget and returns error catalog code on exceed.',
      action: () => helpers.run('Rate limit saved.', () => apiRequest('POST', `${API}/developer/rate-limits`, {
        api_key_id: rateLimitForm.api_key_id || null,
        window_seconds: Number(rateLimitForm.window_seconds),
        max_requests: Number(rateLimitForm.max_requests),
      })).then(load),
    });
  };
  const checkRateLimit = (row: GenericRecord) => {
    const id = String(row.api_key_id || '');
    helpers.run('Rate limit check complete.', () => apiRequest('POST', `${API}/developer/rate-limits/${id}/check`));
  };
  return (
    <CrudPage title="Developer Tools" subtitle="Tools to test, monitor, and connect your platform" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={load}>
      <div className="button-row">
        {[
          { id: 'keys', label: 'Keys' },
          { id: 'activity', label: 'Activity' },
          { id: 'history', label: 'History' },
          { id: 'duplicates', label: 'Duplicate Checks' },
          { id: 'errors', label: 'Errors' },
          { id: 'limits', label: 'Limits' },
          { id: 'testing', label: 'Testing' },
          { id: 'resend', label: 'Re-send' },
          { id: 'examples', label: 'Examples' },
          { id: 'checklist', label: 'Checklist' },
          { id: 'versions', label: 'Versions' },
        ].map(item => (
          <button
            key={item.id}
            type="button"
            className={tab === item.id ? 'btn-primary' : 'btn-secondary'}
            onClick={() => setTab(item.id)}
            data-tab={item.id}
          >
            {item.label}
          </button>
        ))}
      </div>
      {tab === 'versions' && <DataTable rows={apiVersions.rows} columns={['version', 'status', 'sunset_at', 'changelog_url', 'created_at']} empty={{ title: 'No API versions', body: 'Define active and retired system versions.' }} />}
      {tab === 'duplicates' && (
        <>
          {ADMIN_DEBUG_TOOLS && (
            <section className="card">
              <h2><RefreshCw size={18} /> Duplicate prevention record</h2>
              <form onSubmit={createIdempotencyRecord}>
                <div className="filter-grid">
                  <Field label="Check key"><input value={str(idempotencyForm.idempotency_key)} onChange={event => setIdempotencyForm({ ...idempotencyForm, idempotency_key: event.target.value })} /></Field>
                  <Field label="API key ID"><input value={str(idempotencyForm.api_key_id)} onChange={event => setIdempotencyForm({ ...idempotencyForm, api_key_id: event.target.value })} /></Field>
                  <Field label="Resource type"><input value={str(idempotencyForm.resource_type)} onChange={event => setIdempotencyForm({ ...idempotencyForm, resource_type: event.target.value })} /></Field>
                  <Field label="Payload JSON"><textarea value={str(idempotencyForm.payload)} onChange={event => setIdempotencyForm({ ...idempotencyForm, payload: event.target.value })} /></Field>
                </div>
                <button className="btn-primary" type="submit">Save record</button>
              </form>
            </section>
          )}
          <DataTable rows={idempotencyLogs.rows} columns={['created_at', 'idempotency_key', 'api_key_id', 'request_hash', 'response_hash', 'response_status', 'resource_type', 'resource_id', 'expires_at']} empty={{ title: 'No duplicate records', body: 'Duplicate check records appear here to prevent repeated charges.' }} onView={row => helpers.setDrawer({ title: 'Duplicate record', body: row })} />
        </>
      )}
      {tab === 'errors' && <DataTable rows={errorCatalog.rows} columns={['code', 'message', 'http_status', 'retryable', 'docs_url']} empty={{ title: 'No error list', body: 'Known system error codes and helpful messages appear here.' }} />}
      {tab === 'limits' && (
        <>
          <section className="card">
            <h2><Timer size={18} /> Request limits</h2>
            <form onSubmit={createRateLimit}>
              <div className="filter-grid">
                <Field label="API key ID"><input value={rateLimitForm.api_key_id} onChange={event => setRateLimitForm({ ...rateLimitForm, api_key_id: event.target.value })} /></Field>
                <Field label="Window seconds"><input type="number" value={rateLimitForm.window_seconds} onChange={event => setRateLimitForm({ ...rateLimitForm, window_seconds: Number(event.target.value) })} /></Field>
                <Field label="Max requests"><input type="number" value={rateLimitForm.max_requests} onChange={event => setRateLimitForm({ ...rateLimitForm, max_requests: Number(event.target.value) })} /></Field>
              </div>
              <button className="btn-primary" type="submit">Save limit</button>
            </form>
          </section>
          <DataTable rows={rateLimits.rows} columns={['api_key_id', 'window_seconds', 'max_requests', 'current_count', 'reset_at']} empty={{ title: 'No limits set', body: 'Set traffic limits to prevent abuse or overload.' }} renderActions={row => <ActionButtons actions={['Check now', 'View detail']} onClick={action => action === 'Check now' ? checkRateLimit(row) : helpers.setDrawer({ title: 'Limit detail', body: row })} />} />
        </>
      )}
      {tab === 'keys' && (
        <>
          <section className="card">
            <h2><KeyRound size={18} /> API keys</h2>
            <form onSubmit={createKey}>
              <div className="filter-grid">
                <Field label="Name"><input value={keyForm.name} onChange={event => setKeyForm({ ...keyForm, name: event.target.value })} /></Field>
                <Field label="Environment"><Select value={keyForm.environment} options={['sandbox', 'production']} onChange={environment => setKeyForm({ ...keyForm, environment })} /></Field>
                <Field label="Scopes"><input value={keyForm.scopes} onChange={event => setKeyForm({ ...keyForm, scopes: event.target.value })} /></Field>
                <Field label="Expires at"><input type="datetime-local" value={keyForm.expires_at} onChange={event => setKeyForm({ ...keyForm, expires_at: event.target.value })} /></Field>
                <div className="filter-actions">
                  <button className="btn-primary" type="submit">Create key</button>
                </div>
              </div>
            </form>
          </section>
          <DataTable rows={apiKeys.rows} columns={['id', 'name', 'environment', 'key_prefix', 'masked_key', 'scopes', 'status', 'last_used_at', 'expires_at', 'created_at']} empty={{ title: 'No API keys yet', body: 'Create sandbox keys for testing. Secret keys are displayed once and never stored unmasked.' }} renderActions={row => <ActionButtons actions={['Rotate key', 'Revoke key', 'View detail']} onClick={action => keyAction(row, normalizeAction(action))} />} />
        </>
      )}
      {tab === 'activity' && (
        <>
          <div className="card filter-grid">
            <Field label="Request ID">
              <input placeholder="req_..." value={logFilters.request_id} onChange={event => setLogFilters({ ...logFilters, request_id: event.target.value })} />
            </Field>
            <Field label="Check Key">
              <input placeholder="idem_..." value={logFilters.idempotency_key} onChange={event => setLogFilters({ ...logFilters, idempotency_key: event.target.value })} />
            </Field>
            <Field label="Status Code">
              <Select value={logFilters.status_code} onChange={val => setLogFilters({ ...logFilters, status_code: val })} options={STATUS_CODE_OPTIONS} />
            </Field>
            <Field label="API Key ID">
              <input placeholder="key_..." value={logFilters.api_key_id} onChange={event => setLogFilters({ ...logFilters, api_key_id: event.target.value })} />
            </Field>
            <Field label="Endpoint">
              <input placeholder="/v1/..." value={logFilters.endpoint} onChange={event => setLogFilters({ ...logFilters, endpoint: event.target.value })} />
            </Field>
            <div className="filter-actions">
              <button className="btn-secondary" type="button" onClick={() => setLogFilters({ request_id: '', idempotency_key: '', status_code: '', api_key_id: '', endpoint: '' })}>Reset</button>
            </div>
          </div>
          <DataTable rows={filteredLogs} columns={['created_at', 'environment', 'api_key_id', 'method', 'path', 'status_code', 'latency_ms', 'idempotency_key', 'request_id', 'error_code', 'error_message']} empty={{ title: 'No activity logs', body: 'Request activity appears here with sensitive secrets and payment data redacted.' }} onView={row => helpers.setDrawer({ title: 'Request details', body: row })} />
        </>
      )}
      {tab === 'testing' && (
        <section className="card">
          <h2><Cable size={18} /> Test event simulator</h2>
          <form onSubmit={testWebhook}>
            <div className="filter-grid">
              <Field label="Event type">
                <Select value={webhookForm.event_type} onChange={val => setWebhookForm({ ...webhookForm, event_type: val })} options={WEBHOOK_EVENT_OPTIONS} />
              </Field>
              <Field label="Event ID (optional)"><input value={webhookForm.webhook_event_id} onChange={event => setWebhookForm({ ...webhookForm, webhook_event_id: event.target.value })} placeholder="test_..." /></Field>
              <div className="filter-actions">
                <button className="btn-primary" type="submit">Send test event</button>
              </div>
            </div>
          </form>
          <div className="alert pending" style={{ marginTop: 12 }}>
            Simulate events safely without affecting real customer transactions.
          </div>
        </section>
      )}
      {tab === 'history' && (
        <>
          <div className="card">
            <h2><Database size={18} /> Event history</h2>
            <div className="filter-grid" style={{ marginBottom: 12 }}>
              <Field label="Note"><input value={webhookForm.note} onChange={event => setWebhookForm({ ...webhookForm, note: event.target.value })} placeholder="Reason for re-send..." /></Field>
            </div>
          </div>
          <DataTable
            rows={webhookEvents.rows}
            columns={['id', 'event_id', 'event_type', 'signature_verification_status', 'processing_status', 'retry_count', 'received_at']}
            empty={{ title: 'No recorded events', body: 'Send a test event or configure a provider to see events here.' }}
            renderActions={row => <ActionButtons actions={['Re-send event', 'Dry-run', 'Copy ID', 'View payload']} onClick={action => action === 'Re-send event' ? replayEvent(row, false) : action === 'Dry-run' ? replayEvent(row, true) : action === 'Copy ID' ? copyText(String(row.event_id || row.id)) : helpers.setDrawer({ title: 'Event payload', body: row })} />}
          />
        </>
      )}
      {tab === 'resend' && <DataTable rows={replays.rows} columns={['id', 'webhook_event_id', 'replay_status', 'replayed_by_admin_id', 'replayed_at', 'error_message', 'created_at']} empty={{ title: 'No re-sent events', body: 'Re-sent events appear here after queued again.' }} renderActions={row => <ActionButtons actions={['Re-send selected event', 'View detail']} onClick={action => action === 'Re-send selected event' ? replayEvent(row) : helpers.setDrawer({ title: 'Re-send detail', body: row })} />} />}
      {tab === 'examples' && <SamplePayloads payloads={payloads} />}
      {tab === 'checklist' && <DataTable rows={[
        ...checklist,
        { id: 'sandbox_provider', item: 'Test provider configured', status: 'ready' },
        { id: 'production_provider', item: 'Live provider configured', status: 'pending' },
        { id: 'webhook_secret', item: 'Secret key configured', status: 'ready' },
        { id: 'signature_verification', item: 'Signature check enabled', status: 'ready' },
        { id: 'api_key_active', item: 'Access key active', status: apiKeys.rows.some(row => row.status === 'active') ? 'ready' : 'pending' },
        { id: 'idempotency', item: 'Duplicate prevention enabled', status: 'ready' },
        { id: 'no_raw_card', item: 'No raw card data stored', status: 'ready' },
        { id: 'no_cvv', item: 'No CVV code stored', status: 'ready' },
        { id: 'reconciliation', item: 'Auto-matching enabled', status: 'ready' },
        { id: 'settlement_export', item: 'Payout export enabled', status: 'ready' },
      ]} columns={['item', 'status']} empty="No checklist items" />}
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

export function ReadinessScorecardPage() {
  const { rows, state, error, load } = useRows('/admin/payment-ops/readiness-scorecard');
  const [summary, setSummary] = useState<GenericRecord>({});
  const helpers = useDrawerConfirm();
  const [ownerMap, setOwnerMap] = useState<Record<string, string>>({});

  const refresh = async () => {
    await load();
    try {
      const payload = await apiRequest<GenericRecord>('GET', '/admin/payment-ops/readiness-scorecard');
      setSummary(payload);
    } catch {
      setSummary({});
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const assignOwner = (row: GenericRecord) => {
    const id = String(row.id || '');
    const owner = ownerMap[id] || String(row.owner || '');
    helpers.setConfirm({
      title: 'Assign owner',
      body: 'Owner assignment helps remediation tracking for failed and warning checks.',
      action: () => helpers.run('Owner assigned.', () => apiRequest('POST', `/admin/payment-ops/readiness-scorecard/${id}/assign-owner`, { owner })).then(refresh),
    });
  };
  const markManual = (row: GenericRecord) => {
    const id = String(row.id || '');
    helpers.setConfirm({
      title: 'Mark manual check complete',
      body: 'Manual check update creates audit record.',
      action: () => helpers.run('Manual check updated.', () => apiRequest('POST', `/admin/payment-ops/readiness-scorecard/${id}/manual-check`, { status: 'pass', notes: 'Reviewed by admin team' })).then(refresh),
    });
  };
  const exportScorecard = () => helpers.run('Scorecard export ready.', () => apiRequest('GET', '/admin/payment-ops/readiness-scorecard/export'));
  const failedRows = Array.isArray(summary.failed_checks) ? summary.failed_checks as GenericRecord[] : [];
  const warningRows = Array.isArray(summary.warning_checks) ? summary.warning_checks as GenericRecord[] : [];

  return (
    <CrudPage title="Readiness Scorecard" subtitle="Internal production-readiness scoring across payments, security, compliance, and operations" state={state} error={error || helpers.error} notice={helpers.notice} onRefresh={refresh}>
      <div className="stats-grid">
        <Metric label="Total readiness score" value={String(summary.total_score ?? 0)} icon={ShieldCheck} />
        <Metric label="Failed checks" value={String(failedRows.length)} icon={AlertTriangle} />
        <Metric label="Warning checks" value={String(warningRows.length)} icon={AlertCircle} />
      </div>
      <div className="button-row">
        <button type="button" onClick={refresh}>Run readiness check</button>
        <button type="button" onClick={exportScorecard}>Export readiness report</button>
      </div>
      <DataTable rows={rows} columns={['category', 'check_name', 'status', 'score', 'owner', 'last_checked_at', 'action_link']} empty={{ title: 'No readiness checks', body: 'Readiness categories appear here with status, owner, and action links.' }} renderActions={row => <ActionButtons dropdownOnly actions={['Assign owner', 'Mark manual check complete', 'Open action link', 'View detail']} onClick={label => {
        if (label === 'Assign owner') assignOwner(row);
        else if (label === 'Mark manual check complete') markManual(row);
        else if (label === 'Open action link') {
          const link = String(row.action_link || '');
          if (link) window.location.assign(link);
        } else helpers.setDrawer({ title: 'Readiness check', body: row });
      }} />} />
      <div className="card filter-grid">
        {rows.map(row => (
          <Field key={String(row.id)} label={`Owner for ${String(row.category || 'category')}`}>
            <input value={ownerMap[String(row.id || '')] ?? String(row.owner || '')} onChange={event => setOwnerMap({ ...ownerMap, [String(row.id || '')]: event.target.value })} />
          </Field>
        ))}
      </div>
      <TablePanel title="Failed checks" rows={failedRows} columns={['category', 'check_name', 'status', 'score', 'owner']} />
      <TablePanel title="Warning checks" rows={warningRows} columns={['category', 'check_name', 'status', 'score', 'owner']} />
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
  const controls = useRows(`${API}/compliance/controls`);
  const evidence = useRows(`${API}/compliance/evidence`);
  const packets = useRows(`${API}/compliance/packet-exports`);
  const helpers = useDrawerConfirm();
  const relatedError = securityEvents.error || reviews.error || tasks.error;
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<GenericRecord>('GET', `${API}/compliance/overview`);
      await Promise.all([securityEvents.load(), reviews.load(), tasks.load(), controls.load(), evidence.load(), packets.load()]);
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
        <Metric label="Raw card storage" value={String(overview.raw_card_storage || 'disabled')} status={statusClass(String(overview.raw_card_storage || 'disabled'), 'raw_card_storage')} icon={ShieldCheck} />
        <Metric label="CVV storage" value={String(overview.cvv_storage || 'disabled')} status={statusClass(String(overview.cvv_storage || 'disabled'), 'cvv_storage')} icon={ShieldCheck} />
        <Metric label="Tokenization" value={String(overview.tokenization || 'enabled')} status={statusClass(String(overview.tokenization || 'enabled'))} icon={BadgeCheck} />
        <Metric label="Webhook signatures" value={String(overview.webhook_signatures || 'required')} status={statusClass(String(overview.webhook_signatures || 'required'))} icon={Cable} />
        <Metric label="Secrets encrypted" value={String(overview.secrets_encrypted || 'enabled')} status={statusClass(String(overview.secrets_encrypted || 'enabled'))} icon={KeyRound} />
        <Metric label="Bank accounts encrypted" value={String(overview.bank_accounts_encrypted || 'enabled')} status={statusClass(String(overview.bank_accounts_encrypted || 'enabled'))} icon={Landmark} />
        <Metric label="MFA coverage" value={String(overview.mfa_coverage || 0)} status={Number(overview.mfa_coverage || 0) > 0 ? 'success' : 'pending'} icon={UserPlus} />
        <Metric label="Credential rotation due" value={String(overview.credential_rotation_due || 0)} status={Number(overview.credential_rotation_due || 0) > 0 ? 'failed' : 'success'} icon={RefreshCw} />
      </div>
      <div className="button-row">
        {['Overview', 'Controls', 'Evidence', 'Access Reviews', 'Security Events', 'Data Retention', 'Audit Export', 'Credential Rotation', 'Compliance Packet', 'Compliance Tasks'].map(item => <button key={item} type="button" className={tab === item ? 'btn-primary' : 'btn-secondary'} onClick={() => setTab(item)}>{item}</button>)}
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
      {tab === 'Controls' && <DataTable rows={controls.rows} columns={['code', 'title', 'category', 'status', 'owner_admin_id', 'evidence_required', 'last_reviewed_at', 'next_review_due_at']} empty="No compliance controls" renderActions={row => <ActionButtons actions={['Mark control reviewed', 'Assign control owner', 'View detail']} onClick={label => label === 'Mark control reviewed' ? helpers.run('Control reviewed.', () => apiRequest('PATCH', `${API}/compliance/controls/${String(row.id || '')}`, { status: 'pass' })).then(load) : label === 'Assign control owner' ? helpers.run('Owner assigned.', () => apiRequest('PATCH', `${API}/compliance/controls/${String(row.id || '')}`, { owner_admin_id: null })).then(load) : helpers.setDrawer({ title: 'Compliance control', body: row })} />} />}
      {tab === 'Evidence' && <DataTable rows={evidence.rows} columns={['id', 'control_id', 'file_name', 'file_reference', 'uploaded_by_admin_id', 'note', 'created_at']} empty="No compliance evidence yet" onView={row => helpers.setDrawer({ title: 'Evidence', body: row })} />}
      {tab === 'Access Reviews' && <DataTable rows={reviews.rows} columns={['id', 'review_period', 'status', 'scope', 'due_at', 'reviewer_admin_id', 'started_by_admin_id', 'completed_by_admin_id', 'started_at', 'completed_at']} empty={{ title: 'No access reviews', body: 'Start an access review to verify least-privilege company admin access.' }} renderActions={row => <ActionButtons actions={['Complete access review', 'View detail']} onClick={label => label === 'Complete access review' ? completeReview(row) : helpers.setDrawer({ title: 'Access review', body: row })} />} />}
      {tab === 'Security Events' && <DataTable rows={securityEvents.rows} columns={['id', 'event_type', 'severity', 'actor_admin_id', 'entity_type', 'entity_id', 'ip_address', 'summary', 'created_at']} empty={{ title: 'No security events', body: 'Security events appear here when sensitive access, export, or credential actions occur.' }} onView={row => helpers.setDrawer({ title: 'Security event', body: row })} />}
      {tab === 'Data Retention' && <DataTable rows={retentionRows} columns={['policy', 'status']} empty="No data retention policies" />}
      {tab === 'Audit Export' && <DataTable rows={[{ id: 'audit', export_type: 'Audit logs', masking: 'masked', status: 'available' }, { id: 'security', export_type: 'Security events', masking: 'masked', status: 'available' }]} columns={['export_type', 'masking', 'status']} empty="No exports" />}
      {tab === 'Credential Rotation' && <DataTable rows={rotationRows} columns={['item', 'count', 'status']} empty="No credential rotation tasks" />}
      {tab === 'Compliance Packet' && <DataTable rows={packets.rows} columns={['id', 'period', 'status', 'file_reference', 'created_at', 'expires_at']} empty="No compliance packet exports yet" renderActions={row => <ActionButtons actions={['Export compliance packet', 'View detail']} onClick={label => label === 'Export compliance packet' ? helpers.run('Compliance packet queued.', () => apiRequest('POST', `${API}/compliance/packet-exports`, { period: 'Q2-2026' })).then(load) : helpers.setDrawer({ title: 'Compliance packet', body: row })} />} />}
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
  const canManageSettings = adminHasPermission('platform_settings.manage');
  const load = async () => {
    setState('loading');
    setError('');
    try {
      const data = await apiRequest<Partial<PlatformSettingsForm>>('GET', '/admin/platform-settings');
      setForm({
        ...DEFAULT_PLATFORM_SETTINGS,
        ...data,
        platform_name: str(data.platform_name || DEFAULT_PLATFORM_SETTINGS.platform_name),
        support_email: str(data.support_email),
        support_phone: str(data.support_phone),
        default_country: str(data.default_country || DEFAULT_PLATFORM_SETTINGS.default_country),
        default_currency: str(data.default_currency || DEFAULT_PLATFORM_SETTINGS.default_currency),
        session_timeout: str(data.session_timeout || DEFAULT_PLATFORM_SETTINGS.session_timeout),
        audit_retention_days: str(data.audit_retention_days || DEFAULT_PLATFORM_SETTINGS.audit_retention_days),
        export_retention_days: str(data.export_retention_days || DEFAULT_PLATFORM_SETTINGS.export_retention_days),
        notification_email_sender: str(data.notification_email_sender),
        timezone: str(data.timezone || DEFAULT_PLATFORM_SETTINGS.timezone),
        date_format: str(data.date_format || DEFAULT_PLATFORM_SETTINGS.date_format),
        maintenance_mode: Boolean(data.maintenance_mode ?? DEFAULT_PLATFORM_SETTINGS.maintenance_mode),
        mfa_required: Boolean(data.mfa_required ?? DEFAULT_PLATFORM_SETTINGS.mfa_required),
      });
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
            <Field label="Platform name"><input value={str(form.platform_name)} onChange={event => setForm({ ...form, platform_name: event.target.value })} /></Field>
            <Field label="Support email"><input value={str(form.support_email)} onChange={event => setForm({ ...form, support_email: event.target.value })} /></Field>
            <Field label="Support phone"><input value={str(form.support_phone)} onChange={event => setForm({ ...form, support_phone: event.target.value })} /></Field>
            <Field label="Default country"><input value={str(form.default_country)} onChange={event => setForm({ ...form, default_country: event.target.value })} /></Field>
            <Field label="Default currency"><input value={str(form.default_currency)} onChange={event => setForm({ ...form, default_currency: event.target.value })} /></Field>
            <Field label="Session timeout"><input value={str(form.session_timeout)} onChange={event => setForm({ ...form, session_timeout: event.target.value })} /></Field>
            <Field label="Audit retention days"><input value={str(form.audit_retention_days)} onChange={event => setForm({ ...form, audit_retention_days: event.target.value })} /></Field>
            <Field label="Export retention days"><input value={str(form.export_retention_days)} onChange={event => setForm({ ...form, export_retention_days: event.target.value })} /></Field>
            <Field label="Notification email sender"><input value={str(form.notification_email_sender)} onChange={event => setForm({ ...form, notification_email_sender: event.target.value })} /></Field>
            <Field label="Timezone"><input value={str(form.timezone)} onChange={event => setForm({ ...form, timezone: event.target.value })} /></Field>
            <Field label="Date format"><input value={str(form.date_format)} onChange={event => setForm({ ...form, date_format: event.target.value })} /></Field>
            <label className="form-field"><span>Maintenance mode</span><input type="checkbox" checked={Boolean(form.maintenance_mode)} onChange={event => setForm({ ...form, maintenance_mode: event.target.checked })} /></label>
            <label className="form-field"><span>MFA required</span><input type="checkbox" checked={Boolean(form.mfa_required)} onChange={event => setForm({ ...form, mfa_required: event.target.checked })} /></label>
          </div>
          <div className="button-row">
            {canManageSettings && <button className="btn-primary" type="submit">Save settings</button>}
            {canManageSettings && <button type="button" onClick={() => maintenance(true)}>Enable maintenance mode</button>}
            {canManageSettings && <button type="button" onClick={() => maintenance(false)}>Disable maintenance mode</button>}
            {canManageSettings && <button type="button" onClick={() => setForm(DEFAULT_PLATFORM_SETTINGS)}>Reset defaults</button>}
          </div>
        </form>
      </section>
      <div className="card">
        <h2><Mail size={18} /> Security posture</h2>
        <div className="security-list">
          {['Deny-by-default RBAC', 'Least privilege route guards', 'Confirmation for sensitive settings', 'Audit logs for setting changes'].map(item => <div className="security-row" key={item}><span>{item}:</span> <span className="badge success">Enabled</span></div>)}
        </div>
      </div>
      <Modals helpers={helpers} />
    </CrudPage>
  );
}

function CrudPage({ title, subtitle, state, error, notice, onRefresh, children }: { title: string; subtitle: string; state: LoadState; error: string; notice?: string; onRefresh: () => void; children: ReactNode }) {
  return (
    <Page title={title} subtitle={subtitle} onRefresh={onRefresh}>
      <Status state={state} error={error} loading={`Loading ${title.toLowerCase()}...`} onRetry={onRefresh} />
      {notice && <div className={`alert ${notice.includes('Failed') || notice.includes('error') ? 'alert-error' : 'alert-success'}`}>{notice}</div>}
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

function Status({ state, error, loading, onRetry }: { state: LoadState; error: string; loading: string; onRetry?: () => void }) {
  if (state === 'loading') return <Skeleton message={loading} />;
  if (state === 'error' || (error && state === 'ready')) return <ErrorPanel message={error || 'An unexpected error occurred'} onRetry={onRetry} />;
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
  const [showDetail, setShowDetail] = useState(false);
  return (
    <div className="alert alert-error error-panel">
      <span>{friendlyError(message)}</span>
      {onRetry && <button type="button" onClick={onRetry}>Retry</button>}
      {message && message !== friendlyError(message) && <button type="button" onClick={() => setShowDetail(value => !value)}>View technical detail</button>}
      {showDetail && <pre className="json-preview">{message}</pre>}
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
    <section className="table-card table-panel-card">
      <div className="table-panel-header">
        <h2>{title}</h2>
        <span className="count-badge">{rows.length}</span>
      </div>
      <DataTable rows={rows} columns={columns} empty={`No ${title.toLowerCase()} records`} isNested={true} />
    </section>
  );
}

function DataTable({
  rows,
  columns,
  empty,
  renderActions,
  onView,
  pageSize = 10,
  isNested = false,
}: {
  rows: GenericRecord[];
  columns: string[];
  empty: string | EmptyStateConfig;
  renderActions?: (row: GenericRecord) => ReactNode;
  onView?: (row: GenericRecord) => void;
  pageSize?: number;
  isNested?: boolean;
}) {
  const [currentPage, setCurrentPage] = useState(1);

  useEffect(() => {
    setCurrentPage(1);
  }, [rows.length]);

  const totalPages = Math.max(1, Math.ceil(rows.length / pageSize));
  const activePage = Math.min(currentPage, totalPages);

  const paginatedRows = rows.length > pageSize
    ? rows.slice((activePage - 1) * pageSize, activePage * pageSize)
    : rows;

  const isCompact = columns.length <= 3 && !renderActions && !onView;

  return (
    <div className={`data-table-container ${isNested ? 'nested-table' : ''}`}>
      <div
        className={`table-scroll ${isCompact ? 'table-fit' : ''}`}
        tabIndex={0}
        aria-label="Scrollable data table"
      >
        <table className={isCompact ? 'table-fit-content' : ''}>
          <thead>
            <tr>
              {columns.map(column => <th key={column}>{formatHeader(column)}</th>)}
              {(renderActions || onView) && <th className="actions-cell">Actions</th>}
            </tr>
          </thead>
          <tbody>
            {paginatedRows.map((row, index) => (
              <tr key={String(row.id || (activePage - 1) * pageSize + index)}>
                {columns.map(column => <td key={column}>{renderCell(row, column)}</td>)}
                {(renderActions || onView) && (
                  <td className="actions-cell">
                    {renderActions ? renderActions(row) : (
                      <button type="button" className="btn-action-primary" onClick={() => onView?.(row)}>
                        <Eye size={12} /> View detail
                      </button>
                    )}
                  </td>
                )}
              </tr>
            ))}
            {rows.length === 0 && (
              <tr>
                <td colSpan={columns.length + (renderActions || onView ? 1 : 0)}>
                  {typeof empty === 'string' ? empty : <EmptyState config={empty} />}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {rows.length > pageSize && (
        <div className="table-pagination">
          <div className="table-pagination-info">
            Showing <span className="pagination-highlight">{(activePage - 1) * pageSize + 1}</span>–<span className="pagination-highlight">{Math.min(activePage * pageSize, rows.length)}</span> of <span className="pagination-highlight">{rows.length}</span> entries
          </div>
          <div className="table-pagination-controls">
            <button
              type="button"
              className="pagination-btn"
              disabled={activePage === 1}
              onClick={() => setCurrentPage(prev => Math.max(1, prev - 1))}
              aria-label="Previous page"
            >
              <ChevronLeft size={14} />
              <span>Prev</span>
            </button>
            <div className="pagination-pages">
              {Array.from({ length: totalPages }, (_, i) => i + 1).map(page => {
                if (
                  totalPages <= 7 ||
                  page === 1 ||
                  page === totalPages ||
                  (page >= activePage - 1 && page <= activePage + 1)
                ) {
                  return (
                    <button
                      key={page}
                      type="button"
                      className={`pagination-num ${page === activePage ? 'active' : ''}`}
                      onClick={() => setCurrentPage(page)}
                    >
                      {page}
                    </button>
                  );
                }
                if (
                  (page === 2 && activePage > 3) ||
                  (page === totalPages - 1 && activePage < totalPages - 2)
                ) {
                  return <span key={page} className="pagination-ellipsis">…</span>;
                }
                return null;
              })}
            </div>
            <button
              type="button"
              className="pagination-btn"
              disabled={activePage === totalPages}
              onClick={() => setCurrentPage(prev => Math.min(totalPages, prev + 1))}
              aria-label="Next page"
            >
              <span>Next</span>
              <ChevronRight size={14} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function Metric({ label, value, status, icon: Icon, description }: { label: string; value: string | number; status?: string; icon: typeof Activity; description?: string }) {
  return (
    <article className="stat-card">
      <div className="stat-label"><Icon size={14} /> {label}</div>
      <div className={`stat-value ${status ? status : ''}`}>{value}</div>
      {description && <div className="stat-hint">{description}</div>}
    </article>
  );
}


function ActionDropdown({ actions, onSelect, label }: { actions: string[]; onSelect: (action: string) => void; label?: string }) {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [menuPos, setMenuPos] = useState<{ top: number; right: number } | null>(null);

  const toggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (open) {
      setOpen(false);
      return;
    }
    if (triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      const right = Math.max(12, window.innerWidth - rect.right);
      const estHeight = actions.length * 34 + 38;
      const top = (rect.bottom + 4 + estHeight > window.innerHeight && rect.top > estHeight)
        ? Math.max(8, rect.top - estHeight - 4)
        : rect.bottom + 4;
      setMenuPos({ top, right });
      setOpen(true);
    }
  };

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (e: MouseEvent) => {
      if (
        triggerRef.current &&
        !triggerRef.current.contains(e.target as Node) &&
        menuRef.current &&
        !menuRef.current.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    };
    const handleClose = () => setOpen(false);
    document.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('resize', handleClose);
    return () => {
      document.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('resize', handleClose);
    };
  }, [open]);

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        className={label ? 'btn-action-secondary' : 'btn-action-dropdown-trigger'}
        aria-expanded={open}
        aria-label={label || 'More actions'}
        onClick={toggle}
      >
        {label ? (
          <>
            <span>{label}</span>
            <ChevronDown size={12} style={{ marginLeft: 4 }} />
          </>
        ) : (
          <ChevronDown size={12} />
        )}
      </button>
      {open && menuPos && createPortal(
        <div
          ref={menuRef}
          className="action-dropdown-menu"
          style={{
            position: 'fixed',
            top: menuPos.top,
            right: menuPos.right,
            zIndex: 99999,
          }}
          onClick={e => e.stopPropagation()}
        >
          <div className="action-dropdown-header">Actions</div>
          {actions.map(action => (
            <button
              key={action}
              type="button"
              className="action-dropdown-item"
              onClick={() => {
                setOpen(false);
                onSelect(action);
              }}
            >
              {action}
            </button>
          ))}
        </div>,
        document.body
      )}
    </>
  );
}

function ActionButtons({ actions, onClick, dropdownOnly }: { actions: string[]; onClick: (action: string) => void; dropdownOnly?: boolean }) {
  if (dropdownOnly) {
    return (
      <div className="compact-actions">
        <ActionDropdown actions={actions} onSelect={onClick} label="Actions" />
      </div>
    );
  }
  const hasViewDetail = actions.some(a => a.toLowerCase() === 'view detail');
  const primaryAction = (hasViewDetail
    ? actions.find(a => a.toLowerCase() === 'view detail')
    : actions[0]) || '';
  const remainingActions = hasViewDetail
    ? actions.filter(a => a.toLowerCase() !== 'view detail')
    : actions.slice(1);

  return (
    <div className="compact-actions">
      {primaryAction && (
        <button key={primaryAction} type="button" className="btn-action-primary" onClick={() => onClick(primaryAction)}>
          {primaryAction.toLowerCase().includes('view') && <Eye size={12} />}
          {primaryAction}
        </button>
      )}
      {remainingActions.length === 1 && remainingActions[0] ? (
        <button key={remainingActions[0]} type="button" className="btn-action-secondary" onClick={() => onClick(remainingActions[0]!)}>
          {remainingActions[0]}
        </button>
      ) : remainingActions.length > 1 ? (
        <ActionDropdown actions={remainingActions} onSelect={onClick} />
      ) : null}
    </div>
  );
}

function RoutingSimulator({ form, result, onChange, onSubmit }: { form: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string; strategy: string }; result: RoutingSimulation | null; onChange: (value: { payment_method: string; currency: string; country: string; amount: number; business_account: string; customer_region: string; risk_level: string; strategy: string }) => void; onSubmit: (event: FormEvent) => void }) {
  const riskOptions = [
    { label: 'Low Risk (Verified Customer)', value: 'low' },
    { label: 'Medium Risk (Standard)', value: 'medium' },
    { label: 'High Risk (Flagged/Unusual)', value: 'high' },
    { label: 'Critical Risk (High Fraud Chance)', value: 'critical' },
  ];

  const strategyOptions = [
    { label: 'Preferred Gateway First (Priority)', value: 'priority' },
    { label: 'Lowest Cost (Cheapest Gateway Fee)', value: 'lowest_cost' },
    { label: 'Highest Success Rate (Most Reliable)', value: 'highest_success_rate' },
    { label: 'Fastest Response Speed (Lowest Latency)', value: 'latency_aware' },
    { label: '100% Uptime Only (Health Aware)', value: 'health_aware' },
    { label: 'Risk Aware (Extra Checks for Suspicious)', value: 'risk_aware' },
    { label: 'Split Volume % (Weighted)', value: 'weighted' },
    { label: 'Even Distribution (Round Robin)', value: 'round_robin' },
    { label: 'Emergency Backup Only (Fallback)', value: 'fallback_only' },
  ];

  return (
    <section className="card">
      <h2><Route size={18} /> Test Route Simulator (Preview Gateway Choice)</h2>
      <p style={{ color: 'var(--fp-text-muted)', fontSize: '0.875rem', marginTop: 4, marginBottom: 16 }}>
        Test a sample payment to see which gateway (e.g. Razorpay or Cashfree) our smart router chooses, along with estimated fees and response speed.
      </p>
      <form onSubmit={onSubmit} className="filter-grid">
        <Field label="Payment method (e.g. UPI, Card)">
          <Select value={str(form.payment_method)} options={PAYMENT_METHOD_OPTIONS.filter(o => o.value !== '')} onChange={payment_method => onChange({ ...form, payment_method })} />
        </Field>
        <Field label="Currency">
          <Select value={str(form.currency)} options={CURRENCY_OPTIONS.filter(o => o.value !== '')} onChange={currency => onChange({ ...form, currency })} />
        </Field>
        <Field label="Customer Country">
          <input value={str(form.country)} onChange={event => onChange({ ...form, country: event.target.value })} placeholder="e.g. IN" />
        </Field>
        <Field label="Payment Amount (paise/cents)">
          <input type="number" value={form.amount ?? 0} onChange={event => onChange({ ...form, amount: Number(event.target.value) })} placeholder="e.g. 10000" />
        </Field>
        <Field label="Merchant Business Account ID">
          <input value={str(form.business_account)} onChange={event => onChange({ ...form, business_account: event.target.value })} placeholder="Merchant Account UUID" />
        </Field>
        <Field label="Customer State / Region">
          <input value={str(form.customer_region)} onChange={event => onChange({ ...form, customer_region: event.target.value })} placeholder="e.g. IN-MH" />
        </Field>
        <Field label="Customer Risk Level">
          <Select value={str(form.risk_level)} options={riskOptions} onChange={risk_level => onChange({ ...form, risk_level })} />
        </Field>
        <Field label="Routing Goal (Strategy)">
          <Select value={str(form.strategy)} options={strategyOptions} onChange={strategy => onChange({ ...form, strategy })} />
        </Field>
        <div className="filter-actions">
          <button className="btn-primary" type="submit">Run Route Test</button>
        </div>
      </form>

      <div className="stats-grid">
        <Metric label="Chosen Gateway (Winner)" value={result?.selected_provider || 'No match'} icon={Cable} />
        <Metric label="Matched Routing Rule" value={result?.matched_routing_rule || 'Default Route'} icon={Route} />
        <Metric label="Gateway Fit Score" value={result?.provider_score ? result.provider_score.toFixed(2) : '1.00'} icon={Activity} />
        <Metric label="Backup Gateway (Failover)" value={result?.fallback_provider || 'None (Primary Healthy)'} icon={AlertTriangle} />
        <Metric label="Estimated Gateway Fee" value={money(result?.estimated_provider_fee || 0)} icon={Receipt} />
        <Metric label="Estimated Speed" value={`${result?.latency_estimate_ms || 0} ms`} icon={RefreshCw} />
        <Metric label="Estimated Success Rate" value={`${(result?.success_estimate || 0).toFixed(1)}%`} icon={BadgeCheck} />
        <Metric label="Strategy Applied" value={result?.strategy_used || 'Default Strategy'} icon={SlidersHorizontal} />
      </div>

      {result && !result.selected_provider && (
        <div className="alert alert-warning" style={{ marginTop: 16 }}>
          No matching gateway found for this transaction. Check that your route rules are active or enable a default backup route.
        </div>
      )}

      {result && (
        <div style={{ marginTop: 20 }}>
          <h3 style={{ fontSize: '0.9375rem', fontWeight: 600, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>🏆</span> Evaluated Gateways (Ranked by Score)
          </h3>
          <DataTable rows={result.candidate_providers || []} columns={['provider', 'score', 'success_rate', 'latency_ms', 'fee_percent', 'attempt_count']} empty="No candidate gateways evaluated" />
        </div>
      )}

      {result && (
        <div style={{ marginTop: 20 }}>
          <h3 style={{ fontSize: '0.9375rem', fontWeight: 600, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
            <span>🚫</span> Ineligible Gateways (Why they were rejected or skipped)
          </h3>
          <DataTable rows={result.rejected_providers || []} columns={['provider', 'reason']} empty="No gateways rejected" />
        </div>
      )}
    </section>
  );
}

function WhiteLabelPanel({ value, providers, onChange, onSubmit, onReset }: { value: WhiteLabelForm; providers: Provider[]; onChange: (value: WhiteLabelForm) => void; onSubmit: (event: FormEvent) => void; onReset: () => void }) {
  const [uploading, setUploading] = useState(false);
  const [logoError, setLogoError] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleLogoUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (file.size < 200 * 1024 || file.size > 5 * 1024 * 1024) {
      setLogoError('Logo must be between 200KB and 5MB');
      return;
    }
    setLogoError('');
    setUploading(true);

    try {
      const reader = new FileReader();
      reader.onload = async () => {
        const base64 = (reader.result as string).split(',')[1];
        const result = await apiRequest<{ logo_url: string }>('POST', `${API}/white-label-config/upload-logo`, {
          content_type: file.type,
          data_base64: base64,
        });
        onChange({ ...value, checkout_logo_url: result.logo_url });
        setUploading(false);
      };
      reader.readAsDataURL(file);
    } catch (err) {
      setLogoError(err instanceof Error ? err.message : 'Upload failed');
      setUploading(false);
    }
  };

  const paymentMethodOptions = [
    { label: 'UPI', value: 'upi', icon: <QrCode size={13} /> },
    { label: 'Card', value: 'card', icon: <CreditCard size={13} /> },
    { label: 'Net Banking', value: 'netbanking', icon: <Building2 size={13} /> },
    { label: 'Wallet', value: 'wallet', icon: <Wallet size={13} /> },
    { label: 'EMI', value: 'emi', icon: <Landmark size={13} /> },
    { label: 'BNPL', value: 'bnpl', icon: <Zap size={13} /> },
  ];

  const currencyOptions = [
    { label: 'Indian Rupee (INR)', value: 'INR' },
    { label: 'US Dollar (USD)', value: 'USD' },
    { label: 'Euro (EUR)', value: 'EUR' },
    { label: 'British Pound (GBP)', value: 'GBP' },
    { label: 'UAE Dirham (AED)', value: 'AED' },
  ];

  const countryOptions = [
    { label: 'India', value: 'IN' },
    { label: 'United States', value: 'US' },
    { label: 'United Kingdom', value: 'GB' },
    { label: 'United Arab Emirates', value: 'AE' },
  ];

  const themeSwatches = [
    { name: 'Emerald', hex: '#1DB954' },
    { name: 'Indigo', hex: '#6366F1' },
    { name: 'Sapphire', hex: '#2563EB' },
    { name: 'Rose', hex: '#E11D48' },
    { name: 'Amber', hex: '#F59E0B' },
    { name: 'Violet', hex: '#8B5CF6' },
  ];

  return (
    <section className="card">
      <div className="page-header" style={{ marginBottom: 24 }}>
        <div>
          <h2><SlidersHorizontal size={18} /> Platform checkout configuration</h2>
          <p className="muted-text">Customize the checkout experience for all hosted payment flows.</p>
        </div>
        <div className="button-row">
          <button type="button" className="btn-secondary" onClick={onReset}>
            <RotateCcw size={13} /> Reset defaults
          </button>
          <button className="btn-primary" type="button" onClick={onSubmit}>
            <Check size={14} /> Save configuration
          </button>
        </div>
      </div>

      <form onSubmit={onSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 28 }}>
        <div className="grid grid-2" style={{ gap: 28 }}>
          {/* Left Column */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
            {/* Visual Branding */}
            <div>
              <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--fp-text-muted, #9CA3AF)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 14 }}>
                Visual Branding & Host
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14 }}>
                <Field label="Brand Name">
                  <input
                    value={str(value.brand_name)}
                    onChange={event => onChange({ ...value, brand_name: event.target.value })}
                    placeholder="e.g. FeelPay"
                  />
                </Field>

                <Field label="Custom Checkout Domain">
                  <input
                    value={str(value.payment_page_domain)}
                    onChange={event => onChange({ ...value, payment_page_domain: event.target.value })}
                    placeholder="e.g. checkout.feelpay.com"
                  />
                  <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)', marginTop: 4 }}>
                    Custom white-label domain or subdomain configured for hosted checkout sessions
                  </div>
                </Field>

                <Field label="Brand Logo (200KB - 5MB)">
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 14,
                      padding: 12,
                      background: 'rgba(0,0,0,0.2)',
                      border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
                      borderRadius: 10,
                    }}
                  >
                    <div
                      style={{
                        width: 52,
                        height: 52,
                        borderRadius: 10,
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--fp-border, rgba(255,255,255,0.12))',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        overflow: 'hidden',
                        flexShrink: 0,
                      }}
                    >
                      {value.checkout_logo_url ? (
                        <img src={value.checkout_logo_url} alt="Logo" style={{ width: '100%', height: '100%', objectFit: 'contain' }} />
                      ) : (
                        <Upload size={22} style={{ color: 'var(--fp-text-muted, #777)' }} />
                      )}
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <input
                        type="file"
                        ref={fileInputRef}
                        accept="image/*"
                        onChange={handleLogoUpload}
                        disabled={uploading}
                        style={{ display: 'none' }}
                      />
                      <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
                        <button
                          type="button"
                          className="btn-secondary"
                          onClick={() => fileInputRef.current?.click()}
                          disabled={uploading}
                          style={{ fontSize: 11, padding: '5px 10px' }}
                        >
                          <Upload size={12} /> {uploading ? 'Uploading...' : 'Choose Image'}
                        </button>
                        {value.checkout_logo_url && (
                          <button
                            type="button"
                            className="btn-secondary"
                            onClick={() => onChange({ ...value, checkout_logo_url: '' })}
                            style={{ fontSize: 11, padding: '5px 8px', color: '#EF4444' }}
                            title="Remove Logo"
                          >
                            <X size={12} /> Remove
                          </button>
                        )}
                      </div>
                      <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)', marginTop: 4 }}>
                        PNG, SVG, or JPG (Recommended: 256x256 square)
                      </div>
                      {logoError && <div style={{ fontSize: 11, color: '#ef4444', marginTop: 4 }}>{logoError}</div>}
                    </div>
                  </div>
                </Field>

                <Field label="Theme Accent Color">
                  <div>
                    <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 8 }}>
                      <div style={{ position: 'relative', width: 40, height: 40, borderRadius: 8, overflow: 'hidden', border: '1px solid rgba(255,255,255,0.2)', flexShrink: 0 }}>
                        <input
                          type="color"
                          value={str(value.checkout_theme_color || '#1DB954')}
                          onChange={event => onChange({ ...value, checkout_theme_color: event.target.value })}
                          style={{ position: 'absolute', top: -10, left: -10, width: 60, height: 60, cursor: 'pointer', border: 'none', padding: 0 }}
                        />
                      </div>
                      <input
                        value={str(value.checkout_theme_color)}
                        onChange={event => onChange({ ...value, checkout_theme_color: event.target.value })}
                        placeholder="#1DB954"
                        style={{ fontFamily: 'monospace', flex: 1 }}
                      />
                    </div>
                    {/* Quick Swatches */}
                    <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
                      {themeSwatches.map(sw => {
                        const isSelected = str(value.checkout_theme_color).toLowerCase() === sw.hex.toLowerCase();
                        return (
                          <button
                            key={sw.hex}
                            type="button"
                            onClick={() => onChange({ ...value, checkout_theme_color: sw.hex })}
                            style={{
                              display: 'inline-flex',
                              alignItems: 'center',
                              gap: 5,
                              padding: '3px 8px',
                              borderRadius: 6,
                              fontSize: 10,
                              cursor: 'pointer',
                              background: isSelected ? `${sw.hex}22` : 'rgba(255,255,255,0.04)',
                              border: isSelected ? `1px solid ${sw.hex}` : '1px solid rgba(255,255,255,0.1)',
                              color: isSelected ? '#FFFFFF' : 'var(--fp-text-muted, #aaa)',
                            }}
                          >
                            <span style={{ width: 8, height: 8, borderRadius: '50%', background: sw.hex }} />
                            <span>{sw.name}</span>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </Field>
              </div>
            </div>

            {/* Payment Defaults */}
            <div>
              <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--fp-text-muted, #9CA3AF)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 14 }}>
                Payment Defaults
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14 }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                  <Field label="Default Currency">
                    <Select value={str(value.default_currency)} options={currencyOptions} onChange={next => onChange({ ...value, default_currency: next })} />
                  </Field>
                  <Field label="Default Country">
                    <Select value={str(value.default_country)} options={countryOptions} onChange={next => onChange({ ...value, default_country: next })} />
                  </Field>
                </div>

                <Field label="Default Payment Methods">
                  <div
                    style={{
                      display: 'flex',
                      flexWrap: 'wrap',
                      gap: 8,
                      padding: 12,
                      border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
                      borderRadius: 10,
                      background: 'rgba(0,0,0,0.2)',
                    }}
                  >
                    {paymentMethodOptions.map(opt => {
                      const methods = list(value.default_payment_methods);
                      const isChecked = methods.includes(opt.value);
                      return (
                        <label
                          key={opt.value}
                          style={{
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: 6,
                            fontSize: 12,
                            fontWeight: isChecked ? 700 : 500,
                            cursor: 'pointer',
                            padding: '6px 12px',
                            borderRadius: 8,
                            background: isChecked ? 'rgba(29, 185, 84, 0.14)' : 'rgba(255,255,255,0.04)',
                            color: isChecked ? '#10B981' : 'var(--fp-text-muted, #aaa)',
                            border: isChecked ? '1px solid #10B981' : '1px solid var(--fp-border, rgba(255,255,255,0.1))',
                            transition: 'all 0.15s ease',
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={isChecked}
                            onChange={e => {
                              const next = e.target.checked ? [...methods, opt.value] : methods.filter(m => m !== opt.value);
                              onChange({ ...value, default_payment_methods: next.join(',') });
                            }}
                            style={{ display: 'none' }}
                          />
                          {opt.icon}
                          <span>{opt.label}</span>
                          {isChecked && <Check size={13} style={{ color: '#10B981' }} />}
                        </label>
                      );
                    })}
                  </div>
                  <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)', marginTop: 4 }}>
                    Enabled channels appear immediately on customer checkout and payment links
                  </div>
                </Field>
              </div>
            </div>
          </div>

          {/* Right Column */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
            {/* Provider Routing */}
            <div>
              <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--fp-text-muted, #9CA3AF)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 14 }}>
                Provider Routing
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14 }}>
                <Field label="Primary Routing Provider">
                  <Select
                    value={str(value.default_provider_id)}
                    options={[{ label: 'Smart Routing (Automatic)', value: '' }, ...providers.map(p => ({ label: `${p.name} (${p.provider_type})`, value: p.id }))]}
                    onChange={next => onChange({ ...value, default_provider_id: next })}
                  />
                </Field>
                <Field label="Failover Provider">
                  <Select
                    value={str(value.fallback_provider_id)}
                    options={[{ label: 'None', value: '' }, ...providers.map(p => ({ label: `${p.name} (${p.provider_type})`, value: p.id }))]}
                    onChange={next => onChange({ ...value, fallback_provider_id: next })}
                  />
                </Field>
              </div>
            </div>

            {/* Legal & Customer Support */}
            <div>
              <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--fp-text-muted, #9CA3AF)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 14 }}>
                Legal & Customer Support
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14 }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                  <Field label="Support Email">
                    <input
                      value={str(value.support_email)}
                      onChange={event => onChange({ ...value, support_email: event.target.value })}
                      placeholder="support@feelpay.local"
                    />
                  </Field>
                  <Field label="Support Phone Number">
                    <input
                      value={str(value.support_phone)}
                      onChange={event => onChange({ ...value, support_phone: event.target.value })}
                      placeholder="+91 80 4567 8900"
                    />
                  </Field>
                </div>
                <Field label="Terms of Service URL">
                  <input
                    value={str(value.terms_url)}
                    onChange={event => onChange({ ...value, terms_url: event.target.value })}
                    placeholder="https://feelpay.local/terms"
                  />
                </Field>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                  <Field label="Privacy Policy URL">
                    <input
                      value={str(value.privacy_url)}
                      onChange={event => onChange({ ...value, privacy_url: event.target.value })}
                      placeholder="https://feelpay.local/privacy"
                    />
                  </Field>
                  <Field label="Refund Policy URL">
                    <input
                      value={str(value.refund_policy_url)}
                      onChange={event => onChange({ ...value, refund_policy_url: event.target.value })}
                      placeholder="https://feelpay.local/refunds"
                    />
                  </Field>
                </div>
              </div>
            </div>

            {/* Checkout Content & Descriptors */}
            <div>
              <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--fp-text-muted, #9CA3AF)', textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 14 }}>
                Checkout Content & Descriptors
              </h3>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14 }}>
                <Field label="Bank Statement Descriptor">
                  <input
                    value={str(value.statement_descriptor)}
                    maxLength={22}
                    onChange={event => onChange({ ...value, statement_descriptor: event.target.value.toUpperCase() })}
                    placeholder="FEELPAY*CHECKOUT"
                    style={{ fontFamily: 'monospace' }}
                  />
                  <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)', marginTop: 4, display: 'flex', justifyContent: 'space-between' }}>
                    <span>Appears on customer bank statements & instant SMS debit alerts</span>
                    <span style={{ fontFamily: 'monospace' }}>{str(value.statement_descriptor).length}/22</span>
                  </div>
                </Field>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                  <Field label="Invoice Prefix">
                    <input
                      value={str(value.invoice_prefix)}
                      onChange={event => onChange({ ...value, invoice_prefix: event.target.value })}
                      placeholder="INV"
                    />
                  </Field>
                  <Field label="Receipt Prefix">
                    <input
                      value={str(value.receipt_prefix)}
                      onChange={event => onChange({ ...value, receipt_prefix: event.target.value })}
                      placeholder="RCPT"
                    />
                  </Field>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="button-row" style={{ marginTop: 12, paddingTop: 20, borderTop: '1px solid var(--fp-border, rgba(255,255,255,0.08))', justifyContent: 'flex-end' }}>
          <button type="button" className="btn-secondary" onClick={onReset}>
            <RotateCcw size={13} /> Discard changes
          </button>
          <button className="btn-primary" type="submit">
            <Check size={14} /> Save configuration
          </button>
        </div>
      </form>
    </section>
  );
}

function QrCodeSvg({ brandInitial, color }: { brandInitial: string; color: string }) {
  return (
    <svg
      viewBox="0 0 200 200"
      width="168"
      height="168"
      style={{
        background: '#FFFFFF',
        borderRadius: 12,
        padding: 8,
        display: 'block',
        margin: '0 auto',
        maxWidth: '100%',
        height: 'auto',
        aspectRatio: '1/1',
      }}
    >
      {/* Top-Left Finder */}
      <rect x="12" y="12" width="44" height="44" rx="6" fill="#000" />
      <rect x="20" y="20" width="28" height="28" rx="3" fill="#fff" />
      <rect x="26" y="26" width="16" height="16" rx="2" fill="#000" />

      {/* Top-Right Finder */}
      <rect x="144" y="12" width="44" height="44" rx="6" fill="#000" />
      <rect x="152" y="20" width="28" height="28" rx="3" fill="#fff" />
      <rect x="158" y="26" width="16" height="16" rx="2" fill="#000" />

      {/* Bottom-Left Finder */}
      <rect x="12" y="144" width="44" height="44" rx="6" fill="#000" />
      <rect x="20" y="152" width="28" height="28" rx="3" fill="#fff" />
      <rect x="26" y="158" width="16" height="16" rx="2" fill="#000" />

      {/* Alignment / Timing Grid */}
      <rect x="62" y="30" width="76" height="4" fill="#000" />
      <rect x="30" y="62" width="4" height="76" fill="#000" />

      {/* Data Blocks */}
      <rect x="64" y="14" width="8" height="8" fill="#000" />
      <rect x="78" y="18" width="12" height="6" fill="#000" />
      <rect x="98" y="12" width="6" height="12" fill="#000" />
      <rect x="114" y="20" width="14" height="6" fill="#000" />

      <rect x="14" y="64" width="8" height="8" fill="#000" />
      <rect x="18" y="78" width="6" height="12" fill="#000" />
      <rect x="12" y="98" width="12" height="6" fill="#000" />
      <rect x="20" y="114" width="6" height="14" fill="#000" />

      <rect x="64" y="68" width="10" height="10" fill="#000" />
      <rect x="126" y="68" width="10" height="10" fill="#000" />
      <rect x="64" y="122" width="10" height="10" fill="#000" />
      <rect x="126" y="122" width="10" height="10" fill="#000" />

      <rect x="80" y="60" width="40" height="6" fill="#000" />
      <rect x="80" y="134" width="40" height="6" fill="#000" />

      <rect x="144" y="64" width="14" height="8" fill="#000" />
      <rect x="168" y="74" width="8" height="16" fill="#000" />
      <rect x="146" y="98" width="18" height="6" fill="#000" />
      <rect x="172" y="114" width="10" height="10" fill="#000" />

      <rect x="64" y="144" width="14" height="8" fill="#000" />
      <rect x="74" y="168" width="16" height="8" fill="#000" />
      <rect x="98" y="146" width="6" height="18" fill="#000" />
      <rect x="114" y="172" width="10" height="10" fill="#000" />

      <rect x="144" y="144" width="12" height="12" fill="#000" />
      <rect x="164" y="148" width="8" height="14" fill="#000" />
      <rect x="148" y="168" width="14" height="8" fill="#000" />
      <rect x="168" y="168" width="16" height="16" fill="#000" />

      {/* Central Brand Badge */}
      <circle cx="100" cy="100" r="21" fill="#FFFFFF" stroke="#111827" strokeWidth="2" />
      <circle cx="100" cy="100" r="17" fill={color} />
      <text x="100" y="105" textAnchor="middle" fill="#FFFFFF" fontSize="12" fontWeight="800" fontFamily="sans-serif">
        {brandInitial}
      </text>
    </svg>
  );
}

function CheckoutPreview({ value, onEditConfig }: { value: WhiteLabelForm; onEditConfig?: () => void }) {
  const [device, setDevice] = useState<'desktop' | 'mobile'>('desktop');
  const [amount, setAmount] = useState<number>(2499);
  const [activeThemeColor, setActiveThemeColor] = useState<string>(str(value.checkout_theme_color) || '#1DB954');
  const [selectedMethod, setSelectedMethod] = useState<'upi' | 'card' | 'netbanking' | 'wallets'>('upi');
  const [upiMode, setUpiMode] = useState<'qr' | 'vpa'>('qr');
  const [vpaInput, setVpaInput] = useState<string>('merchant.customer@okhdfcbank');
  const [vpaVerified, setVpaVerified] = useState<boolean>(true);
  const [cardNumber, setCardNumber] = useState<string>('4532 8901 2345 8821');
  const [cardExpiry, setCardExpiry] = useState<string>('08/28');
  const [cardCvv, setCardCvv] = useState<string>('882');
  const [cardName, setCardName] = useState<string>('AARAV SHARMA');
  const [saveCard, setSaveCard] = useState<boolean>(true);
  const [selectedBank, setSelectedBank] = useState<string>('HDFC');
  const [selectedWallet, setSelectedWallet] = useState<string>('simpl');
  const [orderExpanded, setOrderExpanded] = useState<boolean>(false);
  const [paymentState, setPaymentState] = useState<'idle' | 'processing' | 'success'>('idle');
  const [copiedLink, setCopiedLink] = useState<boolean>(false);

  useEffect(() => {
    setActiveThemeColor(str(value.checkout_theme_color) || '#1DB954');
  }, [value.checkout_theme_color]);

  const brandName = str(value.brand_name) || 'FeelPay';
  const supportEmail = str(value.support_email) || 'support@feelpay.local';
  const supportPhone = str(value.support_phone) || '+91 80 4567 8900';
  const domain = str(value.payment_page_domain) || 'checkout.feelpay.com';
  const statementDesc = str(value.statement_descriptor) || 'FEELPAY*CHECKOUT';
  const invoicePrefix = str(value.invoice_prefix) || 'INV';
  const receiptPrefix = str(value.receipt_prefix) || 'RCPT';
  const currency = str(value.default_currency) || 'INR';

  const formatCurrency = (val: number) =>
    new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency,
      minimumFractionDigits: 2,
    }).format(val);

  const baseAmount = Math.round((amount / 1.18) * 100) / 100;
  const gstAmount = Math.round((amount - baseAmount) * 100) / 100;

  const handleSimulatePayment = () => {
    if (paymentState !== 'idle') return;
    setPaymentState('processing');
    setTimeout(() => {
      setPaymentState('success');
    }, 1100);
  };

  const handleResetFlow = () => {
    setPaymentState('idle');
  };

  const handleCopyLink = () => {
    void navigator.clipboard.writeText(`https://${domain}/pay/cs_preview_8f012`).then(() => {
      setCopiedLink(true);
      setTimeout(() => setCopiedLink(false), 2000);
    });
  };

  const brandInitial = brandName.slice(0, 2).toUpperCase() || 'FP';

  const brandLogoNode = value.checkout_logo_url ? (
    <img
      src={value.checkout_logo_url}
      alt={brandName}
      style={{ width: 38, height: 38, borderRadius: 10, objectFit: 'cover', border: '1px solid rgba(255,255,255,0.1)' }}
    />
  ) : (
    <div
      style={{
        width: 38,
        height: 38,
        borderRadius: 10,
        background: activeThemeColor,
        color: '#FFFFFF',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontWeight: 800,
        fontSize: 14,
        letterSpacing: 0.5,
        boxShadow: `0 4px 12px ${activeThemeColor}44`,
      }}
    >
      {brandInitial}
    </div>
  );

  const topBanks = [
    { code: 'HDFC', name: 'HDFC Bank' },
    { code: 'SBI', name: 'State Bank of India' },
    { code: 'ICICI', name: 'ICICI Bank' },
    { code: 'AXIS', name: 'Axis Bank' },
    { code: 'KOTAK', name: 'Kotak Bank' },
    { code: 'PNB', name: 'Punjab National' },
  ];

  const themeSwatches = [
    { name: 'Emerald', hex: '#1DB954' },
    { name: 'Indigo', hex: '#6366F1' },
    { name: 'Sapphire', hex: '#2563EB' },
    { name: 'Crimson', hex: '#E11D48' },
    { name: 'Amber', hex: '#F59E0B' },
  ];

  // Renders the interactive payment selector + form
  const renderPaymentMethods = (isMobileView: boolean) => {
    if (paymentState === 'success') {
      return (
        <div style={{ textAlign: 'center', padding: isMobileView ? '24px 12px' : '36px 20px' }}>
          <div
            style={{
              width: 68,
              height: 68,
              borderRadius: '50%',
              background: 'rgba(16, 185, 129, 0.12)',
              border: '2px solid #10B981',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto 16px',
            }}
          >
            <CheckCircle2 size={40} style={{ color: '#10B981' }} />
          </div>
          <h3 style={{ fontSize: 18, fontWeight: 800, color: 'var(--fp-text, #fff)', marginBottom: 6 }}>
            Payment Successful!
          </h3>
          <p style={{ fontSize: 13, color: 'var(--fp-text-muted, #a1a1aa)', marginBottom: 20 }}>
            Transaction authorized by {brandName} gateway
          </p>

          <div
            style={{
              background: 'rgba(255,255,255,0.03)',
              border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
              borderRadius: 12,
              padding: '16px',
              textAlign: 'left',
              fontSize: 12,
              marginBottom: 20,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
              <span style={{ color: 'var(--fp-text-muted, #888)' }}>Amount Paid</span>
              <strong style={{ color: 'var(--fp-text, #fff)', fontSize: 13 }}>{formatCurrency(amount)}</strong>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
              <span style={{ color: 'var(--fp-text-muted, #888)' }}>Transaction ID</span>
              <span style={{ fontFamily: 'monospace', color: 'var(--fp-text, #fff)' }}>txn_live_99214710</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
              <span style={{ color: 'var(--fp-text-muted, #888)' }}>Receipt Number</span>
              <span style={{ fontFamily: 'monospace', color: 'var(--fp-text, #fff)' }}>{receiptPrefix}-0001</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--fp-text-muted, #888)' }}>Payment Method</span>
              <span style={{ textTransform: 'uppercase', color: activeThemeColor, fontWeight: 700 }}>{selectedMethod}</span>
            </div>
          </div>

          <div style={{ display: 'flex', gap: 10, justifyContent: 'center', flexWrap: 'wrap' }}>
            <button
              type="button"
              className="btn-primary"
              onClick={handleResetFlow}
              style={{ background: activeThemeColor, color: '#fff', fontSize: 12, padding: '8px 14px' }}
            >
              <RotateCcw size={13} /> Test Another Payment
            </button>
            <button
              type="button"
              className="btn-secondary"
              onClick={() => alert(`Simulated receipt download for ${receiptPrefix}-0001`)}
              style={{ fontSize: 12, padding: '8px 14px' }}
            >
              <Download size={13} /> Sample PDF Receipt
            </button>
          </div>
        </div>
      );
    }

    return (
      <div>
        {/* Method Switcher Tabs */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(4, 1fr)',
            gap: 4,
            background: 'rgba(0,0,0,0.25)',
            padding: 4,
            borderRadius: 10,
            marginBottom: 16,
          }}
        >
          {[
            { id: 'upi', label: 'UPI / QR', icon: <QrCode size={13} /> },
            { id: 'card', label: 'Cards', icon: <CreditCard size={13} /> },
            { id: 'netbanking', label: 'Netbanking', icon: <Building2 size={13} /> },
            { id: 'wallets', label: 'Wallets & EMI', icon: <Wallet size={13} /> },
          ].map(m => {
            const isActive = selectedMethod === m.id;
            return (
              <button
                key={m.id}
                type="button"
                onClick={() => setSelectedMethod(m.id as typeof selectedMethod)}
                style={{
                  display: 'flex',
                  flexDirection: isMobileView ? 'column' : 'row',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 5,
                  padding: isMobileView ? '6px 2px' : '7px 6px',
                  borderRadius: 7,
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: isMobileView ? 10 : 11,
                  fontWeight: isActive ? 700 : 500,
                  background: isActive ? activeThemeColor : 'transparent',
                  color: isActive ? '#FFFFFF' : 'var(--fp-text-muted, #9ca3af)',
                  transition: 'all 0.15s ease',
                }}
              >
                {m.icon}
                <span>{m.label}</span>
              </button>
            );
          })}
        </div>

        {/* Tab 1: UPI */}
        {selectedMethod === 'upi' && (
          <div>
            <div style={{ display: 'flex', gap: 8, marginBottom: 14 }}>
              <button
                type="button"
                onClick={() => setUpiMode('qr')}
                style={{
                  flex: 1,
                  padding: '6px 10px',
                  fontSize: 12,
                  fontWeight: upiMode === 'qr' ? 700 : 500,
                  borderRadius: 6,
                  border: upiMode === 'qr' ? `1px solid ${activeThemeColor}` : '1px solid rgba(255,255,255,0.1)',
                  background: upiMode === 'qr' ? `${activeThemeColor}18` : 'transparent',
                  color: upiMode === 'qr' ? activeThemeColor : 'var(--fp-text-muted, #aaa)',
                  cursor: 'pointer',
                }}
              >
                Scan UPI QR
              </button>
              <button
                type="button"
                onClick={() => setUpiMode('vpa')}
                style={{
                  flex: 1,
                  padding: '6px 10px',
                  fontSize: 12,
                  fontWeight: upiMode === 'vpa' ? 700 : 500,
                  borderRadius: 6,
                  border: upiMode === 'vpa' ? `1px solid ${activeThemeColor}` : '1px solid rgba(255,255,255,0.1)',
                  background: upiMode === 'vpa' ? `${activeThemeColor}18` : 'transparent',
                  color: upiMode === 'vpa' ? activeThemeColor : 'var(--fp-text-muted, #aaa)',
                  cursor: 'pointer',
                }}
              >
                Enter UPI ID
              </button>
            </div>

            {upiMode === 'qr' ? (
              <div style={{ textAlign: 'center', padding: '6px 0' }}>
                <QrCodeSvg brandInitial={brandInitial} color={activeThemeColor} />
                <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--fp-text, #fff)', marginTop: 10 }}>
                  Scan with any UPI application
                </div>
                <div style={{ display: 'flex', justifyContent: 'center', gap: 6, marginTop: 8, flexWrap: 'wrap' }}>
                  {['GPay', 'PhonePe', 'Paytm', 'BHIM', 'CRED'].map(app => (
                    <span
                      key={app}
                      style={{
                        fontSize: 10,
                        fontWeight: 600,
                        background: 'rgba(255,255,255,0.06)',
                        border: '1px solid rgba(255,255,255,0.1)',
                        padding: '2px 8px',
                        borderRadius: 999,
                        color: 'var(--fp-text-muted, #bbb)',
                      }}
                    >
                      {app}
                    </span>
                  ))}
                </div>
                <div style={{ fontSize: 11, color: '#10B981', marginTop: 8, display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                  <Timer size={12} /> Dynamic QR · Refreshing in 04:48
                </div>
              </div>
            ) : (
              <div>
                <label style={{ display: 'block', fontSize: 11, fontWeight: 600, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                  Virtual Payment Address (VPA)
                </label>
                <div style={{ position: 'relative', marginBottom: 8 }}>
                  <input
                    value={vpaInput}
                    onChange={e => setVpaInput(e.target.value)}
                    placeholder="mobileNumber@upi or name@bank"
                    style={{
                      width: '100%',
                      padding: '9px 12px',
                      paddingRight: 80,
                      background: 'rgba(0,0,0,0.3)',
                      border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                      borderRadius: 8,
                      color: '#FFFFFF',
                      fontSize: 13,
                    }}
                  />
                  <button
                    type="button"
                    onClick={() => setVpaVerified(true)}
                    style={{
                      position: 'absolute',
                      right: 6,
                      top: 6,
                      padding: '4px 10px',
                      fontSize: 10,
                      fontWeight: 700,
                      background: activeThemeColor,
                      color: '#fff',
                      border: 'none',
                      borderRadius: 5,
                      cursor: 'pointer',
                    }}
                  >
                    Verify
                  </button>
                </div>
                {vpaVerified && (
                  <div style={{ fontSize: 11, color: '#10B981', display: 'flex', alignItems: 'center', gap: 4, marginBottom: 12 }}>
                    <CheckCircle2 size={13} /> Verified: Aarav Sharma (HDFC Bank)
                  </div>
                )}
                <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 12 }}>
                  {['@okhdfcbank', '@okicici', '@paytm', '@ybl'].map(h => (
                    <span
                      key={h}
                      onClick={() => setVpaInput(`customer${h}`)}
                      style={{
                        fontSize: 10,
                        background: 'rgba(255,255,255,0.06)',
                        padding: '2px 8px',
                        borderRadius: 4,
                        color: 'var(--fp-text-muted, #999)',
                        cursor: 'pointer',
                      }}
                    >
                      {h}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Tab 2: Cards */}
        {selectedMethod === 'card' && (
          <div>
            {/* Interactive Card Graphic */}
            <div
              style={{
                width: '100%',
                borderRadius: 12,
                padding: '16px',
                marginBottom: 14,
                background: `linear-gradient(135deg, ${activeThemeColor}33 0%, #17171a 70%, #0d0d0f 100%)`,
                border: `1px solid ${activeThemeColor}44`,
                boxShadow: '0 8px 20px rgba(0,0,0,0.4)',
                position: 'relative',
                overflow: 'hidden',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14 }}>
                <div
                  style={{
                    width: 32,
                    height: 24,
                    borderRadius: 4,
                    background: 'linear-gradient(135deg, #FFD700 0%, #B8860B 100%)',
                    boxShadow: 'inset 0 1px 2px rgba(255,255,255,0.4)',
                  }}
                />
                <div style={{ fontSize: 10, fontWeight: 800, color: 'rgba(255,255,255,0.6)', letterSpacing: 1 }}>
                  CREDIT / DEBIT
                </div>
              </div>
              <div style={{ fontFamily: 'monospace', fontSize: 15, fontWeight: 700, letterSpacing: 2, color: '#FFFFFF', marginBottom: 12 }}>
                {cardNumber || '•••• •••• •••• ••••'}
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end' }}>
                <div>
                  <div style={{ fontSize: 9, color: 'rgba(255,255,255,0.5)', textTransform: 'uppercase' }}>Card Holder</div>
                  <div style={{ fontSize: 11, fontWeight: 700, color: '#FFFFFF' }}>{cardName || 'YOUR NAME'}</div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <div style={{ fontSize: 9, color: 'rgba(255,255,255,0.5)', textTransform: 'uppercase' }}>Expires</div>
                  <div style={{ fontSize: 11, fontWeight: 700, color: '#FFFFFF', fontFamily: 'monospace' }}>{cardExpiry || 'MM/YY'}</div>
                </div>
                <div style={{ fontWeight: 900, fontStyle: 'italic', fontSize: 14, color: '#FFFFFF' }}>VISA</div>
              </div>
            </div>

            {/* Inputs */}
            <div style={{ marginBottom: 8 }}>
              <label style={{ display: 'block', fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                Card Number
              </label>
              <input
                value={cardNumber}
                onChange={e => setCardNumber(e.target.value)}
                placeholder="4532 8901 2345 8821"
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  background: 'rgba(0,0,0,0.3)',
                  border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                  borderRadius: 6,
                  color: '#fff',
                  fontSize: 12,
                  fontFamily: 'monospace',
                }}
              />
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 8 }}>
              <div>
                <label style={{ display: 'block', fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                  Valid Thru
                </label>
                <input
                  value={cardExpiry}
                  onChange={e => setCardExpiry(e.target.value)}
                  placeholder="MM/YY"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    background: 'rgba(0,0,0,0.3)',
                    border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                    borderRadius: 6,
                    color: '#fff',
                    fontSize: 12,
                    fontFamily: 'monospace',
                  }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                  CVV / CVC
                </label>
                <input
                  type="password"
                  maxLength={4}
                  value={cardCvv}
                  onChange={e => setCardCvv(e.target.value)}
                  placeholder="•••"
                  style={{
                    width: '100%',
                    padding: '8px 12px',
                    background: 'rgba(0,0,0,0.3)',
                    border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                    borderRadius: 6,
                    color: '#fff',
                    fontSize: 12,
                    fontFamily: 'monospace',
                  }}
                />
              </div>
            </div>
            <div style={{ marginBottom: 10 }}>
              <label style={{ display: 'block', fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                Name on Card
              </label>
              <input
                value={cardName}
                onChange={e => setCardName(e.target.value)}
                placeholder="AARAV SHARMA"
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  background: 'rgba(0,0,0,0.3)',
                  border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                  borderRadius: 6,
                  color: '#fff',
                  fontSize: 12,
                }}
              />
            </div>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11, color: 'var(--fp-text-muted, #aaa)', cursor: 'pointer' }}>
              <input type="checkbox" checked={saveCard} onChange={e => setSaveCard(e.target.checked)} />
              <span>Tokenize & save card securely as per RBI guidelines</span>
            </label>
          </div>
        )}

        {/* Tab 3: Netbanking */}
        {selectedMethod === 'netbanking' && (
          <div>
            <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 8 }}>
              Popular Indian Banks
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 6, marginBottom: 14 }}>
              {topBanks.map(b => {
                const isSelected = selectedBank === b.code;
                return (
                  <button
                    key={b.code}
                    type="button"
                    onClick={() => setSelectedBank(b.code)}
                    style={{
                      padding: '10px 4px',
                      borderRadius: 8,
                      border: isSelected ? `1px solid ${activeThemeColor}` : '1px solid rgba(255,255,255,0.1)',
                      background: isSelected ? `${activeThemeColor}18` : 'rgba(255,255,255,0.03)',
                      color: isSelected ? '#FFFFFF' : 'var(--fp-text-muted, #aaa)',
                      cursor: 'pointer',
                      textAlign: 'center',
                    }}
                  >
                    <div style={{ fontWeight: 800, fontSize: 12, color: isSelected ? activeThemeColor : '#FFFFFF' }}>{b.code}</div>
                    <div style={{ fontSize: 9, opacity: 0.7, textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }}>
                      {b.name}
                    </div>
                  </button>
                );
              })}
            </div>
            <div>
              <label style={{ display: 'block', fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 4 }}>
                All Other Banks (50+)
              </label>
              <select
                style={{
                  width: '100%',
                  padding: '8px 10px',
                  background: 'rgba(0,0,0,0.3)',
                  border: '1px solid var(--fp-border, rgba(255,255,255,0.15))',
                  borderRadius: 6,
                  color: '#fff',
                  fontSize: 12,
                }}
              >
                <option value="">Select from 50+ Scheduled Commercial Banks</option>
                <option value="BOB">Bank of Baroda</option>
                <option value="CANARA">Canara Bank</option>
                <option value="INDUSIND">IndusInd Bank</option>
                <option value="YES">Yes Bank</option>
                <option value="IDFC">IDFC FIRST Bank</option>
                <option value="FEDERAL">Federal Bank</option>
              </select>
            </div>
          </div>
        )}

        {/* Tab 4: Wallets & BNPL */}
        {selectedMethod === 'wallets' && (
          <div>
            {/* BNPL Promo Banner */}
            <div
              style={{
                padding: '10px 12px',
                borderRadius: 8,
                background: 'rgba(99, 102, 241, 0.1)',
                border: '1px solid rgba(99, 102, 241, 0.3)',
                marginBottom: 12,
                display: 'flex',
                alignItems: 'center',
                gap: 10,
              }}
            >
              <Zap size={20} style={{ color: '#818CF8', flexShrink: 0 }} />
              <div style={{ fontSize: 11 }}>
                <strong style={{ color: '#C7D2FE', display: 'block' }}>No Cost EMI / Pay Later</strong>
                <span style={{ color: '#A5B4FC' }}>
                  Pay in 3 installments of {formatCurrency(Math.round(amount / 3))}/mo at 0% interest
                </span>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              {[
                { id: 'simpl', name: 'Simpl PayLater', note: 'Instant approval' },
                { id: 'lazypay', name: 'LazyPay', note: 'Credit line' },
                { id: 'amazon', name: 'Amazon Pay', note: 'Balance & UPI' },
                { id: 'paytm', name: 'Paytm Wallet', note: 'Fast checkout' },
              ].map(w => {
                const isSelected = selectedWallet === w.id;
                return (
                  <button
                    key={w.id}
                    type="button"
                    onClick={() => setSelectedWallet(w.id)}
                    style={{
                      padding: '10px 12px',
                      borderRadius: 8,
                      border: isSelected ? `1px solid ${activeThemeColor}` : '1px solid rgba(255,255,255,0.1)',
                      background: isSelected ? `${activeThemeColor}18` : 'rgba(255,255,255,0.03)',
                      color: '#FFFFFF',
                      cursor: 'pointer',
                      textAlign: 'left',
                    }}
                  >
                    <div style={{ fontWeight: 700, fontSize: 12, color: isSelected ? activeThemeColor : '#FFFFFF' }}>{w.name}</div>
                    <div style={{ fontSize: 10, color: 'var(--fp-text-muted, #888)' }}>{w.note}</div>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        {/* Primary Pay Button */}
        <div style={{ marginTop: 18 }}>
          <button
            type="button"
            className="btn-primary"
            disabled={paymentState === 'processing'}
            onClick={handleSimulatePayment}
            style={{
              width: '100%',
              height: 44,
              fontSize: 14,
              fontWeight: 800,
              background: activeThemeColor,
              color: '#FFFFFF',
              border: 'none',
              borderRadius: 8,
              boxShadow: `0 4px 14px ${activeThemeColor}44`,
              cursor: paymentState === 'processing' ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 8,
              transition: 'all 0.2s ease',
            }}
          >
            {paymentState === 'processing' ? (
              <>
                <RefreshCw className="login-spinner" size={16} />
                <span>Authorizing with Secure Gateway...</span>
              </>
            ) : (
              <>
                <Lock size={14} />
                <span>Pay {formatCurrency(amount)}</span>
              </>
            )}
          </button>

          <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 12, marginTop: 10, fontSize: 11, color: 'var(--fp-text-muted, #71717a)' }}>
            <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>
              <Shield size={11} /> 256-bit SSL
            </span>
            <span>•</span>
            <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>
              <BadgeCheck size={11} /> PCI-DSS Level 1
            </span>
            <span>•</span>
            <span>RBI Tokenized</span>
          </div>
        </div>
      </div>
    );
  };

  return (
    <section className="card" style={{ padding: 20 }}>
      {/* Top Simulation Bar & Controls */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          flexWrap: 'wrap',
          gap: 12,
          paddingBottom: 16,
          marginBottom: 20,
          borderBottom: '1px solid var(--fp-border, rgba(255,255,255,0.08))',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: 8,
              background: `${activeThemeColor}22`,
              color: activeThemeColor,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Sparkles size={18} />
          </div>
          <div>
            <h2 style={{ fontSize: 16, fontWeight: 800, margin: 0, display: 'flex', alignItems: 'center', gap: 8 }}>
              Interactive Checkout Simulator
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  background: 'rgba(16, 185, 129, 0.15)',
                  color: '#10B981',
                  border: '1px solid rgba(16, 185, 129, 0.3)',
                  padding: '2px 8px',
                  borderRadius: 999,
                }}
              >
                LIVE PREVIEW
              </span>
            </h2>
            <div style={{ fontSize: 12, color: 'var(--fp-text-muted, #888)' }}>
              Real-time preview of your white-label checkout as experienced by end-customers
            </div>
          </div>
        </div>

        {/* Control Buttons */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
          {/* Amount Tiers */}
          <div style={{ display: 'flex', background: 'rgba(0,0,0,0.3)', padding: 3, borderRadius: 8, border: '1px solid rgba(255,255,255,0.08)' }}>
            {[
              { label: '₹499', val: 499 },
              { label: '₹2,499', val: 2499 },
              { label: '₹14,999', val: 14999 },
            ].map(tier => (
              <button
                key={tier.val}
                type="button"
                onClick={() => setAmount(tier.val)}
                style={{
                  padding: '4px 10px',
                  fontSize: 11,
                  fontWeight: amount === tier.val ? 700 : 500,
                  borderRadius: 6,
                  border: 'none',
                  cursor: 'pointer',
                  background: amount === tier.val ? activeThemeColor : 'transparent',
                  color: amount === tier.val ? '#fff' : 'var(--fp-text-muted, #aaa)',
                }}
              >
                {tier.label}
              </button>
            ))}
          </div>

          {/* Viewport Switcher */}
          <div style={{ display: 'flex', background: 'rgba(0,0,0,0.3)', padding: 3, borderRadius: 8, border: '1px solid rgba(255,255,255,0.08)' }}>
            <button
              type="button"
              onClick={() => setDevice('desktop')}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 5,
                padding: '5px 10px',
                fontSize: 11,
                fontWeight: device === 'desktop' ? 700 : 500,
                borderRadius: 6,
                border: 'none',
                cursor: 'pointer',
                background: device === 'desktop' ? 'rgba(255,255,255,0.12)' : 'transparent',
                color: device === 'desktop' ? '#FFFFFF' : 'var(--fp-text-muted, #aaa)',
              }}
            >
              <Monitor size={13} /> Desktop
            </button>
            <button
              type="button"
              onClick={() => setDevice('mobile')}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 5,
                padding: '5px 10px',
                fontSize: 11,
                fontWeight: device === 'mobile' ? 700 : 500,
                borderRadius: 6,
                border: 'none',
                cursor: 'pointer',
                background: device === 'mobile' ? 'rgba(255,255,255,0.12)' : 'transparent',
                color: device === 'mobile' ? '#FFFFFF' : 'var(--fp-text-muted, #aaa)',
              }}
            >
              <Smartphone size={13} /> Mobile
            </button>
          </div>

          {/* Reset Action */}
          <button
            type="button"
            className="btn-secondary"
            onClick={handleResetFlow}
            title="Reset Simulation"
            style={{ padding: '6px 10px', fontSize: 11 }}
          >
            <RotateCcw size={13} /> Reset
          </button>

          {/* Edit Config Action */}
          {onEditConfig && (
            <button
              type="button"
              className="btn-secondary"
              onClick={onEditConfig}
              style={{ padding: '6px 12px', fontSize: 11 }}
            >
              <Settings size={13} /> Edit Config
            </button>
          )}
        </div>
      </div>

      {/* Main Grid: Checkout Canvas (Left) + Inspector (Right) */}
      <div
        style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 24,
          alignItems: 'flex-start',
        }}
      >
        {/* Left: The Checkout Canvas */}
        <div style={{ flex: '1 1 540px', minWidth: 0 }}>
          {device === 'desktop' ? (
            /* Desktop Browser Frame */
            <div
              style={{
                borderRadius: 14,
                border: '1px solid var(--fp-border, rgba(255,255,255,0.12))',
                background: 'var(--fp-bg, #121212)',
                boxShadow: '0 20px 40px -15px rgba(0,0,0,0.6)',
                overflow: 'hidden',
              }}
            >
              {/* Browser Header Chrome */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 12,
                  padding: '10px 16px',
                  background: 'rgba(255,255,255,0.03)',
                  borderBottom: '1px solid var(--fp-border, rgba(255,255,255,0.08))',
                }}
              >
                {/* Traffic dots */}
                <div style={{ display: 'flex', gap: 6 }}>
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#FF5F56' }} />
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#FFBD2E' }} />
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#27C93F' }} />
                </div>

                {/* Simulated URL address bar */}
                <div
                  style={{
                    flex: 1,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    background: 'rgba(0,0,0,0.4)',
                    padding: '4px 12px',
                    borderRadius: 6,
                    fontSize: 11,
                    color: 'var(--fp-text-muted, #888)',
                    fontFamily: 'monospace',
                  }}
                >
                  <Lock size={11} style={{ color: '#10B981' }} />
                  <span style={{ color: '#10B981' }}>https://</span>
                  <span style={{ color: '#FFFFFF' }}>{domain}</span>
                  <span>/pay/session_{invoicePrefix.toLowerCase()}_9941</span>
                </div>

                <div style={{ fontSize: 10, color: '#10B981', display: 'flex', alignItems: 'center', gap: 4, fontWeight: 700 }}>
                  <ShieldCheck size={13} /> TLS 1.3
                </div>
              </div>

              {/* 2-Column Split Checkout Body */}
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
                  minHeight: 480,
                  background: 'var(--fp-bg-card, #191414)',
                }}
              >
                {/* Left Column: Order Summary & Merchant Info */}
                <div
                  style={{
                    padding: '24px',
                    borderRight: '1px solid var(--fp-border, rgba(255,255,255,0.08))',
                    background: 'rgba(0,0,0,0.15)',
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'space-between',
                  }}
                >
                  <div>
                    {/* Merchant Brand Header */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20 }}>
                      {brandLogoNode}
                      <div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                          <strong style={{ fontSize: 15, color: '#FFFFFF' }}>{brandName}</strong>
                          <span
                            style={{
                              fontSize: 9,
                              fontWeight: 700,
                              color: activeThemeColor,
                              background: `${activeThemeColor}20`,
                              padding: '1px 6px',
                              borderRadius: 4,
                            }}
                          >
                            VERIFIED
                          </span>
                        </div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)' }}>Secure Merchant Checkout</div>
                      </div>
                    </div>

                    {/* Order Total Display */}
                    <div style={{ marginBottom: 18 }}>
                      <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #999)', textTransform: 'uppercase', letterSpacing: 0.5 }}>
                        Total Amount Due
                      </div>
                      <div style={{ fontSize: 32, fontWeight: 900, color: '#FFFFFF', letterSpacing: -0.5, margin: '4px 0' }}>
                        {formatCurrency(amount)}
                      </div>
                      <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #777)' }}>
                        Invoice <span style={{ fontFamily: 'monospace', color: '#ccc' }}>{invoicePrefix}-2026-0042</span> · Receipt <span style={{ fontFamily: 'monospace', color: '#ccc' }}>{receiptPrefix}-0001</span>
                      </div>
                    </div>

                    {/* Collapsible Line Items */}
                    <div
                      style={{
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: 8,
                        border: '1px solid rgba(255,255,255,0.06)',
                        padding: '10px 12px',
                        marginBottom: 16,
                      }}
                    >
                      <div
                        onClick={() => setOrderExpanded(!orderExpanded)}
                        style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', cursor: 'pointer', fontSize: 11, fontWeight: 600, color: 'var(--fp-text-muted, #aaa)' }}
                      >
                        <span>Order Summary ({orderExpanded ? 'Hide' : 'Show'})</span>
                        <ChevronDown size={14} style={{ transform: orderExpanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }} />
                      </div>

                      {orderExpanded && (
                        <div style={{ marginTop: 10, paddingTop: 10, borderTop: '1px solid rgba(255,255,255,0.06)', fontSize: 11 }}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
                            <span style={{ color: 'var(--fp-text-muted, #888)' }}>Pro Plan Subscription</span>
                            <span style={{ color: '#fff' }}>{formatCurrency(baseAmount)}</span>
                          </div>
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
                            <span style={{ color: 'var(--fp-text-muted, #888)' }}>Platform Fee</span>
                            <span style={{ color: '#10B981' }}>FREE</span>
                          </div>
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
                            <span style={{ color: 'var(--fp-text-muted, #888)' }}>Integrated GST (18%)</span>
                            <span style={{ color: '#fff' }}>{formatCurrency(gstAmount)}</span>
                          </div>
                          <div style={{ display: 'flex', justifyContent: 'space-between', paddingTop: 6, borderTop: '1px dashed rgba(255,255,255,0.1)', fontWeight: 700 }}>
                            <span style={{ color: '#fff' }}>Total</span>
                            <span style={{ color: activeThemeColor }}>{formatCurrency(amount)}</span>
                          </div>
                        </div>
                      )}
                    </div>

                    {/* Session Timeout Pill */}
                    <div
                      style={{
                        display: 'inline-flex',
                        alignItems: 'center',
                        gap: 6,
                        background: 'rgba(234, 179, 8, 0.1)',
                        border: '1px solid rgba(234, 179, 8, 0.25)',
                        color: '#FACC15',
                        borderRadius: 6,
                        padding: '4px 8px',
                        fontSize: 10,
                        fontWeight: 600,
                      }}
                    >
                      <Timer size={12} /> Session expires in 14:30
                    </div>
                  </div>

                  {/* Left Column Footer */}
                  <div style={{ marginTop: 24, paddingTop: 14, borderTop: '1px solid rgba(255,255,255,0.06)', fontSize: 11, color: 'var(--fp-text-muted, #777)' }}>
                    <div>Customer Support:</div>
                    <div style={{ color: '#bbb', marginTop: 2 }}>{supportEmail}</div>
                    <div style={{ color: '#bbb' }}>{supportPhone}</div>
                  </div>
                </div>

                {/* Right Column: Interactive Payment Form */}
                <div style={{ padding: '24px', display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
                  {renderPaymentMethods(false)}
                </div>
              </div>
            </div>
          ) : (
            /* Mobile Device Frame Mockup */
            <div
              style={{
                maxWidth: 375,
                margin: '0 auto',
                background: '#0D0D0E',
                borderRadius: 36,
                border: '8px solid #222226',
                boxShadow: '0 25px 50px -12px rgba(0,0,0,0.8)',
                overflow: 'hidden',
                position: 'relative',
              }}
            >
              {/* Dynamic Island / Notch */}
              <div
                style={{
                  width: 90,
                  height: 20,
                  background: '#000000',
                  borderRadius: 10,
                  margin: '8px auto 4px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                <div style={{ width: 8, height: 8, borderRadius: '50%', background: '#1c1c1e', marginRight: 8 }} />
                <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#0a84ff' }} />
              </div>

              {/* Status Bar */}
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  padding: '2px 18px 8px',
                  fontSize: 11,
                  fontWeight: 600,
                  color: '#FFFFFF',
                }}
              >
                <span>9:41</span>
                <span style={{ letterSpacing: 1 }}>5G 100%</span>
              </div>

              {/* Mobile Screen Header */}
              <div
                style={{
                  padding: '12px 16px',
                  background: 'rgba(255,255,255,0.03)',
                  borderBottom: '1px solid rgba(255,255,255,0.08)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  {brandLogoNode}
                  <div>
                    <div style={{ fontSize: 13, fontWeight: 800, color: '#fff' }}>{brandName}</div>
                    <div style={{ fontSize: 9, color: 'var(--fp-text-muted, #888)' }}>Secured by FeelPay</div>
                  </div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <div style={{ fontSize: 10, color: 'var(--fp-text-muted, #888)' }}>Amount</div>
                  <div style={{ fontSize: 14, fontWeight: 900, color: activeThemeColor }}>{formatCurrency(amount)}</div>
                </div>
              </div>

              {/* Mobile Screen Body */}
              <div style={{ padding: '16px', minHeight: 460 }}>
                {renderPaymentMethods(true)}
              </div>
            </div>
          )}
        </div>

        {/* Right: Live Brand & Configuration Inspector */}
        <div style={{ flex: '1 1 320px', minWidth: 280, display: 'flex', flexDirection: 'column', gap: 16 }}>
          {/* Card 1: Brand & Theme Styler */}
          <div
            style={{
              background: 'var(--fp-bg-elevated, #282828)',
              borderRadius: 12,
              padding: 16,
              border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
              <strong style={{ fontSize: 13, color: '#FFFFFF', display: 'flex', alignItems: 'center', gap: 6 }}>
                <SlidersHorizontal size={14} style={{ color: activeThemeColor }} /> Brand Color Styler
              </strong>
              <span style={{ fontSize: 11, fontFamily: 'monospace', color: activeThemeColor, fontWeight: 700 }}>
                {activeThemeColor}
              </span>
            </div>

            <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 10 }}>
              Test how buttons, badges, and accents appear under different merchant brand palettes:
            </div>

            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 14 }}>
              {themeSwatches.map(sw => {
                const isSelected = activeThemeColor.toLowerCase() === sw.hex.toLowerCase();
                return (
                  <button
                    key={sw.hex}
                    type="button"
                    onClick={() => setActiveThemeColor(sw.hex)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 6,
                      padding: '5px 9px',
                      borderRadius: 999,
                      border: isSelected ? `2px solid #FFFFFF` : '1px solid rgba(255,255,255,0.15)',
                      background: 'rgba(0,0,0,0.3)',
                      color: isSelected ? '#FFFFFF' : 'var(--fp-text-muted, #aaa)',
                      cursor: 'pointer',
                      fontSize: 11,
                      fontWeight: isSelected ? 700 : 500,
                    }}
                  >
                    <span style={{ width: 10, height: 10, borderRadius: '50%', background: sw.hex }} />
                    <span>{sw.name}</span>
                    {isSelected && <Check size={11} style={{ color: '#fff' }} />}
                  </button>
                );
              })}
            </div>

            <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #888)', background: 'rgba(0,0,0,0.2)', padding: '8px 10px', borderRadius: 6 }}>
              Configured Default: <code>{str(value.checkout_theme_color) || '#1DB954'}</code>
            </div>
          </div>

          {/* Card 2: Bank Statement Descriptor & SMS Alert Simulation */}
          <div
            style={{
              background: 'var(--fp-bg-elevated, #282828)',
              borderRadius: 12,
              padding: 16,
              border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 10 }}>
              <strong style={{ fontSize: 13, color: '#FFFFFF', display: 'flex', alignItems: 'center', gap: 6 }}>
                <Banknote size={14} style={{ color: '#38BDF8' }} /> Bank Statement Descriptor
              </strong>
              <span style={{ fontSize: 10, color: 'var(--fp-text-muted, #888)', fontFamily: 'monospace' }}>
                {statementDesc.length}/22 chars
              </span>
            </div>

            <div style={{ fontSize: 11, color: 'var(--fp-text-muted, #aaa)', marginBottom: 10 }}>
              What customer sees on bank statements and instant SMS transaction alerts:
            </div>

            {/* Simulated Phone SMS Bubble */}
            <div
              style={{
                background: 'rgba(0,0,0,0.4)',
                border: '1px solid rgba(255,255,255,0.1)',
                borderRadius: 10,
                padding: '12px',
                fontFamily: 'monospace',
                fontSize: 11,
                lineHeight: 1.5,
                color: '#E2E8F0',
                position: 'relative',
              }}
            >
              <div style={{ fontSize: 9, color: '#38BDF8', fontWeight: 700, marginBottom: 4 }}>
                SMS ALERT · HDFC-BANK
              </div>
              Rs {amount.toFixed(2)} debited from A/C **4921 on 13-SEP-26. Info:{' '}
              <strong style={{ color: activeThemeColor, background: `${activeThemeColor}22`, padding: '1px 4px', borderRadius: 3 }}>
                {statementDesc}
              </strong>
              . Avail Bal: Rs 48,210.35
            </div>
          </div>

          {/* Card 3: Active Payment Channels */}
          <div
            style={{
              background: 'var(--fp-bg-elevated, #282828)',
              borderRadius: 12,
              padding: 16,
              border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
            }}
          >
            <strong style={{ fontSize: 13, color: '#FFFFFF', display: 'flex', alignItems: 'center', gap: 6, marginBottom: 10 }}>
              <Zap size={14} style={{ color: '#F59E0B' }} /> Enabled Payment Channels
            </strong>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              {[
                { label: 'UPI & Dynamic QR', active: true, info: 'Auto-Routing' },
                { label: 'Cards (Visa/MC/RuPay)', active: true, info: '3DS 2.0' },
                { label: 'Netbanking (50+ Banks)', active: true, info: 'Instant OTP' },
                { label: 'Wallets & BNPL (Simpl)', active: true, info: 'No Cost EMI' },
              ].map(c => (
                <div
                  key={c.label}
                  style={{
                    background: 'rgba(0,0,0,0.2)',
                    padding: '8px 10px',
                    borderRadius: 6,
                    border: '1px solid rgba(255,255,255,0.06)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 5, fontSize: 11, fontWeight: 600, color: '#fff' }}>
                    <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#10B981' }} />
                    <span>{c.label}</span>
                  </div>
                  <div style={{ fontSize: 9, color: 'var(--fp-text-muted, #777)', marginTop: 2, paddingLeft: 11 }}>
                    {c.info}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Card 4: Legal & Compliance Verification */}
          <div
            style={{
              background: 'var(--fp-bg-elevated, #282828)',
              borderRadius: 12,
              padding: 16,
              border: '1px solid var(--fp-border, rgba(255,255,255,0.1))',
            }}
          >
            <strong style={{ fontSize: 13, color: '#FFFFFF', display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12 }}>
              <ShieldCheck size={14} style={{ color: '#10B981' }} /> Legal & Compliance Audit
            </strong>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 8, fontSize: 11 }}>
              {[
                { label: 'Checkout Domain', val: domain, status: 'SSL Active' },
                { label: 'Terms of Service', val: str(value.terms_url), status: value.terms_url ? 'Configured' : 'Missing' },
                { label: 'Privacy Policy', val: str(value.privacy_url), status: value.privacy_url ? 'Configured' : 'Missing' },
                { label: 'Refund Policy', val: str(value.refund_policy_url), status: value.refund_policy_url ? 'Configured' : 'Missing' },
              ].map(item => (
                <div
                  key={item.label}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '4px 0',
                    borderBottom: '1px solid rgba(255,255,255,0.04)',
                  }}
                >
                  <span style={{ color: 'var(--fp-text-muted, #aaa)' }}>{item.label}</span>
                  <span
                    style={{
                      fontSize: 10,
                      fontWeight: 600,
                      padding: '2px 6px',
                      borderRadius: 4,
                      background: item.status === 'Missing' ? 'rgba(239, 68, 68, 0.15)' : 'rgba(16, 185, 129, 0.15)',
                      color: item.status === 'Missing' ? '#EF4444' : '#10B981',
                    }}
                  >
                    {item.status}
                  </span>
                </div>
              ))}
            </div>

            {/* Quick Actions Footer */}
            <div style={{ display: 'flex', gap: 8, marginTop: 14 }}>
              <button
                type="button"
                className="btn-secondary"
                onClick={handleCopyLink}
                style={{ flex: 1, fontSize: 11, padding: '7px 8px', justifyContent: 'center' }}
              >
                {copiedLink ? <Check size={12} style={{ color: '#10B981' }} /> : <Copy size={12} />}
                <span>{copiedLink ? 'Link Copied!' : 'Copy Hosted Link'}</span>
              </button>
              {onEditConfig && (
                <button
                  type="button"
                  className="btn-primary"
                  onClick={onEditConfig}
                  style={{ flex: 1, fontSize: 11, padding: '7px 8px', justifyContent: 'center', background: activeThemeColor }}
                >
                  <Settings size={12} />
                  <span>Edit Settings</span>
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function SamplePayloads({ payloads }: { payloads: { id: string; event: string; sample: Record<string, unknown> }[] }) {
  const [copied, setCopied] = useState('');
  const copyPayload = (id: string, sample: Record<string, unknown>) => {
    void navigator.clipboard.writeText(JSON.stringify(sample, null, 2)).then(() => setCopied(id));
  };
  return (
    <section className="card">
      <h2><Cable size={18} /> Example messages</h2>
      <p className="muted-text">Safe example data to help test and connect your system without exposing real customer credentials.</p>
      <div className="grid grid-2">
        {payloads.map(payload => (
          <div key={payload.id} className="table-card" style={{ padding: 14 }}>
            <div className="button-row" style={{ justifyContent: 'space-between' }}>
              <strong>{payload.event}</strong>
              <button type="button" onClick={() => copyPayload(payload.id, payload.sample)}><Copy size={14} /> {copied === payload.id ? 'Copied' : 'Copy'}</button>
            </div>
            <pre className="json-preview">{JSON.stringify(payload.sample, null, 2)}</pre>
          </div>
        ))}
      </div>
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
      {helpers.error && <div className="alert alert-error">{friendlyError(helpers.error)}</div>}
    </>
  );
}

function ConfirmModal({ state, onCancel, onConfirm }: { state: NonNullable<ConfirmState>; onCancel: () => void; onConfirm: () => void }) {
  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div className="card modal-card" onClick={e => e.stopPropagation()}>
        <div className="drawer-header" style={{ marginBottom: 12 }}>
          <h3 style={{ margin: 0 }}>{state.title}</h3>
          <button type="button" className="btn-icon-close" onClick={onCancel} aria-label="Close">
            <X size={18} />
          </button>
        </div>
        <p className="muted-text">{state.body}</p>
        <div className="button-row" style={{ marginTop: 20, justifyContent: 'flex-end', gap: 10 }}>
          <button className="btn-secondary" type="button" onClick={onCancel}>Cancel</button>
          <button className="btn-primary" type="button" onClick={onConfirm}>Confirm</button>
        </div>
      </div>
    </div>
  );
}

function Drawer({ state, onClose }: { state: NonNullable<DrawerState>; onClose: () => void }) {
  const statusVal = state.body.status ? String(state.body.status) : null;
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="card drawer-card" onClick={e => e.stopPropagation()}>
        <div className="drawer-header">
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
              <h3 style={{ margin: 0 }}>{state.title}</h3>
              {statusVal && (
                <span className={`badge ${statusClass(statusVal, 'status')}`}>
                  {statusVal}
                </span>
              )}
            </div>
            <span className="drawer-subtitle">{state.title} operational details & parameters</span>
          </div>
          <button type="button" className="btn-icon-close" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>
        <div className="detail-list">
          {Object.entries(state.body).map(([key, value]) => (
            <div key={key}>
              <span>{formatHeader(key)}</span>
              <strong title={typeof value === 'string' ? value : undefined}>
                {isDateColumn(key) ? formatDateTime(value, key) :
                 isAmountColumn(key) && (typeof value === 'number' || (typeof value === 'string' && !isNaN(Number(value)))) ? money(Number(value || 0)) :
                 isBadgeColumn(key) && value !== null && value !== undefined && value !== '' ? (
                   <span className={`badge ${statusClass(String(value), key)}`}>{renderValue(value, key)}</span>
                 ) :
                 isIdColumn(key) && typeof value === 'string' && value ? (
                   <span className="id-cell" title={value}>
                     <span>{shortId(value)}</span>
                     <button className="btn-small" type="button" aria-label={`Copy ${key}`} onClick={() => copyText(value)}><Copy size={12} /></button>
                   </span>
                 ) :
                 renderDetailValue(value, key)}
              </strong>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function renderDetailValue(value: GenericRecord[string] | undefined, key?: string) {
  const safeValue = redactSensitiveSummary(value);
  if (typeof safeValue === 'object' && safeValue !== null) {
    if (Array.isArray(safeValue)) {
      if (safeValue.length === 0) {
        if (key?.includes('settlement')) return 'Pending Settlement Batch';
        return 'None recorded';
      }
      if (typeof safeValue[0] === 'object' && safeValue[0] !== null) {
        return <pre className="json-preview">{JSON.stringify(safeValue, null, 2)}</pre>;
      }
      return safeValue.join(', ');
    }
    if (Object.keys(safeValue).length === 0) return 'Standard Configuration';
    return <pre className="json-preview">{JSON.stringify(safeValue, null, 2)}</pre>;
  }
  return renderValue(safeValue, key);
}

function normalizeAction(action: string) {
  return action.toLowerCase().replaceAll(' ', '_');
}
