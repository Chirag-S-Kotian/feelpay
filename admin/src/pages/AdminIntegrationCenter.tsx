import { FormEvent, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  Bell,
  KeyRound,
  Landmark,
  Mail,
  RefreshCw,
  ShieldCheck,
  Smartphone,
} from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type ProviderKind =
  | 'email'
  | 'sms'
  | 'otp'
  | 'bank_payout'
  | 'kyc'
  | 'notification'
  | 'reconciliation';

type MaskedSecret = {
  secret_name: string;
  masked_value: string;
  rotated_at: string;
};

type IntegrationProvider = {
  id: string;
  provider_kind: ProviderKind;
  name: string;
  code: string;
  environment: 'test' | 'live';
  status: 'enabled' | 'disabled';
  is_default: boolean;
  priority: number;
  timeout_ms: number;
  max_retries: number;
  retry_backoff_ms: number;
  health_status: string;
  last_success_at?: string | null;
  last_failure_at?: string | null;
  last_error_code?: string | null;
  last_error_message?: string | null;
  config: Record<string, unknown>;
  masked_secrets: MaskedSecret[];
};

type IntegrationLog = {
  id: string;
  provider_id?: string | null;
  action: string;
  request_type: string;
  status: string;
  latency_ms: number;
  error_code?: string | null;
  error_message?: string | null;
  actor_admin_id?: string | null;
  created_at: string;
};

const KINDS: ProviderKind[] = [
  'email',
  'sms',
  'otp',
  'bank_payout',
  'kyc',
  'notification',
  'reconciliation',
];
const TEMPLATE_KEYS = ['onboarding', 'otp', 'payout', 'refund', 'dispute', 'api_key', 'webhook_failure'] as const;

const DEFAULT_FORM = {
  provider_kind: 'email' as ProviderKind,
  name: '',
  code: '',
  environment: 'test' as 'test' | 'live',
  status: 'disabled' as 'enabled' | 'disabled',
  is_default: false,
  priority: 100,
  timeout_ms: 10000,
  max_retries: 3,
  retry_backoff_ms: 1000,
  smtp_host: '',
  smtp_port: '587',
  smtp_user: '',
  smtp_security: 'starttls',
  from_name: '',
  from_email: '',
  reply_to: '',
  sender_id: '',
  route_type: '',
  callback_url: '',
  client_id: '',
  settlement_account_mapping: '',
  reconciliation_source: '',
  utr_reference_field: '',
  template_map: '',
};

const label = (value: string) => value.replaceAll('_', ' ');

export default function AdminIntegrationCenter() {
  const [providers, setProviders] = useState<IntegrationProvider[]>([]);
  const [logs, setLogs] = useState<IntegrationLog[]>([]);
  const [form, setForm] = useState(DEFAULT_FORM);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState('');
  const [secretName, setSecretName] = useState('password');
  const [secretValue, setSecretValue] = useState('');
  const [testRecipient, setTestRecipient] = useState('');
  const [testPhone, setTestPhone] = useState('');
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');

  const load = async () => {
    setError('');
    try {
      const [providerData, logData] = await Promise.all([
        apiRequest<{ items: IntegrationProvider[] }>('GET', '/admin/integrations/providers'),
        apiRequest<{ items: IntegrationLog[] }>('GET', '/admin/integrations/logs'),
      ]);
      setProviders(providerData.items || []);
      setLogs(logData.items || []);
      setSelectedId(current => current || providerData.items?.[0]?.id || '');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Integration center load failed');
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const selected = providers.find(provider => provider.id === selectedId);
  const counts = useMemo(() => ({
    enabled: providers.filter(provider => provider.status === 'enabled').length,
    healthy: providers.filter(provider => provider.health_status === 'healthy').length,
    liveBanks: providers.filter(provider => provider.provider_kind === 'bank_payout' && provider.environment === 'live').length,
  }), [providers]);

  const configFromForm = () => ({
    smtp_host: form.smtp_host || undefined,
    smtp_port: form.smtp_port ? Number(form.smtp_port) : undefined,
    smtp_user: form.smtp_user || undefined,
    smtp_security: form.smtp_security || 'starttls',
    from_name: form.from_name || undefined,
    from_email: form.from_email || undefined,
    reply_to: form.reply_to || undefined,
    sender_id: form.sender_id || undefined,
    route_type: form.route_type || undefined,
    callback_url: form.callback_url || undefined,
    client_id: form.client_id || undefined,
    settlement_account_mapping: form.settlement_account_mapping || undefined,
    reconciliation_source: form.reconciliation_source || undefined,
    utr_reference_field: form.utr_reference_field || undefined,
    template_mapping: form.template_map
      ? Object.fromEntries(form.template_map.split(',').map(item => item.trim()).filter(Boolean).map(item => {
        const [key, value] = item.split(':').map(part => part.trim());
        return [key, value];
      }))
      : {},
  });

  const saveProvider = async (event: FormEvent) => {
    event.preventDefault();
    setNotice('');
    setError('');
    try {
      const payload = {
        provider_kind: form.provider_kind,
        name: form.name,
        code: form.code,
        environment: form.environment,
        status: form.status,
        is_default: form.is_default,
        priority: form.priority,
        timeout_ms: form.timeout_ms,
        max_retries: form.max_retries,
        retry_backoff_ms: form.retry_backoff_ms,
        config: configFromForm(),
        live_enable_approved: false,
      };
      await apiRequest(editingId ? 'PUT' : 'POST', editingId ? `/admin/integrations/providers/${editingId}` : '/admin/integrations/providers', payload);
      setNotice(editingId ? 'Provider updated.' : 'Provider created.');
      setEditingId(null);
      setForm(DEFAULT_FORM);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed');
    }
  };

  const edit = (provider: IntegrationProvider) => {
    const config = provider.config || {};
    setEditingId(provider.id);
    setForm({
      ...DEFAULT_FORM,
      provider_kind: provider.provider_kind,
      name: provider.name,
      code: provider.code,
      environment: provider.environment,
      status: provider.status,
      is_default: provider.is_default,
      priority: provider.priority,
      timeout_ms: provider.timeout_ms,
      max_retries: provider.max_retries,
      retry_backoff_ms: provider.retry_backoff_ms,
      smtp_host: String(config.smtp_host || ''),
      smtp_port: String(config.smtp_port || '587'),
      smtp_user: String(config.smtp_user || ''),
      smtp_security: String(config.smtp_security || 'starttls'),
      from_name: String(config.from_name || ''),
      from_email: String(config.from_email || ''),
      reply_to: String(config.reply_to || ''),
      sender_id: String(config.sender_id || ''),
      route_type: String(config.route_type || ''),
      callback_url: String(config.callback_url || ''),
      client_id: String(config.client_id || ''),
      settlement_account_mapping: String(config.settlement_account_mapping || ''),
      reconciliation_source: String(config.reconciliation_source || ''),
      utr_reference_field: String(config.utr_reference_field || ''),
      template_map: Object.entries((config.template_mapping as Record<string, string>) || {}).map(([key, value]) => `${key}:${value}`).join(','),
    });
  };

  const run = async (labelText: string, action: () => Promise<unknown>) => {
    setNotice('');
    setError('');
    try {
      await action();
      setNotice(labelText);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    }
  };

  return (
    <section className="page-shell">
      <header className="page-header">
        <div>
          <h1>Integration Center</h1>
          <p>Internal provider setup for messaging, verification, payout, notifications, and future adapters</p>
        </div>
        <button className="btn-secondary" onClick={() => void load()}><RefreshCw size={16} /> Refresh</button>
      </header>

      {notice && <div className="alert alert-success">{notice}</div>}
      {error && <div className="alert alert-error">{error}</div>}

      <div className="stats-grid">
        <Metric title="Providers" value={providers.length} icon={ShieldCheck} />
        <Metric title="Enabled" value={counts.enabled} icon={Activity} />
        <Metric title="Healthy" value={counts.healthy} icon={Bell} />
        <Metric title="Live bank configs" value={counts.liveBanks} icon={Landmark} />
      </div>

      <div className="grid grid-2">
        <section className="card">
          <h2>{editingId ? 'Edit provider' : 'Add provider'}</h2>
          <form onSubmit={saveProvider}>
            <div className="filter-grid">
              <Field label="Provider kind"><Select value={form.provider_kind} options={KINDS} onChange={provider_kind => setForm({ ...form, provider_kind: provider_kind as ProviderKind })} /></Field>
              <Field label="Name"><input value={form.name} onChange={event => setForm({ ...form, name: event.target.value })} /></Field>
              <Field label="Code"><input value={form.code} onChange={event => setForm({ ...form, code: event.target.value })} /></Field>
              <Field label="Environment"><Select value={form.environment} options={['test', 'live']} onChange={environment => setForm({ ...form, environment: environment as 'test' | 'live' })} /></Field>
              <Field label="Status"><Select value={form.status} options={['disabled', 'enabled']} onChange={status => setForm({ ...form, status: status as 'enabled' | 'disabled' })} /></Field>
              <Field label="Priority"><input type="number" value={form.priority} onChange={event => setForm({ ...form, priority: Number(event.target.value) })} /></Field>
              <Field label="Timeout ms"><input type="number" value={form.timeout_ms} onChange={event => setForm({ ...form, timeout_ms: Number(event.target.value) })} /></Field>
              <Field label="Max retries"><input type="number" value={form.max_retries} onChange={event => setForm({ ...form, max_retries: Number(event.target.value) })} /></Field>
              <Field label="Retry backoff ms"><input type="number" value={form.retry_backoff_ms} onChange={event => setForm({ ...form, retry_backoff_ms: Number(event.target.value) })} /></Field>
              <Field label="SMTP host"><input value={form.smtp_host} onChange={event => setForm({ ...form, smtp_host: event.target.value })} /></Field>
              <Field label="SMTP port"><input value={form.smtp_port} onChange={event => setForm({ ...form, smtp_port: event.target.value })} /></Field>
              <Field label="SMTP user"><input value={form.smtp_user} onChange={event => setForm({ ...form, smtp_user: event.target.value })} /></Field>
              <Field label="SMTP security"><Select value={form.smtp_security} options={['starttls', 'tls', 'none']} onChange={smtp_security => setForm({ ...form, smtp_security })} /></Field>
              <Field label="From name"><input value={form.from_name} onChange={event => setForm({ ...form, from_name: event.target.value })} /></Field>
              <Field label="From email"><input value={form.from_email} onChange={event => setForm({ ...form, from_email: event.target.value })} /></Field>
              <Field label="Reply to"><input value={form.reply_to} onChange={event => setForm({ ...form, reply_to: event.target.value })} /></Field>
              <Field label="Sender ID"><input value={form.sender_id} onChange={event => setForm({ ...form, sender_id: event.target.value })} /></Field>
              <Field label="Route type"><input value={form.route_type} onChange={event => setForm({ ...form, route_type: event.target.value })} /></Field>
              <Field label="Callback URL"><input value={form.callback_url} onChange={event => setForm({ ...form, callback_url: event.target.value })} /></Field>
              <Field label="Client ID"><input value={form.client_id} onChange={event => setForm({ ...form, client_id: event.target.value })} /></Field>
              <Field label="Settlement mapping"><input value={form.settlement_account_mapping} onChange={event => setForm({ ...form, settlement_account_mapping: event.target.value })} /></Field>
              <Field label="Reconciliation source"><input value={form.reconciliation_source} onChange={event => setForm({ ...form, reconciliation_source: event.target.value })} placeholder="api or file" /></Field>
              <Field label="UTR reference field"><input value={form.utr_reference_field} onChange={event => setForm({ ...form, utr_reference_field: event.target.value })} /></Field>
              <Field label="Template map"><input value={form.template_map} onChange={event => setForm({ ...form, template_map: event.target.value })} placeholder="otp:OTP01,payout:PAY01" /></Field>
            </div>
            <label className="check-row">
              <input type="checkbox" checked={form.is_default} onChange={event => setForm({ ...form, is_default: event.target.checked })} />
              Default provider
            </label>
            <button className="btn-primary" type="submit">Save provider</button>
          </form>
        </section>

        <section className="card">
          <h2><KeyRound size={18} /> Secrets and tests</h2>
          <Field label="Selected provider">
            <Select value={selectedId} options={providers.map(provider => ({ label: `${provider.name} (${label(provider.provider_kind)})`, value: provider.id }))} onChange={setSelectedId} />
          </Field>
          <div className="filter-grid">
            <Field label="Secret name"><input value={secretName} onChange={event => setSecretName(event.target.value)} /></Field>
            <Field label="Secret value"><input type="password" value={secretValue} onChange={event => setSecretValue(event.target.value)} /></Field>
            <Field label="Test email"><input value={testRecipient} onChange={event => setTestRecipient(event.target.value)} /></Field>
            <Field label="Test phone"><input value={testPhone} onChange={event => setTestPhone(event.target.value)} /></Field>
          </div>
          <div className="button-row">
            <button onClick={() => selected && void run('Secret saved.', () => apiRequest('POST', `/admin/integrations/providers/${selected.id}/secrets`, { secret_name: secretName, value: secretValue }))}>Save secret</button>
            <button onClick={() => selected && void run('Secret rotated.', () => apiRequest('POST', `/admin/integrations/providers/${selected.id}/secrets/${secretName}/rotate`, { secret_name: secretName, value: secretValue }))}>Rotate secret</button>
            <button onClick={() => selected && void run('Connection tested.', () => apiRequest('POST', `/admin/integrations/providers/${selected.id}/test-connection`))}>Test connection</button>
            <button onClick={() => selected && void run('Test email sent.', () => apiRequest('POST', `/admin/integrations/providers/${selected.id}/test-email`, { recipient: testRecipient }))}><Mail size={14} /> Test email</button>
            <button onClick={() => selected && void run('Default email tested.', () => apiRequest('POST', `/admin/integrations/providers/default/email/${selected.environment}/test-email`, { recipient: testRecipient }))}>Test default email</button>
            <button disabled title="No safe generic SMS adapter schema exists yet"><Smartphone size={14} /> Test SMS</button>
            <button disabled title="No safe generic OTP adapter schema exists yet">Test OTP</button>
            <button onClick={() => selected && void run('Bank payout test completed.', () => apiRequest('POST', `/admin/integrations/providers/${selected.id}/test-bank-payout`))}><Landmark size={14} /> Test bank payout</button>
          </div>
          <p className="muted-copy">Generic SMS/OTP delivery remains disabled until a safe provider request schema is added.</p>
          <div className="table-wrap">
            <table>
              <thead><tr><th>Secret</th><th>Masked value</th><th>Rotated</th></tr></thead>
              <tbody>
                {(selected?.masked_secrets || []).map(secret => (
                  <tr key={secret.secret_name}>
                    <td>{secret.secret_name}</td>
                    <td>{secret.masked_value}</td>
                    <td>{secret.rotated_at}</td>
                  </tr>
                ))}
                {!selected?.masked_secrets?.length && <tr><td colSpan={3}>No secrets configured</td></tr>}
              </tbody>
            </table>
          </div>
        </section>
      </div>

      <section className="table-wrap">
        <h2>Notification Template Mapping</h2>
        <table>
          <thead><tr><th>Template key</th><th>Selected provider mapping</th></tr></thead>
          <tbody>
            {TEMPLATE_KEYS.map(key => (
              <tr key={key}>
                <td>{label(key)}</td>
                <td>{String(((selected?.config.template_mapping as Record<string, string> | undefined) || {})[key] || '-')}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Provider</th><th>Kind</th><th>Env</th><th>Status</th><th>Default</th>
              <th>Priority</th><th>Health</th><th>Last success</th><th>Last failure</th><th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {providers.map(provider => (
              <tr key={provider.id}>
                <td>{provider.name}</td>
                <td>{label(provider.provider_kind)}</td>
                <td>{provider.environment}</td>
                <td>{provider.status}</td>
                <td>{provider.is_default ? 'yes' : 'no'}</td>
                <td>{provider.priority}</td>
                <td>{provider.health_status}</td>
                <td>{provider.last_success_at || '-'}</td>
                <td>{provider.last_failure_at || '-'}</td>
                <td>
                  <button onClick={() => edit(provider)}>Edit</button>
                  <button onClick={() => void run(provider.status === 'enabled' ? 'Provider disabled.' : 'Provider enabled.', () => apiRequest('POST', `/admin/integrations/providers/${provider.id}/status`, { enabled: provider.status !== 'enabled', live_enable_approved: false }))}>{provider.status === 'enabled' ? 'Disable' : 'Enable'}</button>
                  <button onClick={() => void run('Default provider updated.', () => apiRequest('POST', `/admin/integrations/providers/${provider.id}/default`))}>Set default</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="table-wrap">
        <h2>Integration Logs</h2>
        <table>
          <thead><tr><th>Action</th><th>Type</th><th>Status</th><th>Latency</th><th>Error</th><th>Actor</th><th>Time</th></tr></thead>
          <tbody>
            {logs.map(log => (
              <tr key={log.id}>
                <td>{log.action}</td>
                <td>{log.request_type}</td>
                <td>{log.status}</td>
                <td>{log.latency_ms} ms</td>
                <td>{log.error_code || log.error_message || '-'}</td>
                <td>{log.actor_admin_id || '-'}</td>
                <td>{log.created_at}</td>
              </tr>
            ))}
            {!logs.length && <tr><td colSpan={7}>No integration logs yet</td></tr>}
          </tbody>
        </table>
      </section>
    </section>
  );
}

function Metric({ title, value, icon: Icon }: { title: string; value: number; icon: typeof ShieldCheck }) {
  return <article className="stat-card"><div className="stat-label"><Icon size={14} /> {title}</div><div className="stat-value">{value}</div></article>;
}

function Field({ label: fieldLabel, children }: { label: string; children: React.ReactNode }) {
  return <label className="form-field"><span>{fieldLabel}</span>{children}</label>;
}

function Select({ value, options, onChange }: { value: string; options: (string | { label: string; value: string })[]; onChange: (value: string) => void }) {
  return (
    <select value={value} onChange={event => onChange(event.target.value)}>
      {options.map(option => {
        const item = typeof option === 'string' ? { label: label(option), value: option } : option;
        return <option key={item.value} value={item.value}>{item.label}</option>;
      })}
    </select>
  );
}
