import { useState, useEffect, FormEvent } from 'react';
import { apiRequest } from '../hooks/useApi';

type Txn = {
  id: string;
  order_id: string;
  merchant_id: string;
  amount: number;
  currency: string;
  status: string;
  payment_method?: string;
  customer_email?: string;
  is_frozen: boolean;
  is_flagged: boolean;
  created_at: string;
};

const STATUS_OPTIONS = [
  { label: 'All Statuses', value: '' },
  { label: 'Captured', value: 'captured' },
  { label: 'Authorized', value: 'authorized' },
  { label: 'Pending', value: 'pending' },
  { label: 'Processing', value: 'processing' },
  { label: 'Failed', value: 'failed' },
  { label: 'Refunded', value: 'refunded' },
  { label: 'Partially Refunded', value: 'partially_refunded' },
];

export default function TransactionSearch() {
  const [filters, setFilters] = useState({ order_id: '', customer_email: '', status: '' });
  const [rows, setRows] = useState<Txn[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    void handleSearch();
  }, []);

  async function handleSearch(event?: FormEvent) {
    if (event) event.preventDefault();
    setLoading(true);
    try {
      const params = new URLSearchParams();
      Object.entries(filters).forEach(([key, value]) => value && params.set(key, value));
      const data = await apiRequest<{ results: Txn[] }>('GET', `/v1/admin/transactions/search?${params}`);
      setRows(data.results || []);
    } catch {
      setRows([]);
    } finally {
      setLoading(false);
    }
  }

  function handleReset() {
    setFilters({ order_id: '', customer_email: '', status: '' });
    void apiRequest<{ results: Txn[] }>('GET', '/v1/admin/transactions/search').then(data => setRows(data.results || [])).catch(() => setRows([]));
  }

  return (
    <section className="page-shell">
      <header className="page-header">
        <div>
          <h1>Transaction Search</h1>
          <p style={{ color: 'var(--text-muted, #64748b)', fontSize: '13px', margin: 0 }}>
            Query global transactions by order reference, customer email, or status
          </p>
        </div>
      </header>

      <form className="card filter-grid" onSubmit={handleSearch}>
        <label className="form-field">
          <span>Order ID</span>
          <input
            placeholder="ord_..."
            value={filters.order_id}
            onChange={event => setFilters({ ...filters, order_id: event.target.value })}
          />
        </label>
        <label className="form-field">
          <span>Customer Email</span>
          <input
            type="email"
            placeholder="customer@example.com"
            value={filters.customer_email}
            onChange={event => setFilters({ ...filters, customer_email: event.target.value })}
          />
        </label>
        <label className="form-field">
          <span>Status</span>
          <select
            value={filters.status}
            onChange={event => setFilters({ ...filters, status: event.target.value })}
          >
            {STATUS_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
        </label>
        <div className="filter-actions">
          <button className="btn-primary" type="submit" disabled={loading}>
            {loading ? 'Searching...' : 'Search'}
          </button>
          <button className="btn-secondary" type="button" onClick={handleReset}>
            Reset
          </button>
        </div>
      </form>

      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Order</th>
              <th>Status</th>
              <th>Amount</th>
              <th>Customer</th>
              <th>Flags</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ textAlign: 'center', padding: '32px', color: 'var(--text-muted, #64748b)' }}>
                  {loading ? 'Searching transactions...' : 'No transactions found. Use filters above to search.'}
                </td>
              </tr>
            ) : (
              rows.map(row => (
                <tr key={row.id}>
                  <td>{row.order_id}</td>
                  <td>
                    <span className={`badge ${row.status === 'captured' || row.status === 'completed' ? 'success' : row.status === 'pending' || row.status === 'processing' ? 'pending' : 'failed'}`}>
                      {row.status}
                    </span>
                  </td>
                  <td>{row.currency} {(row.amount / 100).toFixed(2)}</td>
                  <td>{row.customer_email || 'consumer@feelpay.local'}</td>
                  <td>{[row.is_frozen && 'Frozen', row.is_flagged && 'Flagged'].filter(Boolean).join(', ') || 'Standard (Clear)'}</td>
                  <td>{new Date(row.created_at).toLocaleString()}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
