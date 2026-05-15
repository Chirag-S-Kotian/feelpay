import { useEffect, useMemo, useState } from 'react';
import { Eye, Mail, RefreshCw, Send, Smartphone } from 'lucide-react';
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

const DEFAULT_SAMPLE = '{"customer_name":"Asha","amount":"INR 499.00","otp":"000000","reference":"FP123"}';

export default function AdminNotificationTemplates() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [types, setTypes] = useState<string[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [dispatchLogs, setDispatchLogs] = useState<DispatchLog[]>([]);
  const [selectedId, setSelectedId] = useState('');
  const [sampleData, setSampleData] = useState(DEFAULT_SAMPLE);
  const [recipient, setRecipient] = useState('');
  const [phone, setPhone] = useState('');
  const [preview, setPreview] = useState<Record<string, unknown> | null>(null);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');
  const selected = templates.find(template => template.id === selectedId);
  const emailProviders = useMemo(() => providers.filter(p => p.provider_kind === 'email' && p.status === 'enabled'), [providers]);
  const smsProviders = useMemo(() => providers.filter(p => p.provider_kind === 'sms' && p.status === 'enabled'), [providers]);

  const load = async () => {
    const [templateData, providerData, dispatchData] = await Promise.all([
      apiRequest<{ items: Template[]; template_types: string[] }>('GET', '/admin/integrations/templates'),
      apiRequest<{ items: Provider[] }>('GET', '/admin/integrations/providers'),
      apiRequest<{ items: DispatchLog[] }>('GET', '/admin/integrations/dispatch-logs'),
    ]);
    setTemplates(templateData.items);
    setTypes(templateData.template_types);
    setProviders(providerData.items);
    setDispatchLogs(dispatchData.items);
    setSelectedId(current => current || templateData.items[0]?.id || '');
  };
  useEffect(() => { void load(); }, []);

  const parseSample = () => JSON.parse(sampleData) as Record<string, string>;
  const save = async (template: Template) => {
    setError('');
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
    setNotice('Template saved.');
    await load();
  };
  const create = async () => {
    await apiRequest('POST', '/admin/integrations/templates', {
      event_type: types[0],
      channel: 'email',
      environment: 'test',
      subject: 'Hello {{customer_name}}',
      body: 'Hello {{customer_name}}',
      sms_body: null,
      enabled: false,
      default_email_provider_id: emailProviders[0]?.id || null,
      default_sms_provider_id: null,
    });
    setNotice('Template created.');
    await load();
  };
  const previewTemplate = async () => setPreview(await apiRequest('POST', `/admin/integrations/templates/${selectedId}/preview`, { sample_data: parseSample() }));
  const testSend = async () => {
    await apiRequest('POST', `/admin/integrations/templates/${selectedId}/test`, { recipient, phone, sample_data: parseSample() });
    setNotice('Test send completed.');
  };
  const setEnabled = async (enabled: boolean) => {
    await apiRequest('POST', `/admin/integrations/templates/${selectedId}/status`, { enabled });
    setNotice(enabled ? 'Template enabled.' : 'Template disabled.');
    await load();
  };
  const retry = async (id: string) => {
    await apiRequest('POST', `/admin/integrations/dispatch-logs/${id}/retry`);
    setNotice('Retry scheduled.');
    await load();
  };
  if (!selected) return <div className="page"><button onClick={() => void create()}>Create first template</button></div>;
  const update = (patch: Partial<Template>) => setTemplates(items => items.map(item => item.id === selected.id ? { ...item, ...patch } : item));
  return (
    <div className="page fade-in">
      <div className="page-header"><div><h1>Notification Templates</h1><p>Transactional email and SMS delivery mappings</p></div><button onClick={() => void create()}>New template</button></div>
      {notice && <div className="alert alert-success">{notice}</div>}
      {error && <div className="alert alert-error">{error}</div>}
      <div className="grid-2">
        <div className="card">
          <select value={selectedId} onChange={event => setSelectedId(event.target.value)}>{templates.map(t => <option key={t.id} value={t.id}>{t.event_type} / {t.channel} / {t.environment}</option>)}</select>
          <select value={selected.event_type} onChange={event => update({ event_type: event.target.value })}>{types.map(type => <option key={type}>{type}</option>)}</select>
          <select value={selected.channel} onChange={event => update({ channel: event.target.value as Template['channel'] })}><option>email</option><option>sms</option><option>both</option></select>
          <select value={selected.environment} onChange={event => update({ environment: event.target.value as Template['environment'] })}><option>test</option><option>live</option></select>
          <input value={selected.subject || ''} placeholder="Email subject" onChange={event => update({ subject: event.target.value })} />
          <textarea value={selected.body} onChange={event => update({ body: event.target.value })} />
          <textarea value={selected.sms_body || ''} placeholder="SMS body" onChange={event => update({ sms_body: event.target.value })} />
          <select value={selected.default_email_provider_id || ''} onChange={event => update({ default_email_provider_id: event.target.value || null })}><option value="">Email provider</option>{emailProviders.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}</select>
          <select value={selected.default_sms_provider_id || ''} onChange={event => update({ default_sms_provider_id: event.target.value || null })}><option value="">SMS provider</option>{smsProviders.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}</select>
          <label><input type="checkbox" checked={selected.enabled} onChange={event => void setEnabled(event.target.checked)} /> Enabled</label>
          <p className="muted-copy">Allowed variables: {selected.allowed_variables.join(', ') || '-'}</p>
          <button onClick={() => void save(selected)}>Save</button>
        </div>
        <div className="card">
          <textarea value={sampleData} onChange={event => setSampleData(event.target.value)} />
          <input value={recipient} onChange={event => setRecipient(event.target.value)} placeholder="Test email recipient" />
          <input value={phone} onChange={event => setPhone(event.target.value)} placeholder="Test phone" />
          <button onClick={() => void previewTemplate()}><Eye size={14} /> Preview</button>
          <button onClick={() => void testSend()}><Send size={14} /> Test send</button>
          {preview && <pre className="json-preview">{JSON.stringify(preview, null, 2)}</pre>}
          <p className="muted-copy"><Mail size={14} /> Email and <Smartphone size={14} /> SMS tests use mapped providers.</p>
        </div>
      </div>
      <div className="card">
        <h2>Recent Delivery Attempts</h2>
        <div className="table-wrap">
          <table>
            <thead><tr><th>Event</th><th>Channel</th><th>Recipient</th><th>Status</th><th>Retries</th><th>Created</th><th /></tr></thead>
            <tbody>
              {dispatchLogs.map(log => (
                <tr key={log.id}>
                  <td>{log.event_type}</td>
                  <td>{log.channel}</td>
                  <td>{log.recipient_masked}</td>
                  <td>{log.status}{log.error_code ? ` / ${log.error_code}` : ''}</td>
                  <td>{log.retry_count}/{log.max_retries}</td>
                  <td>{new Date(log.created_at).toLocaleString()}</td>
                  <td>
                    {log.status === 'failed' && log.retryable && log.event_type !== 'otp' && (
                      <button onClick={() => void retry(log.id)}><RefreshCw size={14} /> Retry</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
