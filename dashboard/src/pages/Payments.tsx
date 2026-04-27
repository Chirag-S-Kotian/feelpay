import { useState, useEffect } from 'react';

const ALL_PAYMENTS = [
  { id: 'fp_8a3b2c1d', order: 'ORD-2024-1001', amount: 2499, method: 'UPI', connector: 'simulator', status: 'captured', email: 'rahul@example.com', created: '2024-12-15T10:30:00Z' },
  { id: 'fp_7e4f5a6b', order: 'ORD-2024-1002', amount: 15999, method: 'Card', connector: 'core_card_rail', status: 'captured', email: 'priya@example.com', created: '2024-12-15T10:22:00Z' },
  { id: 'fp_6d5c4b3a', order: 'ORD-2024-1003', amount: 499, method: 'UPI', connector: 'simulator', status: 'failed', email: 'amit@example.com', created: '2024-12-15T10:18:00Z' },
  { id: 'fp_5c4d3e2f', order: 'ORD-2024-1004', amount: 3299, method: 'Net Banking', connector: 'core_upi_rail', status: 'processing', email: 'sneha@example.com', created: '2024-12-15T10:15:00Z' },
  { id: 'fp_4b3c2d1e', order: 'ORD-2024-1005', amount: 899, method: 'Wallet', connector: 'simulator', status: 'captured', email: 'vikram@example.com', created: '2024-12-15T09:58:00Z' },
  { id: 'fp_3a2b1c0d', order: 'ORD-2024-1006', amount: 7499, method: 'Card', connector: 'core_card_rail', status: 'refunded', email: 'deepa@example.com', created: '2024-12-15T09:45:00Z' },
  { id: 'fp_2b1c0d9e', order: 'ORD-2024-1007', amount: 1599, method: 'UPI', connector: 'simulator', status: 'captured', email: 'karan@example.com', created: '2024-12-15T09:30:00Z' },
  { id: 'fp_1a0b9c8d', order: 'ORD-2024-1008', amount: 4999, method: 'Card', connector: 'core_card_rail', status: 'captured', email: 'meera@example.com', created: '2024-12-15T09:15:00Z' },
];

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

  useEffect(() => {
    // Simulate API call
    const timer = setTimeout(() => setLoading(false), 800);
    return () => clearTimeout(timer);
  }, []);

  const filtered = ALL_PAYMENTS.filter(p => {
    if (filter !== 'all' && p.status !== filter) return false;
    if (search && !p.id.includes(search) && !p.order.includes(search) && !p.email.includes(search)) return false;
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
              {filtered.map(p => (
                <tr key={p.id}>
                  <td style={{ fontFamily: 'monospace', fontSize: 13, color: 'var(--fp-primary-soft)' }}>{p.id}</td>
                  <td style={{ fontWeight: 500 }}>{p.order}</td>
                  <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>{p.email}</td>
                  <td style={{ fontWeight: 600 }}>₹{p.amount.toLocaleString('en-IN')}</td>
                  <td>
                    <span style={{
                      padding: '3px 10px', borderRadius: 6, fontSize: 12,
                      background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)',
                    }}>{p.method}</span>
                  </td>
                  <td>
                    <span style={{
                      padding: '3px 10px', borderRadius: 6, fontSize: 11, fontWeight: 600,
                      textTransform: 'uppercase', letterSpacing: '0.5px',
                      background: p.connector === 'simulator' ? 'rgba(6,214,160,0.1)' : p.connector === 'core_card_rail' ? 'rgba(59,130,246,0.1)' : 'rgba(245,158,11,0.1)',
                      color: p.connector === 'simulator' ? 'var(--fp-accent)' : p.connector === 'core_card_rail' ? 'var(--fp-info)' : 'var(--fp-warning)',
                    }}>{p.connector}</span>
                  </td>
                  <td><span className={`badge ${p.status}`}>{p.status}</span></td>
                  <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>
                    {new Date(p.created).toLocaleDateString('en-IN', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })}
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
