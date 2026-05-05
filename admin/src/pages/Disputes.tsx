import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Dispute = {
  id: string;
  payment_order_id: string;
  merchant_id: string;
  amount: number;
  reason?: string | null;
  status: string;
  created_at: string;
};

export default function Disputes() {
  const [items, setItems] = useState<Dispute[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const params = new URLSearchParams({ limit: '100' });
        if (status) params.set('status', status);
        const data = await apiRequest<{ items: Dispute[] }>('GET', `/admin/disputes?${params.toString()}`);
        setItems(data.items || []);
      } catch (err) {
        console.error('Failed to load disputes', err);
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
          <h1>Disputes</h1>
          <p>Chargeback and dispute operations from database records</p>
        </div>
      </div>

      <div className="filter-bar">
        <select value={status} onChange={(e) => setStatus(e.target.value)} aria-label="Dispute status">
          <option value="">All statuses</option>
          <option value="needs_response">Needs response</option>
          <option value="under_review">Under review</option>
          <option value="won">Won</option>
          <option value="lost">Lost</option>
          <option value="accepted">Accepted</option>
        </select>
      </div>

      <div className="table-card" style={{ marginTop: 16 }}>
        {loading ? (
          <div style={{ padding: 20 }}>Loading disputes...</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Dispute ID</th>
                <th>Payment ID</th>
                <th>Business Account ID</th>
                <th>Amount</th>
                <th>Status</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              {items.map((d) => (
                <tr key={d.id}>
                  <td>{d.id}</td>
                  <td>{d.payment_order_id}</td>
                  <td>{d.merchant_id}</td>
                  <td>₹{d.amount.toLocaleString('en-IN')}</td>
                  <td><span className={`badge ${d.status}`}>{d.status}</span></td>
                  <td>{d.reason || '-'}</td>
                </tr>
              ))}
              {items.length === 0 && (
                <tr>
                  <td colSpan={6} style={{ textAlign: 'center', padding: 20 }}>No disputes found</td>
                </tr>
              )}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
