import { useEffect, useMemo, useState } from 'react';
import { Eye, Mail, RefreshCw, Send, Plus, Save, Sliders, FileText, CheckCircle, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type Template = {
  id: string;
  event_type: string;
  channel: 'email' | 'sms' | 'both';
  environment: 'test' | 'live';
  subject?: string | null;
  body: string;
  sms_body?: string | null;
  allowed_variables: string[];
  enabled: boolean;
  default_email_provider_id?: string | null;
  default_sms_provider_id?: string | null;
};
type Provider = { id: string; name: string; provider_kind: string; environment: string; status: string };
type DispatchLog = {
  id: string;
  event_type: string;
  channel: 'email' | 'sms';
  recipient_masked: string;
  status: 'sent' | 'failed' | 'skipped';
  error_code?: string | null;
  retry_count: number;
  max_retries: number;
  retryable: boolean;
  created_at: string;
};

const DEFAULT_SAMPLE = '{\n  "customer_name": "Asha",\n  "amount": "₹499.00",\n  "otp": "492019",\n  "reference": "FP-90214"\n}';

function formatDateTime(value?: string) {
  if (!value) return 'Active / Current';
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

export default function AdminNotificationTemplates() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [types, setTypes] = useState<string[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [dispatchLogs, setDispatchLogs] = useState<DispatchLog[]>([]);
  const [selectedId, setSelectedId] = useState('');
  const [sampleData, setSampleData] = useState(DEFAULT_SAMPLE);
  const [recipient, setRecipient] = useState('aditya.sharma@example.com');
  const [phone, setPhone] = useState('+919876543210');
  const [preview, setPreview] = useState<Record<string, unknown> | null>(null);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');
  const selected = templates.find(template => template.id === selectedId);
  const emailProviders = useMemo(() => providers.filter(p => p.provider_kind === 'email' && p.status === 'enabled'), [providers]);
  const smsProviders = useMemo(() => providers.filter(p => p.provider_kind === 'sms' && p.status === 'enabled'), [providers]);

  const load = async () => {
    try {
      const [templateData, providerData, dispatchData] = await Promise.all([
        apiRequest<{ items: Template[]; template_types: string[] }>('GET', '/admin/integrations/templates'),
        apiRequest<{ items: Provider[] }>('GET', '/admin/integrations/providers'),
        apiRequest<{ items: DispatchLog[] }>('GET', '/admin/integrations/dispatch-logs'),
      ]);
      setTemplates(templateData.items || []);
      setTypes(templateData.template_types || []);
      setProviders(providerData.items || []);
      setDispatchLogs(dispatchData.items || []);
      setSelectedId(current => current || templateData.items?.[0]?.id || '');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load templates');
    }
  };

  useEffect(() => { void load(); }, []);

  const parseSample = () => {
    try {
      return JSON.parse(sampleData) as Record<string, string>;
    } catch {
      return {};
    }
  };

  const save = async (template: Template) => {
    setError('');
    try {
      await apiRequest('PUT', `/admin/integrations/templates/${template.id}`, {
        event_type: template.event_type,
        channel: template.channel,
        environment: template.environment,
        subject: template.subject || null,
        body: template.body,
        sms_body: template.sms_body || null,
        enabled: template.enabled,
        default_email_provider_id: template.default_email_provider_id || null,
        default_sms_provider_id: template.default_sms_provider_id || null,
      });
      setNotice('Template configuration saved successfully.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed');
    }
  };

  const create = async () => {
    try {
      await apiRequest('POST', '/admin/integrations/templates', {
        event_type: types[0] || 'payment.success',
        channel: 'email',
        environment: 'test',
        subject: 'FeelPay Update: {{customer_name}}',
        body: 'Hello {{customer_name}},\n\nYour transaction has been processed.',
        sms_body: null,
        enabled: false,
        default_email_provider_id: emailProviders[0]?.id || null,
        default_sms_provider_id: null,
      });
      setNotice('New notification template created.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Creation failed');
    }
  };

  const previewTemplate = async () => {
    try {
      const res = await apiRequest<Record<string, unknown>>('POST', `/admin/integrations/templates/${selectedId}/preview`, { sample_data: parseSample() });
      setPreview(res);
      setNotice('Preview rendered.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Preview failed');
    }
  };

  const testSend = async () => {
    try {
      await apiRequest('POST', `/admin/integrations/templates/${selectedId}/test`, { recipient, phone, sample_data: parseSample() });
      setNotice('Test notification dispatched.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Test send failed');
    }
  };

  const setEnabled = async (enabled: boolean) => {
    try {
      await apiRequest('POST', `/admin/integrations/templates/${selectedId}/status`, { enabled });
      setNotice(enabled ? 'Template activated.' : 'Template deactivated.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Status update failed');
    }
  };

  const retry = async (id: string) => {
    try {
      await apiRequest('POST', `/admin/integrations/dispatch-logs/${id}/retry`);
      setNotice('Notification retry queued.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Retry failed');
    }
  };

  if (!selected) {
    return (
      <div className="page fade-in">
        <div className="card" style={{ textAlign: 'center', padding: '48px' }}>
          <h3>No Notification Templates Configured</h3>
          <p style={{ color: 'var(--fp-text-muted)', marginBottom: '20px' }}>Configure automated transactional messages for customer transactions and alerts.</p>
          <button type="button" className="btn-primary" onClick={() => void create()}>
            <Plus size={16} /> Create first template
          </button>
        </div>
      </div>
    );
  }

  const update = (patch: Partial<Template>) => setTemplates(items => items.map(item => item.id === selected.id ? { ...item, ...patch } : item));

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Notification Templates</h1>
          <p>Transactional email and SMS messaging mappings</p>
        </div>
        <button type="button" className="btn-primary" onClick={() => void create()}>
          <Plus size={16} /> New template
        </button>
      </div>

      {notice && (
        <div className="alert alert-success" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <CheckCircle size={16} /> {notice}
        </div>
      )}
      {error && (
        <div className="alert alert-error" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <AlertCircle size={16} /> {error}
        </div>
      )}

      <div className="grid grid-2" style={{ alignItems: 'start' }}>
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '18px' }}>
            <FileText size={18} style={{ color: 'var(--spotify-tertiary)' }} />
            <h2 style={{ margin: 0 }}>Template Configuration</h2>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <label className="form-field">
              <span>Active Template</span>
              <select value={selectedId} onChange={event => setSelectedId(event.target.value)}>
                {templates.map(t => (
                  <option key={t.id} value={t.id}>
                    {t.event_type} • {t.channel.toUpperCase()} • [{t.environment.toUpperCase()}]
                  </option>
                ))}
              </select>
            </label>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
              <label className="form-field">
                <span>Event Type</span>
                <select value={selected.event_type} onChange={event => update({ event_type: event.target.value })}>
                  {types.map(type => <option key={type} value={type}>{type}</option>)}
                </select>
              </label>

              <label className="form-field">
                <span>Channel</span>
                <select value={selected.channel} onChange={event => update({ channel: event.target.value as Template['channel'] })}>
                  <option value="email">Email</option>
                  <option value="sms">SMS</option>
                  <option value="both">Both (Email & SMS)</option>
                </select>
              </label>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
              <label className="form-field">
                <span>Environment</span>
                <select value={selected.environment} onChange={event => update({ environment: event.target.value as Template['environment'] })}>
                  <option value="test">Test / Sandbox</option>
                  <option value="live">Live / Production</option>
                </select>
              </label>

              <label className="form-field">
                <span>Delivery Status</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', height: '42px' }}>
                  <input
                    type="checkbox"
                    id="template-enabled"
                    checked={selected.enabled}
                    onChange={event => void setEnabled(event.target.checked)}
                    style={{ width: '18px', height: '18px', cursor: 'pointer' }}
                  />
                  <label htmlFor="template-enabled" style={{ cursor: 'pointer', fontWeight: 500, fontSize: '13px' }}>
                    {selected.enabled ? 'Active (Ready to send)' : 'Disabled'}
                  </label>
                </div>
              </label>
            </div>

            {(selected.channel === 'email' || selected.channel === 'both') && (
              <>
                <label className="form-field">
                  <span>Email Subject Line</span>
                  <input
                    value={selected.subject || ''}
                    placeholder="e.g. Your payment receipt from FeelPay"
                    onChange={event => update({ subject: event.target.value })}
                  />
                </label>

                <label className="form-field">
                  <span>Email Body (HTML / Markdown)</span>
                  <textarea
                    rows={5}
                    value={selected.body}
                    placeholder="Write email template content with {{variable}} placeholders..."
                    onChange={event => update({ body: event.target.value })}
                    style={{ fontFamily: 'monospace', fontSize: '13px' }}
                  />
                </label>

                <label className="form-field">
                  <span>Default Email Gateway</span>
                  <select
                    value={selected.default_email_provider_id || ''}
                    onChange={event => update({ default_email_provider_id: event.target.value || null })}
                  >
                    <option value="">Auto (System Default Provider)</option>
                    {emailProviders.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
                  </select>
                </label>
              </>
            )}

            {(selected.channel === 'sms' || selected.channel === 'both') && (
              <>
                <label className="form-field">
                  <span>SMS Message Text</span>
                  <textarea
                    rows={3}
                    value={selected.sms_body || ''}
                    placeholder="SMS text message with {{variable}} tokens..."
                    onChange={event => update({ sms_body: event.target.value })}
                    style={{ fontFamily: 'monospace', fontSize: '13px' }}
                  />
                </label>

                <label className="form-field">
                  <span>Default SMS Gateway</span>
                  <select
                    value={selected.default_sms_provider_id || ''}
                    onChange={event => update({ default_sms_provider_id: event.target.value || null })}
                  >
                    <option value="">Auto (System Default Provider)</option>
                    {smsProviders.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
                  </select>
                </label>
              </>
            )}

            <div>
              <span style={{ fontSize: '12px', color: 'var(--fp-text-muted)', display: 'block', marginBottom: '6px' }}>
                Allowed Variable Tokens
              </span>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
                {selected.allowed_variables.length > 0 ? (
                  selected.allowed_variables.map(v => (
                    <span key={v} className="var-pill">
                      {`{{${v}}}`}
                    </span>
                  ))
                ) : (
                  <span style={{ fontSize: '12px', color: 'var(--fp-text-muted)' }}>None defined</span>
                )}
              </div>
            </div>

            <div style={{ marginTop: '8px', paddingTop: '16px', borderTop: '1px solid var(--fp-border)' }}>
              <button type="button" className="btn-primary" onClick={() => void save(selected)}>
                <Save size={14} /> Save Template Changes
              </button>
            </div>
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '18px' }}>
            <Sliders size={18} style={{ color: 'var(--spotify-tertiary)' }} />
            <h2 style={{ margin: 0 }}>Test Dispatch & Live Preview</h2>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <label className="form-field">
              <span>Sample Test Payload (JSON)</span>
              <textarea
                rows={5}
                value={sampleData}
                onChange={event => setSampleData(event.target.value)}
                style={{ fontFamily: 'monospace', fontSize: '12px' }}
              />
            </label>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
              <label className="form-field">
                <span>Test Email Address</span>
                <input
                  type="email"
                  value={recipient}
                  onChange={event => setRecipient(event.target.value)}
                  placeholder="admin@feelpay.dev"
                />
              </label>

              <label className="form-field">
                <span>Test Phone (+91...)</span>
                <input
                  type="tel"
                  value={phone}
                  onChange={event => setPhone(event.target.value)}
                  placeholder="+919876543210"
                />
              </label>
            </div>

            <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
              <button type="button" className="btn-secondary" onClick={() => void previewTemplate()}>
                <Eye size={14} /> Render Preview
              </button>
              <button type="button" className="btn-primary" onClick={() => void testSend()}>
                <Send size={14} /> Dispatch Test Notification
              </button>
            </div>

            {preview && (
              <div style={{ marginTop: '12px' }}>
                <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--fp-text-muted)', display: 'block', marginBottom: '6px' }}>
                  Rendered Output
                </span>
                <pre className="json-preview" style={{ maxHeight: '240px', overflow: 'auto' }}>
                  {JSON.stringify(preview, null, 2)}
                </pre>
              </div>
            )}

            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px', color: 'var(--fp-text-muted)', marginTop: '8px' }}>
              <Mail size={14} />
              <span>Email & SMS dispatches are routed through active configured providers.</span>
            </div>
          </div>
        </div>
      </div>

      <div className="table-card" style={{ marginTop: '24px' }}>
        <div className="page-header" style={{ marginBottom: '16px' }}>
          <div>
            <h3 style={{ margin: 0 }}>Recent Delivery Attempts</h3>
            <span style={{ fontSize: '13px', color: 'var(--fp-text-muted)' }}>Real-time execution log of transactional messaging events</span>
          </div>
        </div>
        <div className="table-scroll">
          <table>
            <thead>
              <tr>
                <th>Event Type</th>
                <th>Channel</th>
                <th>Recipient</th>
                <th>Status</th>
                <th>Retries</th>
                <th>Dispatched At</th>
                <th className="actions-cell">Actions</th>
              </tr>
            </thead>
            <tbody>
              {dispatchLogs.map(log => (
                <tr key={log.id}>
                  <td style={{ fontWeight: 600, color: 'var(--fp-text)' }}>{log.event_type}</td>
                  <td>
                    <span className="badge" style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)' }}>
                      {log.channel.toUpperCase()}
                    </span>
                  </td>
                  <td className="mono">{log.recipient_masked}</td>
                  <td>
                    <span className={`badge ${log.status === 'sent' ? 'badge-completed' : log.status === 'failed' ? 'badge-failed' : 'badge-pending'}`}>
                      {log.status}
                      {log.error_code ? ` (${log.error_code})` : ''}
                    </span>
                  </td>
                  <td>{log.retry_count} / {log.max_retries}</td>
                  <td>{formatDateTime(log.created_at)}</td>
                  <td className="actions-cell">
                    {log.status === 'failed' && log.retryable && log.event_type !== 'otp' ? (
                      <button type="button" className="btn-action-secondary" onClick={() => void retry(log.id)}>
                        <RefreshCw size={12} /> Retry
                      </button>
                    ) : (
                      <span style={{ color: 'var(--fp-text-muted)', fontSize: '12px' }}>Auto-Processed</span>
                    )}
                  </td>
                </tr>
              ))}
              {dispatchLogs.length === 0 && (
                <tr>
                  <td colSpan={7} style={{ textAlign: 'center', padding: '32px', color: 'var(--fp-text-muted)' }}>
                    No delivery records logged yet
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
