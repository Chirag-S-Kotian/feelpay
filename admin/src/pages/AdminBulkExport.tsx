import { useState, useEffect } from 'react';
import { Download, Calendar, AlertCircle, CheckCircle, Clock } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface ExportJob {
  job_id: string;
  status: 'queued' | 'processing' | 'completed' | 'failed';
  created_at: string;
  completed_at?: string;
  record_count?: number;
  file_size_bytes?: number;
  error_message?: string;
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('en-GB', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  }).format(date).replace(',', '');
}

export default function AdminBulkExport() {
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [format, setFormat] = useState<'csv' | 'excel'>('csv');
  const [entityTypes, setEntityTypes] = useState({
    transactions: true,
    settlements: true,
    refunds: false,
    disputes: false,
  });
  const [businessAccountFilter, setBusinessAccountFilter] = useState('all');
  const [businessAccounts, setBusinessAccounts] = useState<Array<{ id: string; name: string }>>([]);
  const [loading, setLoading] = useState(false);
  const [jobs, setJobs] = useState<ExportJob[]>([]);
  const [jobsLoading, setJobsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successJobId, setSuccessJobId] = useState<string | null>(null);

  useEffect(() => {
    fetchBusinessAccounts();
    fetchExportJobs();
    const interval = setInterval(fetchExportJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchBusinessAccounts = async () => {
    try {
      const data = await apiRequest<{ merchants: Array<{ id: string; name: string }> }>(
        'GET',
        '/admin/merchants'
      );
      setBusinessAccounts(data.merchants || []);
    } catch (err) {
      console.error('Failed to fetch business accounts:', err);
    }
  };

  const fetchExportJobs = async () => {
    try {
      const data = await apiRequest<{ exports: ExportJob[] }>(
        'GET',
        '/admin/exports?limit=20&offset=0'
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
      const selectedTypes = Object.entries(entityTypes)
        .filter(([, selected]) => selected)
        .map(([type]) => type);

      if (selectedTypes.length === 0) {
        setError('Please select at least one entity type');
        setLoading(false);
        return;
      }

      const response = await apiRequest<{ job_id: string }>(
        'POST',
        '/admin/exports/bulk',
        {
          date_range: { start: startDate, end: endDate },
          entity_types: selectedTypes,
          merchant_filter: businessAccountFilter === 'all' ? 'all' : businessAccountFilter,
          business_account_filter: businessAccountFilter === 'all' ? 'all' : businessAccountFilter,
          format,
        }
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
        `/admin/exports/${jobId}/download`
      );
      window.location.href = response.download_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Download failed');
    }
  };

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Bulk Export</h1>
        <p>Export platform-wide data for reconciliation</p>
      </div>

      <div className="grid grid-2">
        <div className="card">
          <h2>Export Configuration</h2>

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
            </div>
          )}

          <div className="form-group">
            <label>Date Range</label>
            <div className="date-range">
              <input
                type="date"
                value={startDate}
                onChange={e => setStartDate(e.target.value)}
              />
              <span>to</span>
              <input
                type="date"
                value={endDate}
                onChange={e => setEndDate(e.target.value)}
              />
            </div>
          </div>

          <div className="form-group">
            <label>Entity Types</label>
            <div className="checkbox-group">
              {Object.entries(entityTypes).map(([type, selected]) => (
                <label key={type} className="checkbox">
                  <input
                    type="checkbox"
                    checked={selected}
                    onChange={e => setEntityTypes({ ...entityTypes, [type]: e.target.checked })}
                  />
                  {type.charAt(0).toUpperCase() + type.slice(1)}
                </label>
              ))}
            </div>
          </div>

          <div className="form-group">
            <label>Business Account Filter</label>
            <select value={businessAccountFilter} onChange={e => setBusinessAccountFilter(e.target.value)}>
              <option value="all">All Business Accounts</option>
              {businessAccounts.map(m => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>Format</label>
            <select value={format} onChange={e => setFormat(e.target.value as 'csv' | 'excel')}>
              <option value="csv">CSV</option>
              <option value="excel">Excel (.xlsx)</option>
            </select>
          </div>

          <button
            className="btn-primary"
            onClick={handleExport}
            disabled={loading}
          >
            {loading ? 'Creating export...' : 'Export Data'}
          </button>
        </div>

        <div className="card">
          <h2>Export History</h2>
          {jobsLoading ? (
            <div className="loading">Loading...</div>
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
                    <th>Status</th>
                    <th>Records</th>
                    <th>Created</th>
                    <th>Action</th>
                  </tr>
                </thead>
                <tbody>
                  {jobs.map(job => (
                    <tr key={job.job_id}>
                      <td className="mono" title={job.job_id}>{job.job_id.slice(0, 8)}…{job.job_id.slice(-5)}</td>
                      <td>
                        <span className={`badge badge-${job.status}`}>
                          {job.status === 'processing' && <Clock size={12} />}
                          {job.status}
                        </span>
                      </td>
                      <td>{job.record_count || '-'}</td>
                      <td title={job.created_at}>{formatDateTime(job.created_at)}</td>
                      <td>
                        {job.status === 'completed' && (
                          <button
                            className="btn-small"
                            onClick={() => handleDownload(job.job_id)}
                          >
                            <Download size={14} />
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

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
        .checkbox-group {
          display: flex;
          flex-direction: column;
          gap: 12px;
          margin-top: 8px;
        }
        .form-group label.checkbox {
          display: flex;
          flex-direction: row;
          align-items: center;
          gap: 12px;
          margin: 0;
          cursor: pointer;
          color: var(--fp-text);
          text-transform: none;
          font-size: 0.95rem;
          font-weight: 500;
          letter-spacing: normal;
        }
        .form-group label.checkbox input[type="checkbox"] {
          appearance: none;
          -webkit-appearance: none;
          width: 20px;
          min-width: 20px;
          height: 20px;
          border: 1px solid var(--fp-border);
          border-radius: 4px;
          background: var(--fp-bg-elevated);
          margin: 0;
          display: grid;
          place-items: center;
          cursor: pointer;
          transition: var(--fp-transition);
        }
        .form-group label.checkbox:hover input[type="checkbox"] {
          border-color: var(--fp-text-muted);
        }
        .form-group label.checkbox input[type="checkbox"]::before {
          content: "";
          width: 12px;
          height: 12px;
          transform: scale(0);
          transition: 120ms transform ease-in-out;
          background-color: var(--spotify-tertiary);
          transform-origin: center;
          clip-path: polygon(14% 44%, 0 65%, 50% 100%, 100% 16%, 80% 0%, 43% 62%);
        }
        .form-group label.checkbox input[type="checkbox"]:checked {
          border-color: var(--spotify-tertiary);
        }
        .form-group label.checkbox input[type="checkbox"]:checked::before {
          transform: scale(1);
        }
        .date-range {
          display: flex;
          align-items: center;
          gap: 12px;
        }
        .date-range span {
          color: var(--fp-text-muted);
          font-weight: 500;
        }
      `}</style>
    </div>
  );
}
