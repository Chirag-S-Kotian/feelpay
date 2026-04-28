import { useState, useEffect } from 'react';
import { Download, Calendar, Filter, AlertCircle, CheckCircle, Clock } from 'lucide-react';
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
  const [merchantFilter, setMerchantFilter] = useState('all');
  const [merchants, setMerchants] = useState<Array<{ id: string; name: string }>>([]);
  const [loading, setLoading] = useState(false);
  const [jobs, setJobs] = useState<ExportJob[]>([]);
  const [jobsLoading, setJobsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successJobId, setSuccessJobId] = useState<string | null>(null);

  useEffect(() => {
    fetchMerchants();
    fetchExportJobs();
    const interval = setInterval(fetchExportJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchMerchants = async () => {
    try {
      const data = await apiRequest<{ merchants: Array<{ id: string; name: string }> }>(
        'GET',
        '/api/v1/admin/merchants?limit=100'
      );
      setMerchants(data.merchants || []);
    } catch (err) {
      console.error('Failed to fetch merchants:', err);
    }
  };

  const fetchExportJobs = async () => {
    try {
      const data = await apiRequest<{ exports: ExportJob[] }>(
        'GET',
        '/api/v1/admin/exports?limit=20&offset=0'
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
        '/api/v1/admin/exports/bulk',
        {
          date_range: { start: startDate, end: endDate },
          entity_types: selectedTypes,
          merchant_filter: merchantFilter === 'all' ? 'all' : merchantFilter,
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
        `/api/v1/admin/exports/${jobId}/download`
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
            <label>Merchant Filter</label>
            <select value={merchantFilter} onChange={e => setMerchantFilter(e.target.value)}>
              <option value="all">All Merchants</option>
              {merchants.map(m => (
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
                      <td className="mono">{job.job_id.slice(0, 8)}...</td>
                      <td>
                        <span className={`badge badge-${job.status}`}>
                          {job.status === 'processing' && <Clock size={12} />}
                          {job.status}
                        </span>
                      </td>
                      <td>{job.record_count || '-'}</td>
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
          gap: 8px;
        }
        .checkbox {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }
        .checkbox input {
          cursor: pointer;
        }
      `}</style>
    </div>
  );
}
