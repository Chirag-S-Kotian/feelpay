import { useState } from 'react';
import { RefreshCw } from 'lucide-react';

export default function Settings() {
  const [apiKey] = useState('fp_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6');
  const [copied, setCopied] = useState(false);

  const copy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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
            <input type="password" value="fp_secret_xxxxxxxxxxxxxxxxxxxx" readOnly />
          </div>
          <button className="btn btn-primary" style={{ marginTop: 8 }}>
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
            <input type="url" placeholder="https://your-domain.com/webhooks/feelpay" />
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
          <button className="btn btn-primary">Save Webhook</button>
        </div>

        {/* Business Info */}
        <div className="card" style={{ gridColumn: '1 / -1' }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Business Information</h3>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20 }}>
            <div className="input-group">
              <label>Business Name</label>
              <input type="text" placeholder="Your Business Pvt Ltd" />
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
              <input type="email" placeholder="finance@yourbusiness.com" />
            </div>
            <div className="input-group">
              <label>Website</label>
              <input type="url" placeholder="https://yourbusiness.com" />
            </div>
          </div>
          <button className="btn btn-primary" style={{ marginTop: 8 }}>Update Business Info</button>
        </div>
      </div>
    </div>
  );
}
