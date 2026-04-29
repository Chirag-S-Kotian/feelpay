import { useEffect, useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

export default function Settings() {
  const [apiKey, setApiKey] = useState('');
  const [name, setName] = useState('');
  const [businessName, setBusinessName] = useState('');
  const [webhookUrl, setWebhookUrl] = useState('');
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const load = async () => {
      const [profile, keys] = await Promise.all([
        apiRequest<{ name: string; business_name?: string; webhook_url?: string }>('GET', '/v1/merchants/profile'),
        apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys'),
      ]);
      setName(profile.name || '');
      setBusinessName(profile.business_name || '');
      setWebhookUrl(profile.webhook_url || '');
      setApiKey(keys.api_key || '');
    };
    void load();
  }, []);

  const copy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const rotateKeys = async () => {
    await apiRequest('POST', '/v1/merchants/keys/roll', {});
    const keys = await apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys');
    setApiKey(keys.api_key);
  };

  const saveProfile = async () => {
    await apiRequest('PUT', '/v1/merchants/profile', {
      name,
      business_name: businessName,
      webhook_url: webhookUrl || null,
    });
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Settings</h1>
          <p>Manage your API keys, webhooks, and account settings</p>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 24 }}>
        {/* API Keys */}
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>API Keys</h3>
          <div className="input-group">
            <label>API Key (Public)</label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input type="text" value={apiKey} readOnly style={{ fontFamily: 'monospace', fontSize: 12 }} />
              <button className="btn btn-secondary" onClick={() => copy(apiKey)}
                style={{ whiteSpace: 'nowrap', fontSize: 12 }}>
                {copied ? '✓ Copied' : 'Copy'}
              </button>
            </div>
          </div>
          <div className="input-group">
            <label>API Secret (Hidden)</label>
            <input type="password" value="Only shown at creation/rotation" readOnly />
          </div>
          <button className="btn btn-primary" style={{ marginTop: 8 }} onClick={() => void rotateKeys()}>
            <RefreshCw size={16} /> Rotate Keys
          </button>
          <p style={{ fontSize: 12, color: 'var(--fp-text-dim)', marginTop: 12 }}>
            Rotating keys will invalidate the current secret. Make sure to update your integration.
          </p>
        </div>

        {/* Webhook */}
        <div className="card">
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Webhook Configuration</h3>
          <div className="input-group">
            <label>Webhook URL</label>
            <input type="url" placeholder="https://your-domain.com/webhooks/feelpay" value={webhookUrl} onChange={(e) => setWebhookUrl(e.target.value)} />
          </div>
          <div className="input-group">
            <label>Webhook Secret</label>
            <input type="password" value="whsec_xxxxxxxxxxxxxxxxxxxx" readOnly />
          </div>
          <div style={{ marginBottom: 20 }}>
            <label style={{ fontSize: 13, fontWeight: 600, color: 'var(--fp-text-muted)', marginBottom: 10, display: 'block' }}>Events</label>
            {['payment.captured', 'payment.failed', 'refund.processed', 'settlement.completed'].map(evt => (
              <label key={evt} style={{
                display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8,
                fontSize: 13, cursor: 'pointer', color: 'var(--fp-text)',
              }}>
                <input type="checkbox" defaultChecked style={{ accentColor: 'var(--fp-primary)' }} />
                <code style={{ fontSize: 12, padding: '2px 8px', background: 'var(--fp-bg-elevated)', borderRadius: 4 }}>{evt}</code>
              </label>
            ))}
          </div>
          <button className="btn btn-primary" onClick={() => void saveProfile()}>Save Webhook</button>
        </div>

        {/* Business Info */}
        <div className="card" style={{ gridColumn: '1 / -1' }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Business Information</h3>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20 }}>
            <div className="input-group">
              <label>Business Name</label>
              <input type="text" value={businessName} onChange={(e) => setBusinessName(e.target.value)} placeholder="Your Business Pvt Ltd" />
            </div>
            <div className="input-group">
              <label>Business Type</label>
              <select>
                <option>Private Limited</option>
                <option>Partnership</option>
                <option>Proprietorship</option>
                <option>LLP</option>
              </select>
            </div>
            <div className="input-group">
              <label>Email</label>
              <input type="email" placeholder="finance@yourbusiness.com" readOnly />
            </div>
            <div className="input-group">
              <label>Website</label>
              <input type="url" placeholder="https://yourbusiness.com" readOnly />
            </div>
          </div>
          <button className="btn btn-primary" style={{ marginTop: 8 }} onClick={() => void saveProfile()}>Update Business Info</button>
        </div>
      </div>
    </div>
  );
}
