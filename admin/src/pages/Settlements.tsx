import { useEffect, useState } from 'react';
import { Landmark } from 'lucide-react';
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
          <p>Merchant Payouts & Net Earnings — settlement pipeline and payouts to merchant bank accounts</p>
        </div>
      </div>

      <div
        style={{
          marginBottom: '1.25rem',
          padding: '1.1rem 1.25rem',
          display: 'flex',
          alignItems: 'flex-start',
          gap: '1rem',
          background: '#0f241a',
          border: '1px solid #10b981',
          borderRadius: '0.75rem',
          boxShadow: '0 4px 16px rgba(0, 0, 0, 0.4)'
        }}
      >
        <div
          style={{
            padding: '0.625rem',
            borderRadius: '0.5rem',
            background: 'rgba(16, 185, 129, 0.25)',
            color: '#34d399',
            flexShrink: 0
          }}
        >
          <Landmark size={22} />
        </div>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.6rem', marginBottom: '0.35rem' }}>
            <h4 style={{ margin: 0, fontSize: '1rem', fontWeight: 700, color: '#ffffff', letterSpacing: '-0.01em' }}>
              Settlements = Sales Payouts to Merchant / Business
            </h4>
            <span
              style={{
                fontSize: '0.75rem',
                fontWeight: 700,
                padding: '0.2rem 0.6rem',
                borderRadius: '1rem',
                background: 'rgba(16, 185, 129, 0.3)',
                color: '#6ee7b7',
                border: '1px solid rgba(110, 231, 183, 0.4)'
              }}
            >
              Merchant Payout
            </span>
          </div>
          <p style={{ margin: 0, fontSize: '0.88rem', color: '#f1f5f9', lineHeight: 1.5 }}>
            A <strong style={{ color: '#ffffff', fontWeight: 600 }}>Settlement</strong> transfers batched captured sales earnings directly into the <strong style={{ color: '#ffffff', fontWeight: 600 }}>merchant's registered business bank account</strong> after deducting fees and reserve holdbacks.
            <span style={{ marginLeft: '0.5rem', color: '#34d399', fontWeight: 600 }}>
              (Differs from Refund: Settlements are business payouts to sellers; Refunds are money returned to consumers).
            </span>
          </p>
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
