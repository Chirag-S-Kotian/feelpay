import { useState, useEffect } from 'react';
import { Download, Calendar, Filter, Settings, Copy, CheckCircle, AlertCircle, Clock } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

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

  const merchantId = localStorage.getItem('feelpay_merchant_id') || 'test-merchant';

  useEffect(() => {
    fetchExportJobs();
    const interval = setInterval(fetchExportJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchExportJobs = async () => {
    try {
      const data = await apiRequest<{ exports: ExportJob[] }>(
        'GET',
        `/api/v1/merchants/${merchantId}/exports?limit=20&offset=0`
      );
      setJobs(data.exports || []);
    } catch (err) {
      console.error('Failed to fetch export jobs:', err);
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
        `/api/v1/merchants/${merchantId}/exports/transactions`,
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
        `/api/v1/merchants/${merchantId}/exports/${jobId}/download`
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
    <div className="page-container">
      <div className="page-header">
        <h1>Transaction Exports</h1>
        <p>Export transactions for reconciliation and analysis</p>
      </div>

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
    </div>
  );
}
