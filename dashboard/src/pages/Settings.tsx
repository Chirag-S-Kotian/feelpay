import { useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  Check,
  Copy,
  KeyRound,
  RefreshCw,
  ShieldCheck,
  Webhook,
  Zap,
} from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface MerchantProfile {
  id: string;
  name: string;
  email: string;
  business_name?: string | null;
  business_type?: string | null;
  webhook_url?: string | null;
  daily_limit: number;
  monthly_limit: number;
  onboarding_status: string;
  status: string;
}

interface SecuritySettings {
  mfa_enabled: boolean;
  settlement_cycle: string;
  per_transaction_limit: number;
  daily_limit: number;
  monthly_limit: number;
  allowed_payment_methods: Record<string, unknown>;
  webhook_secret_hint: string;
  updated_at: string;
}

const EVENTS = [
  'payment.success',
  'payment.failed',
  'refund.initiated',
  'settlement.processed',
  'dispute.created',
];

export default function Settings() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [name, setName] = useState('');
  const [businessName, setBusinessName] = useState('');
  const [businessType, setBusinessType] = useState('individual');
  const [webhookUrl, setWebhookUrl] = useState('');
  const [profile, setProfile] = useState<MerchantProfile | null>(null);
  const [security, setSecurity] = useState<SecuritySettings | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [notice, setNotice] = useState<string>('');
  const [warning, setWarning] = useState<string>('');

  const merchantId = useMemo(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  }, []);

  const loadAll = async () => {
    setLoading(true);
    try {
      const [profileRes, keysRes] = await Promise.all([
        apiRequest<MerchantProfile>('GET', '/v1/merchants/profile'),
        apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys'),
      ]);
      let securityRes: SecuritySettings;
      try {
        securityRes = await apiRequest<SecuritySettings>('GET', '/v1/merchants/settings/security');
        setWarning('');
      } catch (err) {
        const msg = err instanceof Error ? err.message : '';
        // Backward compatibility: backend route not deployed yet.
        if (msg.includes('(404)')) {
          securityRes = {
            mfa_enabled: false,
            settlement_cycle: 'T+2',
            per_transaction_limit: 0,
            daily_limit: profileRes.daily_limit || 0,
            monthly_limit: profileRes.monthly_limit || 0,
            allowed_payment_methods: {},
            webhook_secret_hint: 'unsupported_on_current_backend',
            updated_at: new Date().toISOString(),
          };
          setWarning('Security endpoint unavailable on running backend. Restart/update API server to enable full controls.');
        } else {
          throw err;
        }
      }
      setProfile(profileRes);
      setSecurity(securityRes);
      setApiKey(keysRes.api_key || '');
      setName(profileRes.name || '');
      setBusinessName(profileRes.business_name || '');
      setBusinessType((profileRes.business_type || 'individual').toLowerCase());
      setWebhookUrl(profileRes.webhook_url || '');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadAll();
  }, []);

  const copy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 1600);
  };

  const rotateApiKeys = async () => {
    setNotice('');
    await apiRequest('POST', '/v1/merchants/keys/roll', {});
    const keys = await apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys');
    setApiKey(keys.api_key);
    setNotice('API keys rotated. Update server env immediately.');
  };

  const rotateWebhookSecret = async () => {
    setNotice('');
    setWarning('');
    try {
      await apiRequest('POST', '/v1/merchants/webhook-secret/roll', {});
      await loadAll();
      setNotice('Webhook secret rotated. Update signature verification key.');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '';
      if (msg.includes('(404)')) {
        setWarning('Webhook secret rotation route unavailable on running backend. Deploy/restart latest API build.');
        return;
      }
      setWarning(msg || 'Failed to rotate webhook secret.');
    }
  };

  const saveProfile = async () => {
    setSaving(true);
    setNotice('');
    try {
      await apiRequest('PUT', '/v1/merchants/profile', {
        name: name.trim() || null,
        business_name: businessName.trim() || null,
        business_type: businessType,
        webhook_url: webhookUrl.trim() || null,
      });
      await loadAll();
      setNotice('Settings saved.');
    } finally {
      setSaving(false);
    }
  };

  const sendTestWebhook = async () => {
    if (!merchantId) {
      setNotice('Merchant session missing. Re-login required.');
      return;
    }
    const normalizedWebhookUrl = webhookUrl.trim();
    if (!normalizedWebhookUrl) {
      setWarning('Set and save a webhook URL before sending a test event.');
      return;
    }
    if (!normalizedWebhookUrl.startsWith('https://')) {
      setWarning('Webhook URL must start with https:// for test delivery.');
      return;
    }
    setNotice('');
    try {
      await apiRequest('POST', `/v1/merchants/${merchantId}/webhooks/test`, {
        event_type: 'payment.success',
        webhook_url: normalizedWebhookUrl,
      });
      setNotice('Test webhook sent for payment.success.');
      setWarning('');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '';
      if (msg.toLowerCase().includes('not implemented') || msg.includes('(501)')) {
        setWarning('Test webhook route exists but server still running old/stub implementation. Restart API with latest code.');
        return;
      }
      setWarning(msg || 'Failed to send test webhook.');
    }
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Gateway Settings</h1>
          <p>Production controls for identity, API auth, webhooks, and operational safety.</p>
        </div>
      </div>

      {notice && (
        <div className="card" style={{ marginBottom: 16, borderColor: 'rgba(88, 174, 116, 0.42)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: '#97efb2' }}>
            <Check size={16} />
            <span>{notice}</span>
          </div>
        </div>
      )}
      {warning && (
        <div className="card" style={{ marginBottom: 16, borderColor: 'rgba(245, 158, 11, 0.42)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: '#f8c786' }}>
            <AlertTriangle size={16} />
            <span>{warning}</span>
          </div>
        </div>
      )}

      <div className="settings-grid">
        <section className="card">
          <h3 className="section-title"><ShieldCheck size={17} /> Business Identity</h3>
          <div className="input-group">
            <label>Owner / Account Name</label>
            <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Merchant account name" />
          </div>
          <div className="input-group">
            <label>Business Name</label>
            <input value={businessName} onChange={(e) => setBusinessName(e.target.value)} placeholder="Legal business name" />
          </div>
          <div className="input-group">
            <label>Business Type</label>
            <select value={businessType} onChange={(e) => setBusinessType(e.target.value)}>
              <option value="individual">Individual</option>
              <option value="sole_proprietorship">Sole Proprietorship</option>
              <option value="partnership">Partnership</option>
              <option value="corporation">Corporation</option>
              <option value="llc">LLC</option>
            </select>
          </div>
          <div className="input-group">
            <label>Email</label>
            <input value={profile?.email || ''} readOnly />
          </div>
          <button className="btn-primary" onClick={() => void saveProfile()} disabled={saving}>
            {saving ? 'Saving...' : 'Save Business Settings'}
          </button>
        </section>

        <section className="card">
          <h3 className="section-title"><KeyRound size={17} /> API Authentication</h3>
          <div className="input-group">
            <label>API Key</label>
            <div className="key-row">
              <input value={apiKey} readOnly style={{ fontFamily: 'Fira Code, monospace', fontSize: 12 }} />
              <button className="btn-secondary" onClick={() => copy(apiKey, 'api')}>
                {copied === 'api' ? <Check size={14} /> : <Copy size={14} />}
              </button>
            </div>
          </div>
          <div className="security-list">
            <div className="security-row">
              <span>MFA status</span>
              <strong>{security?.mfa_enabled ? 'Enabled' : 'Disabled'}</strong>
            </div>
            <div className="security-row">
              <span>Merchant status</span>
              <strong>{profile?.status || 'unknown'}</strong>
            </div>
            <div className="security-row">
              <span>Onboarding</span>
              <strong>{profile?.onboarding_status || 'unknown'}</strong>
            </div>
          </div>
          <button className="btn-secondary" onClick={() => void rotateApiKeys()}>
            <RefreshCw size={16} /> Rotate API Keys
          </button>
          <p className="mini-help">Rotation invalidates prior secret. Roll one deployment window, not random time.</p>
        </section>
      </div>

      <div className="settings-grid" style={{ marginTop: 16 }}>
        <section className="card">
          <h3 className="section-title"><Webhook size={17} /> Webhook Security</h3>
          <div className="input-group">
            <label>Webhook URL</label>
            <input
              type="url"
              value={webhookUrl}
              onChange={(e) => setWebhookUrl(e.target.value)}
              placeholder="https://merchant.example/webhooks/feelpay"
            />
          </div>
          <div className="input-group">
            <label>Webhook Secret Hint</label>
            <input value={security?.webhook_secret_hint || 'not_set'} readOnly />
          </div>
          <div className="webhook-events">
            {EVENTS.map((evt) => (
              <div key={evt} className="event-chip">{evt}</div>
            ))}
          </div>
          <div className="settings-actions">
            <button className="btn-primary" onClick={() => void saveProfile()} disabled={saving}>Save Webhook URL</button>
            <button className="btn-secondary" onClick={() => void rotateWebhookSecret()}>Rotate Webhook Secret</button>
            <button className="btn-secondary" onClick={() => void sendTestWebhook()}>Send Test Webhook</button>
          </div>
          <p className="mini-help">Enforce `https`. Verify `X-FeelPay-Signature` against raw body with HMAC-SHA256.</p>
        </section>

        <section className="card">
          <h3 className="section-title"><Zap size={17} /> Operational Limits</h3>
          <div className="ops-grid">
            <div className="ops-tile">
              <span>Per transaction</span>
              <strong>INR {security?.per_transaction_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Daily limit</span>
              <strong>INR {security?.daily_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Monthly limit</span>
              <strong>INR {security?.monthly_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Settlement cycle</span>
              <strong>{security?.settlement_cycle || 'T+2'}</strong>
            </div>
          </div>
          <div className="method-box">
            <label>Allowed Payment Methods JSON</label>
            <pre>{JSON.stringify(security?.allowed_payment_methods || {}, null, 2)}</pre>
          </div>
          <div className="risk-note">
            <AlertTriangle size={14} />
            <span>Limit updates require compliance approval. Use onboarding/ops workflow for changes.</span>
          </div>
        </section>
      </div>

      <style>{`
        .settings-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
          gap: 16px;
        }
        .section-title {
          font-size: 16px;
          margin-bottom: 14px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
        }
        .key-row {
          display: flex;
          gap: 8px;
        }
        .key-row .btn-secondary {
          min-width: 42px;
          display: inline-flex;
          justify-content: center;
          align-items: center;
        }
        .security-list {
          border: 1px solid rgba(103, 166, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          margin: 12px 0;
          background: rgba(14, 34, 63, 0.56);
        }
        .security-row {
          display: flex;
          justify-content: space-between;
          padding: 7px 0;
          font-size: 13px;
          border-bottom: 1px solid rgba(100, 158, 245, 0.14);
        }
        .security-row:last-child {
          border-bottom: none;
        }
        .webhook-events {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin: 10px 0 12px;
        }
        .event-chip {
          font-size: 12px;
          border: 1px solid rgba(99, 164, 253, 0.28);
          background: rgba(14, 33, 60, 0.72);
          border-radius: 999px;
          padding: 5px 9px;
          color: #c8ddfa;
        }
        .settings-actions {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }
        .ops-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 10px;
          margin-bottom: 12px;
        }
        .ops-tile {
          border: 1px solid rgba(98, 163, 253, 0.26);
          border-radius: 10px;
          padding: 10px;
          background: rgba(13, 34, 62, 0.6);
          display: flex;
          flex-direction: column;
          gap: 4px;
          font-size: 12px;
          color: #aac7ea;
        }
        .ops-tile strong {
          font-size: 14px;
          color: #eef5ff;
        }
        .method-box label {
          font-size: 12px;
          color: #b6cff0;
          display: block;
          margin-bottom: 6px;
        }
        .method-box pre {
          margin: 0;
          background: rgba(9, 22, 42, 0.84);
          border: 1px solid rgba(102, 164, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          max-height: 170px;
          overflow: auto;
          color: #d2e6ff;
          font-size: 12px;
        }
        .risk-note {
          margin-top: 10px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: #f7c98c;
          font-size: 12px;
          border: 1px solid rgba(245, 158, 11, 0.28);
          background: rgba(245, 158, 11, 0.1);
          border-radius: 8px;
          padding: 8px 10px;
        }
        .mini-help {
          margin-top: 8px;
          font-size: 12px;
          color: #9ebbe0;
        }
      `}</style>
    </div>
  );
}
