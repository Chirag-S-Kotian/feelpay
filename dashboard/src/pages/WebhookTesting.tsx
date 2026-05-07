import { useState, useEffect } from 'react';
import { Send, Eye, AlertCircle, CheckCircle, Clock, Webhook, Save } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface WebhookEndpoint {
  id: string;
  url: string;
  status: 'active' | 'inactive';
  event_subscriptions: string[];
}

interface WebhookEvent {
  id: string;
  event_type: string;
  status: string;
  http_status?: number;
  attempts: number;
  signature_verified?: boolean;
  created_at: string;
  last_attempt_at?: string;
  response_body?: string;
  error_message?: string;
}

interface TestResult {
  test_id: string;
  status: 'sent' | 'failed';
  http_status?: number;
  response_time_ms?: number;
  response_body?: string;
  error_message?: string;
}

interface SamplePayload {
  event_type: string;
  payload: Record<string, unknown>;
}

const EVENT_TYPES = [
  'payment.success',
  'payment.failed',
  'refund.initiated',
  'settlement.processed',
  'dispute.created',
];

export default function WebhookTesting() {
  const [endpoints, setEndpoints] = useState<WebhookEndpoint[]>([]);
  const [selectedEndpoint, setSelectedEndpoint] = useState<WebhookEndpoint | null>(null);
  const [events, setEvents] = useState<WebhookEvent[]>([]);
  const [webhookUrl, setWebhookUrl] = useState('');
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');
  const [selectedEventType, setSelectedEventType] = useState('payment.success');
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [showPayload, setShowPayload] = useState(false);
  const [samplePayload, setSamplePayload] = useState<SamplePayload | null>(null);

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  useEffect(() => {
    fetchWebhookEndpoints();
  }, []);

  const fetchWebhookEndpoints = async () => {
    setError('');
    try {
      const [profile, webhookData] = await Promise.all([
        apiRequest<{ webhook_url?: string | null }>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 5_000 }),
        merchantId
          ? apiRequest<{ logs?: WebhookEvent[]; events?: WebhookEvent[]; webhooks?: WebhookEndpoint[] }>('GET', `/v1/merchants/${merchantId}/webhooks/test-logs`, undefined, undefined, { cacheMs: 5_000 })
          : Promise.resolve({ logs: [], events: [], webhooks: [] }),
      ]);
      const savedUrl = profile.webhook_url || '';
      setWebhookUrl(savedUrl);
      const configuredEndpoint: WebhookEndpoint[] = savedUrl ? [{
        id: 'merchant-webhook-url',
        url: savedUrl,
        status: 'active',
        event_subscriptions: EVENT_TYPES,
      }] : [];
      const webhooks = webhookData.webhooks?.length ? webhookData.webhooks : configuredEndpoint;
      setEndpoints(webhooks);
      setSelectedEndpoint(webhooks[0] ?? null);
      setEvents(webhookData.logs || webhookData.events || []);
    } catch (err) {
      console.error('Failed to fetch webhook endpoints:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch webhook configuration.');
    } finally {
      setLoading(false);
    }
  };

  const saveWebhookUrl = async () => {
    setNotice('');
    setError('');
    const trimmed = webhookUrl.trim();
    const isLocalhost = /^https?:\/\/(localhost|127\.0\.0\.1|\[::1\])(?::\d+)?/i.test(trimmed);
    if (!trimmed.startsWith('https://') && !isLocalhost) {
      setError('Webhook URL must be HTTPS. Localhost is allowed only in test mode.');
      return;
    }
    try {
      await apiRequest('PUT', '/v1/merchants/profile', { webhook_url: trimmed });
      setNotice('Webhook URL saved.');
      await fetchWebhookEndpoints();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to save webhook URL.');
    }
  };

  const handleTestWebhook = async () => {
    if (!selectedEndpoint) return;

    setTesting(true);
    setTestResult(null);

    try {
      const data = await apiRequest<TestResult>(
        'POST',
        `/v1/merchants/${merchantId}/webhooks/test`,
        {
          event_type: selectedEventType,
          webhook_endpoint_id: selectedEndpoint.id,
          webhook_url: selectedEndpoint.url,
        }
      );
      setTestResult(data);
      await fetchWebhookEndpoints();
      setNotice('Test event sent. Recent delivery event updated.');
    } catch (err) {
      setTestResult({
        test_id: 'error',
        status: 'failed',
        error_message: err instanceof Error ? err.message : 'Test failed',
      });
    } finally {
      setTesting(false);
    }
  };

  const handleViewPayload = async () => {
    try {
      setSamplePayload({
        event_type: selectedEventType,
        payload: {
          id: `evt_${Date.now()}`,
          created_at: new Date().toISOString(),
          merchant_id: merchantId,
          event_type: selectedEventType,
        },
      });
      setShowPayload(true);
    } catch (err) {
      console.error('Failed to fetch sample payload:', err);
    }
  };

  const handleReplayWebhook = async (eventId: string) => {
    if (!merchantId) return;

    setTesting(true);
    setTestResult(null);

    try {
      const data = await apiRequest<TestResult>(
        'POST',
        `/v1/merchants/${merchantId}/webhooks/replay/${eventId}`
      );
      setTestResult(data);
      await fetchWebhookEndpoints();
      setNotice('Event replayed. Delivery status updated.');
    } catch (err) {
      setTestResult({
        test_id: 'error',
        status: 'failed',
        error_message: err instanceof Error ? err.message : 'Replay failed',
      });
    } finally {
      setTesting(false);
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="card mc-loader-card"><span className="fp-loader"></span> Loading webhook configuration...</div>
      </div>
    );
  }

  return (
    <div className="page-container webhooks-page mc-page">
      <div className="page-header">
        <div>
          <h1>Webhook Testing</h1>
          <p>Configure HTTPS endpoint, inspect signed payload, send test event.</p>
        </div>
        <div className="webhook-mark"><Webhook size={28} /></div>
      </div>

      {notice && <div className="card methods-alert success"><CheckCircle size={16} /> {notice}</div>}
      {error && <div className="card methods-alert danger"><AlertCircle size={16} /> {error}</div>}

      <div className="grid grid-2">
        <div className="card webhook-config-card">
          <h2>Endpoint</h2>

          <div className="form-group">
            <label>Webhook URL</label>
            <div className="webhook-url-row">
              <input
                value={webhookUrl}
                onChange={(e) => setWebhookUrl(e.target.value)}
                placeholder="https://merchant.example/webhooks/feelpay"
              />
              <button className="btn-secondary" onClick={() => void saveWebhookUrl()}>
                <Save size={16} /> Save
              </button>
            </div>
            <p className="mini-help">HTTPS only. Test runner signs payload with merchant webhook secret.</p>
          </div>

          <div className="form-group">
            <label>Webhook Endpoint</label>
            <select
              aria-label="Webhook Endpoint"
              value={selectedEndpoint?.id || ''}
              onChange={e => {
                const endpoint = endpoints.find(ep => ep.id === e.target.value);
                setSelectedEndpoint(endpoint || null);
              }}
            >
              {endpoints.length === 0 && <option value="">No saved endpoint</option>}
              {endpoints.map(ep => (
                <option key={ep.id} value={ep.id}>
                  {ep.url}
                </option>
              ))}
            </select>
            {selectedEndpoint && (
              <div className="endpoint-info">
                <div className="info-row">
                  <span>Status:</span>
                  <span className={`badge badge-${selectedEndpoint.status}`}>
                    {selectedEndpoint.status}
                  </span>
                </div>
                <div className="info-row">
                  <span>Subscribed Events:</span>
                  <span>{selectedEndpoint.event_subscriptions.length}</span>
                </div>
              </div>
            )}
          </div>

          <div className="form-group">
            <label>Event Type</label>
            <select
              aria-label="Event Type"
              value={selectedEventType}
              onChange={e => setSelectedEventType(e.target.value)}
            >
              {EVENT_TYPES.map(et => (
                <option key={et} value={et}>
                  {et}
                </option>
              ))}
            </select>
          </div>

          <div className="form-actions">
            <button
              className="btn-secondary"
              onClick={handleViewPayload}
            >
              <Eye size={16} />
              View Sample Payload
            </button>
            <button
              className="btn-primary"
              onClick={handleTestWebhook}
              disabled={testing}
            >
              {testing ? (
                <>
                  <Clock size={16} className="spin" />
                  Sending...
                </>
              ) : (
                <>
                  <Send size={16} />
                  Send Test Event
                </>
              )}
            </button>
          </div>
        </div>

        <div className="card webhook-events-card">
        <h2>Recent Delivery Events</h2>
          {testResult ? (
            <div className={`result-container result-${testResult.status}`}>
              {testResult.status === 'sent' ? (
                <CheckCircle size={32} />
              ) : (
                <AlertCircle size={32} />
              )}

              <div className="result-status">
                {testResult.status === 'sent' ? 'Success' : 'Failed'}
              </div>

              {testResult.http_status && (
                <div className="result-row">
                  <span>HTTP Status:</span>
                  <strong>{testResult.http_status}</strong>
                </div>
              )}

              {testResult.response_time_ms && (
                <div className="result-row">
                  <span>Response Time:</span>
                  <strong>{testResult.response_time_ms}ms</strong>
                </div>
              )}

              {testResult.response_body && (
                <div className="result-row">
                  <span>Response Body:</span>
                  <pre className="response-body">{testResult.response_body}</pre>
                </div>
              )}

              {testResult.error_message && (
                <div className="result-row">
                  <span>Error:</span>
                  <span className="error-text">{testResult.error_message}</span>
                </div>
              )}

              <div className="result-row">
                <span>Test ID:</span>
                <span className="mono">{testResult.test_id}</span>
              </div>
            </div>
          ) : null}
            <div className="webhook-event-list">
              {events.length === 0 ? (
                <div className="empty-state webhook-empty">
                  <Webhook size={34} />
                  <strong>No delivery events yet</strong>
                  <p>Save endpoint and send test event. Events will appear here after delivery attempts.</p>
                </div>
              ) : events.map((event) => (
                <div className="webhook-event-item" key={event.id}>
                  <strong>{event.event_type}</strong>
                  <span>{event.status}</span>
                  <small>
                    HTTP {event.http_status ?? '-'} • {event.attempts} attempt(s) • signature {event.signature_verified === false ? 'not verified' : 'verified'}
                  </small>
                  <small>Created {new Date(event.created_at).toLocaleString()} • Last attempt {new Date(event.last_attempt_at || event.created_at).toLocaleString()}</small>
                  <div className="event-actions">
                    <button className="btn-secondary" onClick={() => {
                      setSamplePayload({
                        event_type: event.event_type,
                        payload: {
                          id: event.id,
                          event_type: event.event_type,
                          status: event.status,
                          http_status: event.http_status ?? null,
                          response_body: event.response_body ?? null,
                          error_message: event.error_message ?? null,
                        },
                      });
                      setShowPayload(true);
                    }}>View Payload</button>
                    <button className="btn-secondary" onClick={() => void handleReplayWebhook(event.id)}>Replay</button>
                  </div>
                </div>
              ))}
            </div>
        </div>
      </div>

      {showPayload && samplePayload && (
        <div className="modal-overlay" onClick={() => setShowPayload(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Sample Payload: {samplePayload.event_type}</h2>
              <button className="btn-close" onClick={() => setShowPayload(false)}>
                ✕
              </button>
            </div>
            <div className="modal-body">
              <pre className="payload-json">{JSON.stringify(samplePayload.payload, null, 2)}</pre>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .webhooks-page .page-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          border: 1px solid #d5c9af;
          border-radius: 34px;
          background: #fff9ec;
          padding: 24px;
        }
        .webhook-mark {
          width: 64px;
          height: 64px;
          border-radius: 999px;
          display: grid;
          place-items: center;
          background: #eb001b;
          color: #fff9ec;
        }
        .grid-2 {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
        }
        @media (max-width: 1024px) {
          .grid-2 {
            grid-template-columns: 1fr;
          }
        }
        .webhook-config-card,
        .webhook-events-card {
          border-radius: 34px !important;
          background: #fff9ec !important;
        }
        .webhook-url-row {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 10px;
        }
        .endpoint-info {
          margin-top: 12px;
          padding: 14px;
          background: #f5efe2;
          border: 1px solid #d5c9af;
          border-radius: 22px;
          font-size: 13px;
        }
        .info-row {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
        }
        .result-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
          padding: 20px;
          border-radius: 28px;
          text-align: center;
        }
        .result-container.result-sent {
          background: #eef8ef;
          color: #166534;
        }
        .result-container.result-failed {
          background: #fff0ef;
          color: #991b1b;
        }
        .result-status {
          font-size: 18px;
          font-weight: 600;
        }
        .result-row {
          width: 100%;
          text-align: left;
          padding: 8px 0;
          border-bottom: 1px solid rgba(0, 0, 0, 0.1);
          display: flex;
          justify-content: space-between;
          font-size: 13px;
        }
        .result-row:last-child {
          border-bottom: none;
        }
        .response-body {
          background: #f5efe2;
          padding: 8px;
          border-radius: 18px;
          font-size: 11px;
          overflow-x: auto;
          max-height: 150px;
          margin: 0;
        }
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(20, 20, 19, 0.34);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .modal-content {
          background: #fff9ec;
          border-radius: 28px;
          box-shadow: 0 18px 50px rgba(20, 20, 19, 0.18);
          max-width: 600px;
          width: 90%;
          max-height: 80vh;
          overflow-y: auto;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px;
          border-bottom: 1px solid #d5c9af;
        }
        .modal-header h2 {
          margin: 0;
          font-size: 16px;
        }
        .btn-close {
          background: none;
          border: none;
          cursor: pointer;
          font-size: 20px;
          color: #6b6862;
        }
        .modal-body {
          padding: 20px;
        }
        .payload-json {
          background: #f5efe2;
          padding: 12px;
          border-radius: 20px;
          font-size: 12px;
          overflow-x: auto;
          margin: 0;
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        .webhook-event-list { display: grid; gap: 10px; }
        .webhook-event-item {
          border: 1px solid #d5c9af;
          border-radius: 24px;
          background: #f5efe2;
          padding: 14px;
          display: grid;
          gap: 4px;
        }
        .webhook-event-item strong { color: #312d28; }
        .webhook-event-item span { color: #eb001b; text-transform: uppercase; font-size: 12px; font-weight: 800; letter-spacing: 0.06em; }
        .webhook-event-item small { color: #6b6862; }
        .event-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 6px; }
        .webhook-empty {
          min-height: 260px;
          display: grid;
          place-items: center;
          align-content: center;
          gap: 10px;
          color: #6b6862;
        }
        .webhook-empty strong { color: #312d28; font-size: 20px; }
        .methods-alert { margin: 14px 0; display: flex; align-items: center; gap: 8px; }
        .methods-alert.success { border-color: rgba(35, 110, 62, 0.34) !important; color: #236e3e !important; }
        .methods-alert.danger { border-color: rgba(200, 31, 47, 0.34) !important; color: #a01525 !important; }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
