import { useEffect, useMemo, useState } from 'react';
import {
  BookOpen,
  Check,
  Copy,
  KeyRound,
  RefreshCw,
  Rocket,
  ShieldCheck,
  TerminalSquare,
  Webhook,
} from 'lucide-react';
import { apiRequest, getEnvironmentMode, setEnvironmentMode, type EnvironmentMode } from '../hooks/useApi';

type SnippetId = 'curl' | 'node' | 'python' | 'webhook';
type WebhookEvent = {
  id: string;
  event_type: string;
  status: string;
  attempts: number;
  created_at: string;
};

const API_BASE_BY_MODE: Record<EnvironmentMode, string> = {
  test: import.meta.env.VITE_SANDBOX_API_BASE_URL || import.meta.env.VITE_API_BASE_URL || '/api',
  live: import.meta.env.VITE_LIVE_API_BASE_URL || import.meta.env.VITE_API_BASE_URL || '/api',
};

const USE_CASES = [
  {
    title: 'One-time payments',
    desc: 'Create payment, track status, capture/refund with idempotent retries.',
    endpoints: ['POST /v1/payments', 'GET /v1/payments/{id}', 'POST /v1/payments/{id}/refund'],
  },
  {
    title: 'Subscriptions and mandates',
    desc: 'Recurring billing with pause/resume/cancel lifecycle APIs.',
    endpoints: ['POST /v1/subscriptions', 'POST /v1/mandates', 'POST /v1/subscriptions/{id}/pause'],
  },
  {
    title: 'Settlement and reconciliation',
    desc: 'Fetch settlement list, balance, exports, and automate reporting.',
    endpoints: ['GET /v1/settlements', 'GET /v1/settlements/balance', 'GET /v1/merchants/{id}/exports'],
  },
];

const ERROR_GUIDE = [
  { code: '400', meaning: 'Invalid payload, state, or validation failed', action: 'Fix request fields, retry with corrected body' },
  { code: '401', meaning: 'Missing/invalid API key or auth context', action: 'Verify `x-api-key`, auth token, and env mode' },
  { code: '403', meaning: 'Live mode or resource access is blocked', action: 'Complete onboarding or check merchant ownership' },
  { code: '404', meaning: 'Resource/route not found', action: 'Check path params, merchant scope, and endpoint version' },
  { code: '409', meaning: 'Idempotency key reused with different payload', action: 'Reuse key only for exact same body' },
  { code: '429', meaning: 'Rate limit exceeded', action: 'Back off with jitter and retry using the same idempotency key' },
  { code: '500', meaning: 'Server/database failure', action: 'Retry with same idempotency key; alert support if persistent' },
];

export default function Developers() {
  const [apiKey, setApiKey] = useState('Loading...');
  const [createdSecret, setCreatedSecret] = useState<string | null>(null);
  const [preflight, setPreflight] = useState<{
    can_go_live: boolean;
    blockers: string[];
    checks: Record<string, boolean>;
  } | null>(null);
  const [mode, setMode] = useState<EnvironmentMode>(getEnvironmentMode());
  const [copied, setCopied] = useState<string | null>(null);
  const [selectedSnippet, setSelectedSnippet] = useState<SnippetId>('curl');
  const [webhookEvents, setWebhookEvents] = useState<WebhookEvent[]>([]);
  const [apiLogs, setApiLogs] = useState<Array<{
    request_id: string;
    endpoint: string;
    status: string;
    actor_id?: string | null;
    ip_address?: string | null;
    created_at: string;
  }>>([]);
  const [apiLogStatus, setApiLogStatus] = useState('all');
  const [apiLogEndpoint, setApiLogEndpoint] = useState('');
  const [selectedLog, setSelectedLog] = useState<{
    request_id: string;
    endpoint: string;
    status: string;
    actor_id?: string | null;
    ip_address?: string | null;
    created_at: string;
  } | null>(null);

  const baseUrl = API_BASE_BY_MODE[mode];
  const sampleIdempotencyKey = 'pay_01HV7M5YVQX2S2AZ8WMK6QF7SZ';

  const loadKeys = async () => {
    const data = await apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys');
    setApiKey(data.api_key);
  };

  const loadReadiness = async () => {
    try {
      const data = await apiRequest<{
        can_go_live: boolean;
        blockers: string[];
        checks: Record<string, boolean>;
      }>('GET', '/v1/onboarding/live-preflight', undefined, undefined, { cacheMs: 15_000 });
      setPreflight(data);
    } catch (err) {
      console.error('Preflight check failed:', err);
    }
  };

  const loadWebhookEvents = async () => {
    try {
      const data = await apiRequest<{ events?: WebhookEvent[] }>(
        'GET',
        '/v1/developers/webhooks',
        undefined,
        undefined,
        { cacheMs: 20_000 },
      );
      setWebhookEvents(data.events || []);
    } catch {
      setWebhookEvents([]);
    }
  };

  const loadApiLogs = async () => {
    const qs = new URLSearchParams();
    if (apiLogStatus !== 'all') qs.set('status', apiLogStatus);
    if (apiLogEndpoint.trim()) qs.set('endpoint', apiLogEndpoint.trim());
    const data = await apiRequest<{ logs?: Array<{
      request_id: string;
      endpoint: string;
      status: string;
      actor_id?: string | null;
      ip_address?: string | null;
      created_at: string;
    }> }>(
      'GET',
      `/v1/developers/api-logs${qs.toString() ? `?${qs.toString()}` : ''}`,
      undefined,
      undefined,
      { cacheMs: 12_000 },
    );
    setApiLogs(data.logs || []);
  };

  useEffect(() => {
    setEnvironmentMode(mode);
    void loadKeys();
    void loadWebhookEvents();
    void loadApiLogs();
    void loadReadiness();
  }, [mode]);

  useEffect(() => {
    void loadApiLogs();
  }, [apiLogStatus, apiLogEndpoint]);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 1600);
  };

  const regenerateKeys = async () => {
    const data = await apiRequest<{ api_key: string; api_secret?: string }>('POST', '/v1/merchants/keys/roll', {});
    setCreatedSecret(data.api_secret || null);
    await loadKeys();
  };


  const snippets = useMemo<Record<SnippetId, string>>(
    () => ({
      curl: `curl -X POST ${baseUrl}/v1/payments \\
  -H "x-api-key: ${apiKey}" \\
  -H "x-feelpay-environment: ${mode}" \\
  -H "idempotency-key: ${sampleIdempotencyKey}" \\
  -H "Content-Type: application/json" \\
  -d '{
    "amount": 50000,
    "currency": "INR",
    "order_id": "ORD-2026-0001",
    "customer_email": "customer@example.com",
    "customer_phone": "+919999999999",
    "description": "Test card only - not real payment data",
    "return_url": "https://merchant.example/return",
    "method_data": {
      "method": "card",
      "data": {
        "card_number": "4111111111111111",
        "expiry_month": 12,
        "expiry_year": 2028,
        "cvv": "123"
      }
    }
  }'`,
      node: `const payload = {
  amount: 50000,
  currency: 'INR',
  order_id: 'ORD-2026-0001',
  customer_email: 'customer@example.com',
  customer_phone: '+919999999999',
  description: 'Test card only - not real payment data',
  return_url: 'https://merchant.example/return',
  method_data: {
    method: 'card',
    data: {
      card_number: '4111111111111111',
      expiry_month: 12,
      expiry_year: 2028,
      cvv: '123',
    },
  },
};

const res = await fetch('${baseUrl}/v1/payments', {
  method: 'POST',
  headers: {
    'x-api-key': '${apiKey}',
    'x-feelpay-environment': '${mode}',
    'idempotency-key': '${sampleIdempotencyKey}',
    'content-type': 'application/json',
  },
  body: JSON.stringify(payload),
});

console.log(res.status, await res.json());`,
      python: `import requests

url = "${baseUrl}/v1/payments"
headers = {
    "x-api-key": "${apiKey}",
    "x-feelpay-environment": "${mode}",
    "idempotency-key": "${sampleIdempotencyKey}",
    "content-type": "application/json",
}
payload = {
    "amount": 50000,
    "currency": "INR",
    "order_id": "ORD-2026-0001",
    "customer_email": "customer@example.com",
    "customer_phone": "+919999999999",
    "description": "Test card only - not real payment data",
    "return_url": "https://merchant.example/return",
    "method_data": {
        "method": "card",
        "data": {
            "card_number": "4111111111111111",
            "expiry_month": 12,
            "expiry_year": 2028,
            "cvv": "123"
        }
    }
}

r = requests.post(url, headers=headers, json=payload, timeout=20)
print(r.status_code, r.json())`,
      webhook: `import crypto from 'crypto';

function verifyFeelPayWebhook(rawBody, signature, webhookSecret) {
  const expected = crypto
    .createHmac('sha256', webhookSecret)
    .update(rawBody)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(expected, 'hex'),
    Buffer.from(signature, 'hex'),
  );
}

// Verify header: X-FeelPay-Signature
// Use raw request body. Do not parse before signature check.`,
    }),
    [apiKey, baseUrl, mode],
  );

  return (
    <div className="page">
      <div className="dev-hero">
        <div>
          <div className="dev-kicker">Developer Platform</div>
          <h1>Build, test, ship payments fast.</h1>
          <p>Integration hub for FeelPay keys, examples, idempotency, reliability, and go-live controls.</p>
        </div>
        <div className="mode-toggle">
          <button className={mode === 'test' ? 'active' : ''} onClick={() => setMode('test')}>Test Mode</button>
          <button 
            className={mode === 'live' ? 'active' : ''} 
            onClick={() => {
              if (preflight?.can_go_live) {
                setMode('live');
              } else {
                alert(`Cannot switch to Live Mode. Blockers:\n• ${preflight?.blockers.join('\n• ') || 'Onboarding incomplete'}`);
              }
            }}
          >
            Live Mode
          </button>
        </div>
      </div>

      <section className="api-section">
        <div className="dev-section-title"><KeyRound size={18} /> Credentials & Base URL</div>
        <div className="dev-grid">
          <div className="dev-card">
            <label>Base URL</label>
            <div className="key-row">
              <code>{baseUrl}</code>
              <button onClick={() => copyToClipboard(baseUrl, 'base')}>
                {copied === 'base' ? <Check size={16} /> : <Copy size={16} />}
              </button>
            </div>
          </div>
          <div className="dev-card">
            <label>API Key</label>
            <div className="key-row">
              <code>{apiKey}</code>
              <button onClick={() => copyToClipboard(apiKey, 'key')}>
                {copied === 'key' ? <Check size={16} /> : <Copy size={16} />}
              </button>
            </div>
          </div>
        </div>
        <div className="dev-actions">
          <button className="btn-secondary" onClick={regenerateKeys}>
            <RefreshCw size={16} /> Rotate API Key
          </button>
          <p className="help-text">Secret keys not re-displayed. Rotate immediately if leaked.</p>
          {createdSecret && (
            <div className="one-time-secret" role="status">
              <p className="help-text">New secret is available once in this session. Copy it now, then it will be masked.</p>
              <div className="key-row">
                <code>{createdSecret}</code>
                <button onClick={() => copyToClipboard(createdSecret, 'secret')}>
                  {copied === 'secret' ? <Check size={16} /> : <Copy size={16} />}
                </button>
              </div>
            </div>
          )}
          {mode === 'live' && <p className="help-text">Live keys unlock after onboarding approval.</p>}
        </div>
      </section>

      <section className="docs-section">
        <div className="dev-section-title"><TerminalSquare size={18} /> Quickstart</div>
        <ul className="dev-bullets">
          <li>Auth via <code>x-api-key</code>.</li>
          <li>POST retries must include stable <code>idempotency-key</code>.</li>
          <li>Amount in paise (<code>50000 = INR 500.00</code>).</li>
          <li>Set <code>x-feelpay-environment</code> = <code>test</code> or <code>live</code>.</li>
        </ul>
        <div className="lang-tabs">
          {[
            ['curl', 'cURL'],
            ['node', 'Node.js'],
            ['python', 'Python'],
            ['webhook', 'Webhook Verify'],
          ].map(([id, label]) => (
            <button key={id} className={selectedSnippet === id ? 'active' : ''} onClick={() => setSelectedSnippet(id as SnippetId)}>
              {label}
            </button>
          ))}
        </div>
        <pre className="code-block">
          <code>{snippets[selectedSnippet]}</code>
        </pre>
      </section>

      <section className="card">
        <div className="dev-section-title"><BookOpen size={18} /> Use Cases</div>
        <div className="usecase-grid">
          {USE_CASES.map((item) => (
            <div key={item.title} className="usecase-card">
              <h3>{item.title}</h3>
              <p>{item.desc}</p>
              <div className="endpoint-list">
                {item.endpoints.map((ep) => (
                  <code key={ep}>{ep}</code>
                ))}
              </div>
            </div>
          ))}
        </div>
      </section>

      <section className="webhooks-section">
        <div className="dev-section-title"><Webhook size={18} /> Webhooks & Security</div>
        <div className="webhook-meta">
          <div className="meta-item"><ShieldCheck size={16} /> Verify <code>X-FeelPay-Signature</code> with HMAC-SHA256</div>
          <div className="meta-item"><ShieldCheck size={16} /> Store dedupe key/event id; ignore duplicate deliveries</div>
          <div className="meta-item"><ShieldCheck size={16} /> Do not rely on event ordering; make handlers idempotent</div>
        </div>
        <div className="webhook-events">
          <div className="event">payment.success</div>
          <div className="event">payment.failed</div>
          <div className="event">refund.initiated</div>
          <div className="event">settlement.processed</div>
          <div className="event">dispute.created</div>
        </div>
      </section>

      <section className="card golive-hub-section">
        <div className="dev-section-title"><Rocket size={18} /> Go-Live Readiness Hub</div>
        <div className="hub-layout">
          <div className="checklist-box">
            <div className={`check-item ${preflight?.checks?.onboarding_approved ? 'done' : ''}`}>
              <span className="icon">{preflight?.checks?.onboarding_approved ? '✅' : '○'}</span>
              <div className="text">
                <strong>Business & KYC Verification</strong>
                <p>Status: {preflight?.checks?.onboarding_approved ? 'Approved' : 'Pending Review'}</p>
              </div>
            </div>
            <div className={`check-item ${preflight?.checks?.verified_bank_account ? 'done' : ''}`}>
              <span className="icon">{preflight?.checks?.verified_bank_account ? '✅' : '○'}</span>
              <div className="text">
                <strong>Settlement Bank Account</strong>
                <p>{preflight?.checks?.verified_bank_account ? 'Verified primary account' : 'Verification required'}</p>
              </div>
            </div>
            <div className={`check-item ${preflight?.checks?.webhook_configured ? 'done' : ''}`}>
              <span className="icon">{preflight?.checks?.webhook_configured ? '✅' : '○'}</span>
              <div className="text">
                <strong>Webhook Configuration</strong>
                <p>{preflight?.checks?.webhook_configured ? 'HTTPS endpoint saved' : 'Production URL missing'}</p>
              </div>
            </div>
            <div className={`check-item ${preflight?.checks?.team_present ? 'done' : ''}`}>
              <span className="icon">{preflight?.checks?.team_present ? '✅' : '○'}</span>
              <div className="text">
                <strong>Team & Security</strong>
                <p>{preflight?.checks?.team_present ? 'Team members invited' : 'No team members found'}</p>
              </div>
            </div>
          </div>

          <div className="readiness-summary">
            <div className={`summary-card ${preflight?.can_go_live ? 'ready' : 'not-ready'}`}>
              <h3>{preflight?.can_go_live ? 'You are ready!' : 'Readiness: Incomplete'}</h3>
              <p>
                {preflight?.can_go_live 
                  ? 'All production requirements are met. You can now use Live Mode keys.' 
                  : `You have ${preflight?.blockers.length || 0} blocker(s) remaining before you can go live.`}
              </p>
              
              {!preflight?.can_go_live && preflight?.blockers.length ? (
                <ul className="blocker-list">
                  {preflight.blockers.map((b, i) => <li key={i}>• {b}</li>)}
                </ul>
              ) : null}

              {preflight?.can_go_live ? (
                <button className="btn-primary" onClick={() => setMode('live')}>
                  Switch to Live Mode
                </button>
              ) : (
                <button className="btn-secondary" disabled style={{ opacity: 0.5, cursor: 'not-allowed' }}>
                  Go Live (Locked)
                </button>
              )}
            </div>
          </div>
        </div>
      </section>

      <section className="card">
        <div className="dev-section-title"><Webhook size={18} /> Recent Webhook Deliveries</div>
        <div className="error-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th>Event</th>
                <th>Status</th>
                <th>Attempts</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {webhookEvents.slice(0, 10).map((item) => (
                <tr key={item.id}>
                  <td><code>{item.event_type}</code></td>
                  <td>{item.status}</td>
                  <td>{item.attempts}</td>
                  <td>{new Date(item.created_at).toLocaleString('en-IN')}</td>
                </tr>
              ))}
              {webhookEvents.length === 0 && (
                <tr>
                  <td colSpan={4}>No webhook logs yet.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>

      <section className="card">
        <div className="dev-section-title">API Activity Logs</div>
        <div className="dev-actions" style={{ marginBottom: 10 }}>
          <select value={apiLogStatus} onChange={(e) => setApiLogStatus(e.target.value)}>
            <option value="all">All status</option>
            <option value="success">Success</option>
            <option value="failed">Failed</option>
          </select>
          <input
            value={apiLogEndpoint}
            onChange={(e) => setApiLogEndpoint(e.target.value)}
            placeholder="Filter by endpoint/action"
            style={{ minWidth: 260 }}
          />
        </div>
        <div className="error-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th>Request ID</th>
                <th>Endpoint</th>
                <th>Status</th>
                <th>Time</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {apiLogs.slice(0, 20).map((item) => (
                <tr key={item.request_id}>
                  <td><code>{item.request_id.slice(0, 12)}...</code></td>
                  <td>{item.endpoint}</td>
                  <td>{item.status}</td>
                  <td>{new Date(item.created_at).toLocaleString('en-IN')}</td>
                  <td>
                    <button className="btn-secondary" onClick={() => setSelectedLog(item)}>View</button>
                  </td>
                </tr>
              ))}
              {apiLogs.length === 0 && (
                <tr>
                  <td colSpan={5}>No API logs found.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>

      <section className="card">
        <div className="dev-section-title">Error Handling Matrix</div>
        <div className="error-table-wrap">
          <table className="data-table">
            <thead>
              <tr>
                <th>Code</th>
                <th>Meaning</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {ERROR_GUIDE.map((item) => (
                <tr key={item.code}>
                  <td><code>{item.code}</code></td>
                  <td>{item.meaning}</td>
                  <td>{item.action}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {selectedLog && (
        <div className="modal-overlay" onClick={() => setSelectedLog(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3 style={{ marginTop: 0 }}>API Log Detail</h3>
            <p><strong>Request ID:</strong> <code>{selectedLog.request_id}</code></p>
            <p><strong>Endpoint:</strong> {selectedLog.endpoint}</p>
            <p><strong>Status:</strong> {selectedLog.status}</p>
            <p><strong>Actor:</strong> {selectedLog.actor_id || '-'}</p>
            <p><strong>IP:</strong> {selectedLog.ip_address || '-'}</p>
            <p><strong>Created:</strong> {new Date(selectedLog.created_at).toLocaleString('en-IN')}</p>
            <div className="modal-actions">
              <button className="btn-secondary" onClick={() => copyToClipboard(selectedLog.request_id, 'request-id')}>Copy Request ID</button>
              <button className="btn-primary" onClick={() => setSelectedLog(null)}>Close</button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .dev-hero {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 20px;
          border: 1px solid rgba(108, 170, 255, 0.3);
          border-radius: 16px;
          padding: 20px;
          margin-bottom: 18px;
          background:
            radial-gradient(140% 140% at 10% 0%, rgba(89, 152, 246, 0.2), transparent 42%),
            radial-gradient(120% 130% at 100% 0%, rgba(245, 158, 11, 0.14), transparent 46%),
            linear-gradient(180deg, rgba(16, 38, 72, 0.98), rgba(10, 24, 46, 0.98));
        }
        .dev-kicker {
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.15em;
          color: #9ec5ff;
          margin-bottom: 6px;
          font-weight: 700;
        }
        .dev-hero h1 {
          margin: 0;
          font-size: 32px;
          letter-spacing: -0.02em;
          line-height: 1.03;
        }
        .dev-hero p {
          margin-top: 8px;
          color: #a9c4e7;
          max-width: 680px;
        }
        .dev-section-title {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          font-weight: 700;
          margin-bottom: 12px;
        }
        .dev-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 12px;
        }
        .dev-card {
          border: 1px solid rgba(97, 160, 252, 0.25);
          border-radius: 12px;
          padding: 12px;
          background: rgba(17, 39, 72, 0.7);
        }
        .dev-card label {
          display: block;
          margin-bottom: 8px;
          color: #b8cff0;
          font-size: 12px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }
        .dev-actions {
          margin-top: 12px;
          display: flex;
          gap: 12px;
          align-items: center;
          flex-wrap: wrap;
        }
        .one-time-secret {
          display: grid;
          gap: 8px;
          padding: 12px;
          border: 1px solid rgba(245, 158, 11, 0.35);
          border-radius: 10px;
          background: rgba(245, 158, 11, 0.08);
        }
        .dev-bullets {
          margin: 8px 0 14px 18px;
          color: #c4d9f5;
          display: grid;
          gap: 6px;
        }
        .usecase-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
          gap: 12px;
        }
        .usecase-card {
          border: 1px solid rgba(99, 161, 252, 0.26);
          background: rgba(16, 38, 70, 0.65);
          border-radius: 12px;
          padding: 14px;
        }
        .usecase-card h3 {
          margin: 0 0 6px;
          font-size: 18px;
        }
        .usecase-card p {
          margin: 0 0 10px;
          color: #acc7e9;
          font-size: 14px;
        }
        .endpoint-list {
          display: flex;
          flex-direction: column;
          gap: 7px;
        }
        .endpoint-list code {
          background: rgba(7, 19, 38, 0.75);
          border: 1px solid rgba(93, 155, 248, 0.28);
          border-radius: 8px;
          padding: 6px 8px;
          color: #d0e4ff;
          font-size: 12px;
        }
        .webhook-meta {
          display: grid;
          gap: 8px;
          margin-bottom: 12px;
        }
        .meta-item {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: #b9d3f4;
          font-size: 14px;
        }
        .checklist-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
          gap: 10px;
        }
        .check-item {
          border: 1px solid rgba(102, 166, 255, 0.25);
          background: rgba(13, 34, 62, 0.72);
          border-radius: 10px;
          padding: 10px 12px;
          color: #cde2ff;
          font-size: 14px;
        }
        .error-table-wrap {
          overflow-x: auto;
        }
        .modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(4, 10, 22, 0.75);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2000;
          padding: 16px;
        }
        .modal-content {
          width: min(560px, 100%);
          border: 1px solid rgba(98, 163, 253, 0.3);
          border-radius: 14px;
          background: rgba(11, 28, 53, 0.96);
          padding: 18px;
        }
        .modal-actions {
          display: flex;
          gap: 10px;
          justify-content: flex-end;
          margin-top: 12px;
        }
        @media (max-width: 860px) {
          .dev-hero {
            flex-direction: column;
          }
        }
      `}</style>
    </div>
  );
}
