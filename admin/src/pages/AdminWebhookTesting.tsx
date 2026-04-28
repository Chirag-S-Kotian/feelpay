import { useState, useEffect } from 'react';
import { Send, Eye, AlertCircle, CheckCircle, Clock } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Merchant {
  id: string;
  name: string;
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
  payload: Record<string, any>;
}

const EVENT_TYPES = [
  'payment.success',
  'payment.failed',
  'refund.initiated',
  'settlement.processed',
  'dispute.created',
];

export default function AdminWebhookTesting() {
  const [merchants, setMerchants] = useState<Merchant[]>([]);
  const [selectedMerchantId, setSelectedMerchantId] = useState<string | null>(null);
  const [selectedEventType, setSelectedEventType] = useState('payment.success');
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [showPayload, setShowPayload] = useState(false);
  const [samplePayload, setSamplePayload] = useState<SamplePayload | null>(null);

  useEffect(() => {
    fetchMerchants();
  }, []);

  const fetchMerchants = async () => {
    try {
      const data = await apiRequest<{ merchants: Merchant[] }>(
        'GET',
        '/api/v1/admin/merchants?limit=100'
      );
      const merchantList = data.merchants || [];
      setMerchants(merchantList);
      if (merchantList[0]) {
        setSelectedMerchantId(merchantList[0].id);
      }
    } catch (err) {
      console.error('Failed to fetch merchants:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleTestWebhook = async () => {
    if (!selectedMerchantId) return;

    setTesting(true);
    setTestResult(null);

    try {
      const data = await apiRequest<TestResult>(
        'POST',
        `/api/v1/admin/merchants/${selectedMerchantId}/webhooks/test`,
        {
          event_type: selectedEventType,
        }
      );
      setTestResult(data);
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
      const data = await apiRequest<SamplePayload>(
        'GET',
        `/api/v1/admin/webhooks/sample-payload?event_type=${selectedEventType}`
      );
      setSamplePayload(data);
      setShowPayload(true);
    } catch (err) {
      console.error('Failed to fetch sample payload:', err);
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">Loading merchants...</div>
      </div>
    );
  }

  if (merchants.length === 0) {
    return (
      <div className="page-container">
        <div className="page-header">
          <h1>Webhook Testing</h1>
          <p>Test merchant webhook configurations</p>
        </div>
        <div className="card">
          <div className="empty-state">
            <AlertCircle size={32} />
            <p>No merchants found</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Webhook Testing</h1>
        <p>Send test events to merchant webhooks</p>
      </div>

      <div className="grid grid-2">
        <div className="card">
          <h2>Test Configuration</h2>

          <div className="form-group">
            <label>Merchant</label>
            <select
              value={selectedMerchantId || ''}
              onChange={e => setSelectedMerchantId(e.target.value)}
            >
              {merchants.map(m => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>Event Type</label>
            <select
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

        {testResult && (
          <div className="card">
            <h2>Test Result</h2>
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
            </div>
          </div>
        )}
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
        .result-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
          padding: 20px;
          border-radius: 8px;
          text-align: center;
        }
        .result-container.result-sent {
          background: #f0fdf4;
          color: #166534;
        }
        .result-container.result-failed {
          background: #fef2f2;
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
          background: var(--bg-secondary);
          padding: 8px;
          border-radius: 4px;
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
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .modal-content {
          background: var(--bg-primary);
          border-radius: 8px;
          box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
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
          border-bottom: 1px solid var(--border-color);
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
          color: var(--text-secondary);
        }
        .modal-body {
          padding: 20px;
        }
        .payload-json {
          background: var(--bg-secondary);
          padding: 12px;
          border-radius: 4px;
          font-size: 12px;
          overflow-x: auto;
          margin: 0;
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
