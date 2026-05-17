import { useState, useEffect } from 'react';
import { Plus, Edit2, Trash2, Clock } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import ScheduleReportForm from '../components/ScheduleReportForm';

interface ScheduledReport {
  id: string;
  report_type: string;
  frequency: 'daily' | 'weekly' | 'monthly';
  day_of_week?: number;
  day_of_month?: number;
  time_of_day: string;
  recipient_emails: string[];
  format: 'csv' | 'pdf' | 'excel';
  is_active: boolean;
  next_run_at: string;
  created_at: string;
}

export default function ScheduledReports() {
  const [reports, setReports] = useState<ScheduledReport[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editingReport, setEditingReport] = useState<ScheduledReport | null>(null);
  const [error, setError] = useState('');

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
    fetchScheduledReports();
  }, []);

  const fetchScheduledReports = async () => {
    if (!merchantId) {
      setError('Merchant session missing. Login again.');
      setLoading(false);
      return;
    }
    try {
      setError('');
      const data = await apiRequest<{ reports?: ScheduledReport[]; scheduled_reports?: ScheduledReport[] }>(
        'GET',
        `/v1/merchants/${merchantId}/scheduled-reports`
      );
      setReports(data.reports || data.scheduled_reports || []);
    } catch (err) {
      console.error('Failed to fetch scheduled reports:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch scheduled reports');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (reportId: string) => {
    if (!confirm('Delete this scheduled report?')) return;

    try {
      await apiRequest('DELETE', `/v1/merchants/${merchantId}/scheduled-reports/${reportId}`);
      setReports(reports.filter(r => r.id !== reportId));
    } catch (err) {
      console.error('Failed to delete report:', err);
    }
  };

  const handleFormClose = () => {
    setShowForm(false);
    setEditingReport(null);
    fetchScheduledReports();
  };

  const getFrequencyLabel = (report: ScheduledReport) => {
    if (report.frequency === 'daily') return 'Daily';
    if (report.frequency === 'weekly') {
      const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
      return `Weekly (${days[report.day_of_week || 0]})`;
    }
    return `Monthly (Day ${report.day_of_month})`;
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">Loading scheduled reports...</div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <div>
          <h1>Scheduled Reports</h1>
          <p>Manage recurring report generation and delivery</p>
        </div>
      </div>
      {error && <div className="error-banner">{error}</div>}

      {showForm && (
        <ScheduleReportForm
          report={editingReport}
          onClose={handleFormClose}
        />
      )}

      <div className="card">
        <div className="card-header">
          <h2>Active Schedules</h2>
          {reports.length > 0 && (
            <button
              className="btn-primary"
              onClick={() => {
                setEditingReport(null);
                setShowForm(true);
              }}
            >
              <Plus size={16} />
              Schedule Report
            </button>
          )}
        </div>

        {reports.length === 0 ? (
          <div className="empty-state-container">
            <Clock size={40} className="empty-icon" />
            <h3 className="empty-title">No Scheduled Reports</h3>
            <p className="empty-description">
              Create recurring reports for finance, settlement, GST, or other metrics to be delivered directly to your inbox.
            </p>
            <button
              className="btn-primary empty-btn"
              onClick={() => {
                setEditingReport(null);
                setShowForm(true);
              }}
            >
              <Plus size={16} />
              Create First Schedule
            </button>
          </div>
        ) : (
          <div className="table-container">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Report Type</th>
                  <th>Frequency</th>
                  <th>Time</th>
                  <th>Recipients</th>
                  <th>Format</th>
                  <th>Next Run</th>
                  <th>Status</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {reports.map(report => (
                  <tr key={report.id}>
                    <td className="font-semibold">{report.report_type}</td>
                    <td>{getFrequencyLabel(report)}</td>
                    <td className="mono">{report.time_of_day}</td>
                    <td>
                      <div className="recipients-list">
                        {report.recipient_emails.slice(0, 2).map((email, i) => (
                          <span key={i} className="recipient-badge">{email}</span>
                        ))}
                        {report.recipient_emails.length > 2 && (
                          <span className="recipient-badge">+{report.recipient_emails.length - 2}</span>
                        )}
                      </div>
                    </td>
                    <td>{report.format.toUpperCase()}</td>
                    <td>{new Date(report.next_run_at).toLocaleString()}</td>
                    <td>
                      <span className={`badge badge-${report.is_active ? 'success' : 'secondary'}`}>
                        {report.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </td>
                    <td>
                      <div className="action-buttons">
                        <button
                          className="btn-icon"
                          onClick={() => {
                            setEditingReport(report);
                            setShowForm(true);
                          }}
                          title="Edit"
                        >
                          <Edit2 size={14} />
                        </button>
                        <button
                          className="btn-icon btn-danger"
                          onClick={() => handleDelete(report.id)}
                          title="Delete"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <style>{`
        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
          border-bottom: 1px solid var(--fp-border);
          padding-bottom: 16px;
        }
        .card-header h2 {
          font-size: 1.35rem;
          font-weight: 700;
          color: var(--fp-text);
          margin: 0;
        }
        .card-header button {
          display: inline-flex;
          align-items: center;
          gap: 6px;
        }
        .empty-state-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          text-align: center;
          padding: 60px 40px;
          background: rgba(235, 0, 27, 0.02);
          border: 1px dashed rgba(235, 0, 27, 0.15);
          border-radius: 16px;
          margin: 20px 0;
          transition: all 0.3s ease;
        }
        .empty-state-container:hover {
          background: rgba(235, 0, 27, 0.04);
          border-color: rgba(235, 0, 27, 0.3);
        }
        .empty-icon {
          color: var(--fp-text-muted);
          margin-bottom: 18px;
          opacity: 0.85;
          animation: pulse 3s infinite ease-in-out;
        }
        .empty-title {
          font-size: 1.2rem;
          font-weight: 600;
          color: var(--fp-text);
          margin: 0 0 8px 0;
        }
        .empty-description {
          font-size: 0.9rem;
          color: var(--fp-text-muted);
          max-width: 480px;
          line-height: 1.5;
          margin: 0 0 20px 0;
        }
        .empty-btn {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          padding: 10px 20px;
          border-radius: 9999px;
          font-weight: 600;
          transition: all 0.2s ease;
        }
        @keyframes pulse {
          0%, 100% { transform: scale(1); opacity: 0.85; }
          50% { transform: scale(1.05); opacity: 1; }
        }
        .error-banner {
          margin-bottom: 16px;
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
          border-radius: 12px;
          padding: 10px 14px;
          font-size: 13px;
          font-weight: 600;
        }
        .recipients-list {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
        }
        .recipient-badge {
          display: inline-block;
          padding: 2px 8px;
          background: var(--bg-secondary);
          border-radius: 4px;
          font-size: 12px;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          max-width: 120px;
        }
        .action-buttons {
          display: flex;
          gap: 8px;
        }
        .btn-icon {
          padding: 6px;
          background: var(--bg-secondary);
          border: 1px solid var(--border-color);
          border-radius: 4px;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.2s;
        }
        .btn-icon:hover {
          background: var(--bg-tertiary);
        }
        .btn-icon.btn-danger:hover {
          background: #fee;
          border-color: #f00;
          color: #f00;
        }
      `}</style>
    </div>
  );
}
