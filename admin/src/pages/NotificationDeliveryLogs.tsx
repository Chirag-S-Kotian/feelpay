import { useState, useEffect } from 'react';
import { Filter, Eye, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface NotificationLog {
  notification_id: string;
  merchant_id: string;
  event_type: string;
  delivery_channel: 'email' | 'sms' | 'in_app';
  recipient: string;
  sent_at: string;
  status: 'sent' | 'failed' | 'bounced';
  error_message?: string;
  payload?: Record<string, any>;
}

interface Merchant {
  id: string;
  name: string;
}

export default function NotificationDeliveryLogs() {
  const [logs, setLogs] = useState<NotificationLog[]>([]);
  const [merchants, setMerchants] = useState<Merchant[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedLog, setSelectedLog] = useState<NotificationLog | null>(null);
  const [showDetail, setShowDetail] = useState(false);

  // Filters
  const [merchantId, setMerchantId] = useState('');
  const [eventType, setEventType] = useState('');
  const [channel, setChannel] = useState('');
  const [status, setStatus] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  useEffect(() => {
    fetchMerchants();
    fetchLogs();
  }, []);

  const fetchMerchants = async () => {
    try {
      const data = await apiRequest<{ merchants: Merchant[] }>(
        'GET',
        '/api/v1/admin/merchants?limit=100'
      );
      setMerchants(data.merchants || []);
    } catch (err) {
      console.error('Failed to fetch merchants:', err);
    }
  };

  const fetchLogs = async () => {
    try {
      const params = new URLSearchParams();
      if (merchantId) params.append('merchant_id', merchantId);
      if (eventType) params.append('event_type', eventType);
      if (channel) params.append('delivery_channel', channel);
      if (status) params.append('status', status);
      if (startDate) params.append('start_date', startDate);
      if (endDate) params.append('end_date', endDate);
      params.append('limit', '100');

      const data = await apiRequest<{ logs: NotificationLog[] }>(
        'GET',
        `/api/v1/admin/notification-delivery-logs?${params.toString()}`
      );
      setLogs(data.logs || []);
    } catch (err) {
      console.error('Failed to fetch logs:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleViewDetail = async (log: NotificationLog) => {
    try {
      const data = await apiRequest<NotificationLog>(
        'GET',
        `/api/v1/admin/notification-delivery-logs/${log.notification_id}`
      );
      setSelectedLog(data);
      setShowDetail(true);
    } catch (err) {
      console.error('Failed to fetch log detail:', err);
    }
  };

  const handleFilterChange = () => {
    setLoading(true);
    fetchLogs();
  };

  const EVENT_TYPES = [
    'payment.success',
    'payment.failed',
    'refund.initiated',
    'settlement.processed',
    'dispute.created',
  ];

  const CHANNELS = ['email', 'sms', 'in_app'];
  const STATUSES = ['sent', 'failed', 'bounced'];

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Notification Delivery Logs</h1>
        <p>Monitor notification delivery status and troubleshoot failures</p>
      </div>

      <div className="card">
        <h2>Filters</h2>
        <div className="filters-grid">
          <div className="filter-group">
            <label>Merchant</label>
            <select value={merchantId} onChange={e => setMerchantId(e.target.value)}>
              <option value="">All Merchants</option>
              {merchants.map(m => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>
          </div>

          <div className="filter-group">
            <label>Event Type</label>
            <select value={eventType} onChange={e => setEventType(e.target.value)}>
              <option value="">All Events</option>
              {EVENT_TYPES.map(et => (
                <option key={et} value={et}>
                  {et}
                </option>
              ))}
            </select>
          </div>

          <div className="filter-group">
            <label>Channel</label>
            <select value={channel} onChange={e => setChannel(e.target.value)}>
              <option value="">All Channels</option>
              {CHANNELS.map(ch => (
                <option key={ch} value={ch}>
                  {ch}
                </option>
              ))}
            </select>
          </div>

          <div className="filter-group">
            <label>Status</label>
            <select value={status} onChange={e => setStatus(e.target.value)}>
              <option value="">All Statuses</option>
              {STATUSES.map(st => (
                <option key={st} value={st}>
                  {st}
                </option>
              ))}
            </select>
          </div>

          <div className="filter-group">
            <label>Start Date</label>
            <input
              type="date"
              value={startDate}
              onChange={e => setStartDate(e.target.value)}
            />
          </div>

          <div className="filter-group">
            <label>End Date</label>
            <input
              type="date"
              value={endDate}
              onChange={e => setEndDate(e.target.value)}
            />
          </div>
        </div>

        <button className="btn-primary" onClick={handleFilterChange} disabled={loading}>
          <Filter size={16} />
          Apply Filters
        </button>
      </div>

      <div className="card">
        <h2>Delivery Logs</h2>
        {loading ? (
          <div className="loading">Loading logs...</div>
        ) : logs.length === 0 ? (
          <div className="empty-state">
            <AlertCircle size={32} />
            <p>No logs found</p>
          </div>
        ) : (
          <div className="table-container">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Notification ID</th>
                  <th>Merchant</th>
                  <th>Event Type</th>
                  <th>Channel</th>
                  <th>Recipient</th>
                  <th>Status</th>
                  <th>Sent At</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {logs.map(log => (
                  <tr key={log.notification_id}>
                    <td className="mono">{log.notification_id.slice(0, 8)}...</td>
                    <td>{merchants.find(m => m.id === log.merchant_id)?.name || log.merchant_id}</td>
                    <td>{log.event_type}</td>
                    <td>{log.delivery_channel}</td>
                    <td className="truncate">{log.recipient}</td>
                    <td>
                      <span className={`badge badge-${log.status}`}>
                        {log.status}
                      </span>
                    </td>
                    <td>{new Date(log.sent_at).toLocaleString()}</td>
                    <td>
                      <button
                        className="btn-small"
                        onClick={() => handleViewDetail(log)}
                      >
                        <Eye size={14} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {showDetail && selectedLog && (
        <div className="modal-overlay" onClick={() => setShowDetail(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Notification Details</h2>
              <button className="btn-close" onClick={() => setShowDetail(false)}>
                ✕
              </button>
            </div>

            <div className="modal-body">
              <div className="detail-section">
                <h3>Metadata</h3>
                <div className="detail-row">
                  <span>Notification ID:</span>
                  <strong className="mono">{selectedLog.notification_id}</strong>
                </div>
                <div className="detail-row">
                  <span>Event Type:</span>
                  <strong>{selectedLog.event_type}</strong>
                </div>
                <div className="detail-row">
                  <span>Channel:</span>
                  <strong>{selectedLog.delivery_channel}</strong>
                </div>
                <div className="detail-row">
                  <span>Recipient:</span>
                  <strong>{selectedLog.recipient}</strong>
                </div>
                <div className="detail-row">
                  <span>Status:</span>
                  <span className={`badge badge-${selectedLog.status}`}>
                    {selectedLog.status}
                  </span>
                </div>
                <div className="detail-row">
                  <span>Sent At:</span>
                  <strong>{new Date(selectedLog.sent_at).toLocaleString()}</strong>
                </div>
              </div>

              {selectedLog.error_message && (
                <div className="detail-section">
                  <h3>Error</h3>
                  <div className="error-box">
                    {selectedLog.error_message}
                  </div>
                </div>
              )}

              {selectedLog.payload && (
                <div className="detail-section">
                  <h3>Payload</h3>
                  <pre className="payload-json">
                    {JSON.stringify(selectedLog.payload, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      <style>{`
        .filters-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 16px;
        }
        .filter-group {
          display: flex;
          flex-direction: column;
        }
        .filter-group label {
          margin-bottom: 6px;
          font-weight: 500;
          font-size: 13px;
        }
        .filter-group input,
        .filter-group select {
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          font-size: 13px;
        }
        .truncate {
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
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
        .detail-section {
          margin-bottom: 20px;
        }
        .detail-section h3 {
          margin: 0 0 12px 0;
          font-size: 14px;
          font-weight: 600;
        }
        .detail-row {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          border-bottom: 1px solid var(--border-color);
          font-size: 13px;
        }
        .detail-row:last-child {
          border-bottom: none;
        }
        .error-box {
          background: #fef2f2;
          border: 1px solid #fca5a5;
          border-radius: 4px;
          padding: 12px;
          color: #991b1b;
          font-size: 13px;
          font-family: monospace;
        }
        .payload-json {
          background: var(--bg-secondary);
          padding: 12px;
          border-radius: 4px;
          font-size: 11px;
          overflow-x: auto;
          margin: 0;
        }
      `}</style>
    </div>
  );
}
