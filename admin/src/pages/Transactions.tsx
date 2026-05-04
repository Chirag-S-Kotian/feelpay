import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';

type Txn = {
  id: string;
  merchant_id: string;
  amount: number;
  currency?: string | null;
  status: string;
  payment_method?: string | null;
  created_at: string;
};

export default function Transactions() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [items, setItems] = useState<Txn[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState(searchParams.get('status') || '');
  const [query, setQuery] = useState(searchParams.get('q') || '');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const params = new URLSearchParams();
        if (status) params.set('status', status);
        if (query.trim()) params.set('q', query.trim());
        params.set('limit', '100');
        const data = await apiRequest<{ items: Txn[] }>('GET', `/admin/transactions?${params.toString()}`);
        setItems(data.items || []);
      } catch (err) {
        console.error('Failed to load transactions', err);
        setItems([]);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [status, query]);

  useEffect(() => {
    const next = new URLSearchParams();
    if (status) next.set('status', status);
    if (query.trim()) next.set('q', query.trim());
    setSearchParams(next, { replace: true });
  }, [query, setSearchParams, status]);

  const volume = useMemo(() => items.reduce((sum, it) => sum + (it.amount || 0), 0), [items]);

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Transactions</h1>
          <p>Live payment order feed from production database tables</p>
        </div>
      </div>

      <div className="stats-grid" style={{ marginBottom: 20 }}>
        <div className="stat-card">
          <div className="stat-label">Transactions</div>
          <div className="stat-value">{items.length.toLocaleString()}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Volume</div>
          <div className="stat-value">₹{volume.toLocaleString('en-IN')}</div>
        </div>
      </div>

      <div className="filter-bar">
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search by transaction or merchant id"
          aria-label="Search transactions"
        />
        <select value={status} onChange={(e) => setStatus(e.target.value)} aria-label="Transaction status">
          <option value="">All statuses</option>
          <option value="created">Created</option>
          <option value="captured">Captured</option>
          <option value="failed">Failed</option>
          <option value="refunded">Refunded</option>
        </select>
      </div>

      <div className="table-card" style={{ marginTop: 16 }}>
        {loading ? (
          <div style={{ padding: 20 }}>Loading transactions...</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Transaction ID</th>
                <th>Merchant ID</th>
                <th>Amount</th>
                <th>Method</th>
                <th>Status</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {items.map((tx) => (
                <tr key={tx.id}>
                  <td>{tx.id}</td>
                  <td>{tx.merchant_id}</td>
                  <td>₹{tx.amount.toLocaleString('en-IN')}</td>
                  <td>{tx.payment_method || 'N/A'}</td>
                  <td><span className={`badge ${tx.status}`}>{tx.status}</span></td>
                  <td>{new Date(tx.created_at).toLocaleString()}</td>
                </tr>
              ))}
              {items.length === 0 && (
                <tr>
                  <td colSpan={6} style={{ textAlign: 'center', padding: 20 }}>No transactions found</td>
                </tr>
              )}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
