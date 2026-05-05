import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Settlement = {
  id: string;
  merchant_id: string;
  amount: number;
  status: string;
  created_at: string;
  processed_at?: string | null;
};

export default function Settlements() {
  const [items, setItems] = useState<Settlement[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const params = new URLSearchParams({ limit: '100' });
        if (status) params.set('status', status);
        const data = await apiRequest<{ items: Settlement[] }>('GET', `/admin/settlements?${params.toString()}`);
        setItems(data.items || []);
      } catch (err) {
        console.error('Failed to load settlements', err);
        setItems([]);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [status]);

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Settlements</h1>
          <p>Settlement pipeline and payouts from live DB rows</p>
        </div>
      </div>

      <div className="filter-bar">
        <select value={status} onChange={(e) => setStatus(e.target.value)} aria-label="Settlement status">
          <option value="">All statuses</option>
          <option value="pending">Pending</option>
          <option value="processing">Processing</option>
          <option value="processed">Processed</option>
          <option value="paid">Paid</option>
          <option value="failed">Failed</option>
        </select>
      </div>

      <div className="table-card" style={{ marginTop: 16 }}>
        {loading ? (
          <div style={{ padding: 20 }}>Loading settlements...</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Settlement ID</th>
                <th>Business Account ID</th>
                <th>Amount</th>
                <th>Status</th>
                <th>Created</th>
                <th>Processed</th>
              </tr>
            </thead>
            <tbody>
              {items.map((s) => (
                <tr key={s.id}>
                  <td>{s.id}</td>
                  <td>{s.merchant_id}</td>
                  <td>₹{s.amount.toLocaleString('en-IN')}</td>
                  <td><span className={`badge ${s.status}`}>{s.status}</span></td>
                  <td>{new Date(s.created_at).toLocaleString()}</td>
                  <td>{s.processed_at ? new Date(s.processed_at).toLocaleString() : '-'}</td>
                </tr>
              ))}
              {items.length === 0 && (
                <tr>
                  <td colSpan={6} style={{ textAlign: 'center', padding: 20 }}>No settlements found</td>
                </tr>
              )}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
