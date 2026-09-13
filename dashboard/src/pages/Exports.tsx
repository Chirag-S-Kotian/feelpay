import { useState, useEffect } from 'react';
import { Download, Calendar, Filter, Settings, Copy, CheckCircle, AlertCircle, Clock } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import ReportNavTabs from '../components/ReportNavTabs';

interface ExportJob {
  job_id: string;
  job_type: string;
  status: 'queued' | 'processing' | 'completed' | 'failed';
  created_at: string;
  completed_at?: string;
  record_count?: number;
  file_size_bytes?: number;
  error_message?: string;
  progress?: { percent: number; records_processed: number; estimated_time_remaining_seconds: number };
}

interface ExportRequest {
  date_range: { start: string; end: string };
  format: 'csv' | 'excel';
  filters?: {
    status?: string[];
    payment_method?: string[];
    amount_range?: { min: number; max: number };
  };
  columns?: string[];
}

export default function Exports() {
  const [activeTab, setActiveTab] = useState<'export' | 'history'>('export');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [format, setFormat] = useState<'csv' | 'excel'>('csv');
  const [selectedColumns, setSelectedColumns] = useState<string[]>([
    'transaction_id',
    'amount_paise',
    'payment_method',
    'status',
    'created_at',
  ]);
  const [filters, setFilters] = useState({
    status: [] as string[],
    payment_method: [] as string[],
    amount_min: 0,
    amount_max: 1000000,
  });
  const [showFilters, setShowFilters] = useState(false);
  const [showColumns, setShowColumns] = useState(false);
  const [loading, setLoading] = useState(false);
  const [jobs, setJobs] = useState<ExportJob[]>([]);
  const [jobsLoading, setJobsLoading] = useState(true);
  const [successJobId, setSuccessJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

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
    fetchExportJobs();
    const interval = setInterval(fetchExportJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchExportJobs = async () => {
    try {
      const data = await apiRequest<{ exports: ExportJob[] }>(
        'GET',
        `/v1/merchants/${merchantId}/exports?limit=20&offset=0`
      );
      setJobs(data.exports || []);
    } catch (err) {
      console.error('Failed to fetch export jobs:', err);
      setError(
        err instanceof Error
          ? err.message
          : 'Failed to fetch export jobs. Check backend is running.'
      );
    } finally {
      setJobsLoading(false);
    }
  };

  const handleExport = async () => {
    if (!startDate || !endDate) {
      setError('Please select both start and end dates');
      return;
    }

    setLoading(true);
    setError(null);
    setSuccessJobId(null);

    try {
      const payload: ExportRequest = {
        date_range: { start: startDate, end: endDate },
        format,
        columns: selectedColumns,
        filters: {
          status: filters.status.length > 0 ? filters.status : undefined,
          payment_method: filters.payment_method.length > 0 ? filters.payment_method : undefined,
          amount_range:
            filters.amount_min > 0 || filters.amount_max < 1000000
              ? { min: filters.amount_min, max: filters.amount_max }
              : undefined,
        },
      };

      const response = await apiRequest<{ job_id: string; status: string }>(
        'POST',
        `/v1/merchants/${merchantId}/exports/transactions`,
        payload
      );

      setSuccessJobId(response.job_id);
      await fetchExportJobs();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Export failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async (jobId: string) => {
    try {
      const response = await apiRequest<{ download_url: string }>(
        'GET',
        `/v1/merchants/${merchantId}/exports/${jobId}/download`
      );
      window.location.href = response.download_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Download failed');
    }
  };

  const allColumns = [
    'transaction_id',
    'order_id',
    'customer_email',
    'customer_phone',
    'amount_paise',
    'currency',
    'payment_method',
    'status',
    'created_at',
    'settled_at',
    'refund_amount',
    'refund_status',
  ];

  return (
    <div className="page-container exports-page">
      <div className="page-header">
        <h1>Transaction Exports</h1>
        <p>Export transactions for reconciliation and analysis</p>
      </div>

      <ReportNavTabs />

      <div className="tabs">
        <button
          className={`tab ${activeTab === 'export' ? 'active' : ''}`}
          onClick={() => setActiveTab('export')}
        >
          New Export
        </button>
        <button
          className={`tab ${activeTab === 'history' ? 'active' : ''}`}
          onClick={() => setActiveTab('history')}
        >
          Export History
        </button>
      </div>

      {activeTab === 'export' && (
        <div className="card">
          <div className="form-section">
            <h2>Export Transactions</h2>

            {error && (
              <div className="alert alert-error">
                <AlertCircle size={16} />
                {error}
              </div>
            )}

            {successJobId && (
              <div className="alert alert-success">
                <CheckCircle size={16} />
                Export job created! Job ID: {successJobId}
                <button
                  className="btn-copy"
                  onClick={() => {
                    navigator.clipboard.writeText(successJobId);
                  }}
                >
                  <Copy size={14} />
                </button>
              </div>
            )}

            <div className="form-group">
              <label>Date Range</label>
              <div className="date-range">
                <input
                  type="date"
                  value={startDate}
                  onChange={e => setStartDate(e.target.value)}
                  placeholder="Start date"
                />
                <span>to</span>
                <input
                  type="date"
                  value={endDate}
                  onChange={e => setEndDate(e.target.value)}
                  placeholder="End date"
                />
              </div>
            </div>

            <div className="form-group">
              <label>Format</label>
              <select value={format} onChange={e => setFormat(e.target.value as 'csv' | 'excel')}>
                <option value="csv">CSV</option>
                <option value="excel">Excel (.xlsx)</option>
              </select>
            </div>

            <div className="form-actions">
              <button
                className="btn-secondary"
                onClick={() => setShowFilters(!showFilters)}
              >
                <Filter size={16} />
                Filters
              </button>
              <button
                className="btn-secondary"
                onClick={() => setShowColumns(!showColumns)}
              >
                <Settings size={16} />
                Columns
              </button>
            </div>

            {showFilters && (
              <div className="filter-panel">
                <h3>Filters</h3>
                <div className="filter-group">
                  <label>Status</label>
                  <div className="checkbox-group">
                    {['success', 'failed', 'pending'].map(s => (
                      <label key={s} className="checkbox">
                        <input
                          type="checkbox"
                          checked={filters.status.includes(s)}
                          onChange={e => {
                            if (e.target.checked) {
                              setFilters(f => ({ ...f, status: [...f.status, s] }));
                            } else {
                              setFilters(f => ({ ...f, status: f.status.filter(x => x !== s) }));
                            }
                          }}
                        />
                        {s}
                      </label>
                    ))}
                  </div>
                </div>

                <div className="filter-group">
                  <label>Payment Method</label>
                  <div className="checkbox-group">
                    {['card', 'upi', 'netbanking'].map(m => (
                      <label key={m} className="checkbox">
                        <input
                          type="checkbox"
                          checked={filters.payment_method.includes(m)}
                          onChange={e => {
                            if (e.target.checked) {
                              setFilters(f => ({ ...f, payment_method: [...f.payment_method, m] }));
                            } else {
                              setFilters(f => ({
                                ...f,
                                payment_method: f.payment_method.filter(x => x !== m),
                              }));
                            }
                          }}
                        />
                        {m}
                      </label>
                    ))}
                  </div>
                </div>

                <div className="filter-group">
                  <label>Amount Range (₹)</label>
                  <div className="amount-range">
                    <input
                      type="number"
                      value={filters.amount_min}
                      onChange={e => setFilters(f => ({ ...f, amount_min: parseInt(e.target.value) }))}
                      placeholder="Min"
                    />
                    <input
                      type="number"
                      value={filters.amount_max}
                      onChange={e => setFilters(f => ({ ...f, amount_max: parseInt(e.target.value) }))}
                      placeholder="Max"
                    />
                  </div>
                </div>
              </div>
            )}

            {showColumns && (
              <div className="columns-panel">
                <h3>Select Columns</h3>
                <div className="checkbox-group">
                  {allColumns.map(col => (
                    <label key={col} className="checkbox">
                      <input
                        type="checkbox"
                        checked={selectedColumns.includes(col)}
                        onChange={e => {
                          if (e.target.checked) {
                            setSelectedColumns([...selectedColumns, col]);
                          } else {
                            setSelectedColumns(selectedColumns.filter(x => x !== col));
                          }
                        }}
                      />
                      {col}
                    </label>
                  ))}
                </div>
              </div>
            )}

            <button
              className="btn-primary"
              onClick={handleExport}
              disabled={loading}
            >
              {loading ? 'Creating export...' : 'Export Transactions'}
            </button>
          </div>
        </div>
      )}

      {activeTab === 'history' && (
        <div className="card">
          <h2>Export History</h2>
          {jobsLoading ? (
            <div className="loading">Loading export history...</div>
          ) : jobs.length === 0 ? (
            <div className="empty-state">
              <Calendar size={32} />
              <p>No exports yet</p>
            </div>
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Job ID</th>
                    <th>Type</th>
                    <th>Status</th>
                    <th>Records</th>
                    <th>Size</th>
                    <th>Created</th>
                    <th>Action</th>
                  </tr>
                </thead>
                <tbody>
                  {jobs.map(job => (
                    <tr key={job.job_id}>
                      <td className="mono">{job.job_id.slice(0, 8)}...</td>
                      <td>{job.job_type}</td>
                      <td>
                        <span className={`badge badge-${job.status}`}>
                          {job.status === 'processing' && <Clock size={12} />}
                          {job.status}
                        </span>
                      </td>
                      <td>{job.record_count || '-'}</td>
                      <td>{job.file_size_bytes ? `${(job.file_size_bytes / 1024).toFixed(2)} KB` : '-'}</td>
                      <td>{new Date(job.created_at).toLocaleDateString()}</td>
                      <td>
                        {job.status === 'completed' && (
                          <button
                            className="btn-small"
                            onClick={() => handleDownload(job.job_id)}
                          >
                            <Download size={14} />
                          </button>
                        )}
                        {job.status === 'failed' && (
                          <span className="error-text" title={job.error_message}>
                            Error
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      <style>{`
        .exports-page .tabs {
          display: inline-flex;
          gap: 8px;
          padding: 6px;
          border-radius: 16px;
          border: 1px solid var(--fp-border);
          background: #e7decb;
          margin-bottom: 18px;
        }

        .exports-page .tab {
          border: 0;
          border-radius: 12px;
          padding: 9px 14px;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted) !important;
          background: transparent;
          cursor: pointer;
        }

        .exports-page .tab.active {
          color: #fff9ec !important;
          background: var(--fp-cta);
          border: 1px solid #d74a58;
          box-shadow: 0 8px 14px rgba(235, 0, 27, 0.24);
        }

        .exports-page .card {
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .exports-page h2 {
          margin: 0 0 14px;
          font-size: 21px;
        }

        .exports-page .form-group {
          margin-bottom: 14px;
        }

        .exports-page .form-group > label {
          display: block;
          margin-bottom: 8px;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
        }

        .exports-page .date-range {
          display: grid;
          grid-template-columns: 1fr auto 1fr;
          gap: 10px;
          align-items: center;
        }

        .exports-page .date-range span {
          color: var(--fp-text-muted);
          font-size: 13px;
        }

        .exports-page input,
        .exports-page select {
          height: 42px;
          width: 100%;
          min-width: 0;
          padding: 0 12px;
          font-size: 14px;
          border-radius: 999px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text);
        }

        .exports-page input:focus,
        .exports-page select:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .exports-page select {
          appearance: none;
          background-image:
            linear-gradient(45deg, transparent 50%, #8a857d 50%),
            linear-gradient(135deg, #8a857d 50%, transparent 50%);
          background-position:
            calc(100% - 16px) calc(50% + 1px),
            calc(100% - 11px) calc(50% + 1px);
          background-size: 5px 5px, 5px 5px;
          background-repeat: no-repeat;
          padding-right: 30px;
        }

        .exports-page .form-actions {
          display: flex;
          gap: 10px;
          justify-content: flex-end;
          margin: 12px 0;
        }

        .exports-page .btn-primary,
        .exports-page .btn-secondary {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          border-radius: 10px;
          font-weight: 600;
          border: 1px solid var(--fp-border);
          cursor: pointer;
        }

        .exports-page .btn-primary {
          height: 44px;
          padding: 0 16px;
          border: 0;
          color: #fff9ec;
          background: var(--fp-cta);
        }

        .exports-page .btn-secondary {
          height: 40px;
          padding: 0 12px;
          color: var(--fp-text);
          background: #f2ead8;
        }

        .exports-page .filter-panel,
        .exports-page .columns-panel {
          margin: 12px 0;
          padding: 14px;
          border-radius: 12px;
          border: 1px solid var(--fp-border);
          background: #f7f0e1;
        }

        .exports-page .checkbox-group {
          display: flex;
          flex-wrap: wrap;
          gap: 10px;
        }

        .exports-page .checkbox {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          font-size: 13px;
          color: var(--fp-text-muted);
          padding: 6px 10px;
          border: 1px solid var(--fp-border);
          border-radius: 8px;
          background: #f2ead8;
        }

        .exports-page .checkbox input {
          width: 14px;
          height: 14px;
        }

        .exports-page .amount-range {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 10px;
        }

        .exports-page .alert {
          margin-bottom: 12px;
          border-radius: 10px;
          padding: 10px 12px;
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 13px;
        }

        .exports-page .alert-error {
          border: 1px solid rgba(239, 68, 68, 0.45);
          color: #991627;
          background: rgba(200, 31, 47, 0.1);
        }

        .exports-page .alert-success {
          border: 1px solid rgba(34, 197, 94, 0.4);
          color: #2f6b40;
          background: rgba(52, 122, 71, 0.1);
        }

        .exports-page .btn-copy {
          margin-left: auto;
          border: 1px solid rgba(148, 163, 184, 0.3);
          background: #f2ead8;
          color: var(--fp-text);
          width: 28px;
          height: 28px;
          border-radius: 8px;
          display: grid;
          place-items: center;
          cursor: pointer;
        }

        .exports-page .mono {
          font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
          color: var(--fp-primary-soft);
        }

        .exports-page .btn-small {
          border: 1px solid var(--fp-border);
          background: #f2ead8;
          color: var(--fp-text);
          width: 30px;
          height: 30px;
          border-radius: 8px;
          display: grid;
          place-items: center;
          cursor: pointer;
        }

        .exports-page .error-text {
          color: #fca5a5;
          font-size: 12px;
        }

        @media (max-width: 900px) {
          .exports-page .date-range {
            grid-template-columns: 1fr;
          }

          .exports-page .form-actions {
            justify-content: flex-start;
            flex-wrap: wrap;
          }

          .exports-page .amount-range {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
}
