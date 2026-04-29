import { useState, useEffect } from 'react';
import { apiRequest } from '../hooks/useApi';

type Payment = {
  id: string;
  order_id: string;
  amount: number;
  payment_method?: string | null;
  connector?: string | null;
  status: string;
  customer_email?: string | null;
  created_at: string;
};

function SkeletonRow() {
  return (
    <tr>
      <td><div className="skeleton" style={{ width: 120, height: 16 }}></div></td>
      <td><div className="skeleton" style={{ width: 100, height: 16 }}></div></td>
      <td><div className="skeleton" style={{ width: 140, height: 16 }}></div></td>
      <td><div className="skeleton" style={{ width: 60, height: 16 }}></div></td>
      <td><div className="skeleton" style={{ width: 70, height: 24 }}></div></td>
      <td><div className="skeleton" style={{ width: 80, height: 24 }}></div></td>
      <td><div className="skeleton" style={{ width: 70, height: 24 }}></div></td>
      <td><div className="skeleton" style={{ width: 100, height: 16 }}></div></td>
    </tr>
  );
}

function SkeletonTable() {
  return (
    <div className="card">
      <table className="data-table">
        <thead>
          <tr>
            <th>Payment ID</th>
            <th>Order</th>
            <th>Customer</th>
            <th>Amount</th>
            <th>Method</th>
            <th>Connector</th>
            <th>Status</th>
            <th>Date</th>
          </tr>
        </thead>
        <tbody>
          {[...Array(5)].map((_, i) => <SkeletonRow key={i} />)}
        </tbody>
      </table>
    </div>
  );
}

export default function Payments() {
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [payments, setPayments] = useState<Payment[]>([]);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const status = filter === 'all' ? '' : `&status=${encodeURIComponent(filter)}`;
        const res = await apiRequest<{ data: Payment[] }>(
          'GET',
          `/v1/payments?limit=100${status}`,
          undefined,
          undefined,
          { useApiKey: true },
        );
        setPayments(res.data || []);
      } finally {
        setLoading(false);
      }
    };
    void load();
  }, [filter]);

  const filtered = payments.filter(p => {
    if (
      search &&
      !p.id.includes(search) &&
      !p.order_id.includes(search) &&
      !(p.customer_email || '').includes(search)
    ) return false;
    return true;
  });

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Payments</h1>
          <p>View and manage all payment transactions</p>
        </div>
        <button className="btn btn-primary">+ Create Payment</button>
      </div>

      {/* Filters */}
      <div style={{ display: 'flex', gap: 16, marginBottom: 24, alignItems: 'center' }}>
        <div className="pill-tabs">
          {['all', 'captured', 'processing', 'failed', 'refunded'].map(f => (
            <button key={f} className={`pill-tab ${filter === f ? 'active' : ''}`}
              onClick={() => setFilter(f)}>
              {f === 'all' ? 'All' : f.charAt(0).toUpperCase() + f.slice(1)}
            </button>
          ))}
        </div>
        <input
          type="text" placeholder="Search by ID, order, or email..."
          value={search} onChange={e => setSearch(e.target.value)}
          style={{
            padding: '8px 16px', borderRadius: 8, border: '1px solid var(--fp-border)',
            background: 'var(--fp-bg-elevated)', color: 'var(--fp-text)',
            fontSize: 13, width: 280, fontFamily: 'var(--fp-font)',
          }}
        />
      </div>

      {/* Table */}
      {loading ? <SkeletonTable /> : (
        <div className="card">
          <table className="data-table">
            <thead>
              <tr>
                <th>Payment ID</th>
                <th>Order ID</th>
                <th>Customer</th>
                <th>Amount</th>
                <th>Method</th>
                <th>Connector</th>
                <th>Status</th>
                <th>Date</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((p) => (
                <tr key={p.id}>
                  <td style={{ fontFamily: 'monospace', fontSize: 13, color: 'var(--fp-primary-soft)' }}>{p.id}</td>
                  <td style={{ fontWeight: 500 }}>{p.order_id}</td>
                  <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>{p.customer_email || '-'}</td>
                  <td style={{ fontWeight: 600 }}>₹{p.amount.toLocaleString('en-IN')}</td>
                  <td>
                    <span style={{
                      padding: '3px 10px', borderRadius: 6, fontSize: 12,
                      background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)',
                    }}>{p.payment_method || '-'}</span>
                  </td>
                  <td>
                    <span style={{
                      padding: '3px 10px', borderRadius: 6, fontSize: 11, fontWeight: 600,
                      textTransform: 'uppercase', letterSpacing: '0.5px',
                      background: p.connector === 'simulator' ? 'rgba(6,214,160,0.1)' : p.connector === 'core_card_rail' ? 'rgba(59,130,246,0.1)' : 'rgba(245,158,11,0.1)',
                      color: p.connector === 'simulator' ? 'var(--fp-accent)' : p.connector === 'core_card_rail' ? 'var(--fp-info)' : 'var(--fp-warning)',
                    }}>{p.connector || '-'}</span>
                  </td>
                  <td><span className={`badge ${p.status}`}>{p.status}</span></td>
                  <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>
                    {new Date(p.created_at).toLocaleDateString('en-IN', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {filtered.length === 0 && (
            <div style={{ textAlign: 'center', padding: 48, color: 'var(--fp-text-muted)' }}>
              No payments found matching your criteria
            </div>
          )}
        </div>
      )}
    </div>
  );
}
