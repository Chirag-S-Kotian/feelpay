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

function paymentAmount(amount: number) {
  return (amount / 100).toLocaleString('en-IN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

function statusTone(status: string) {
  const normalized = status.toLowerCase();
  if (['captured', 'completed', 'success'].includes(normalized)) return 'captured';
  if (['failed', 'declined', 'error'].includes(normalized)) return 'failed';
  if (['refunded', 'partially_refunded'].includes(normalized)) return 'refunded';
  if (['processing', 'requires_action', 'created', 'pending'].includes(normalized)) return 'processing';
  return 'pending';
}

function connectorLabel(connector?: string | null) {
  if (!connector) return 'Not set';
  const labels: Record<string, string> = {
    simulator: 'Simulator',
    core_card_rail: 'Card rail',
    upi_stub: 'UPI rail',
    razorpay: 'Razorpay',
    stripe: 'Stripe',
  };
  return labels[connector] || connector.replace(/_/g, ' ');
}

function connectorTone(connector?: string | null) {
  if (!connector) return 'unknown';
  if (connector === 'simulator') return 'test';
  if (connector.includes('card') || connector === 'stripe') return 'card';
  if (connector.includes('upi') || connector === 'razorpay') return 'upi';
  return 'live';
}

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
        <div className="payments-toolbar">
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
        <div className="card payments-table-card">
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
                  <td style={{ fontWeight: 600 }}>₹{paymentAmount(p.amount)}</td>
                  <td>
                    <span className="payment-chip">{p.payment_method || 'Not set'}</span>
                  </td>
                  <td>
                    <span className={`connector-chip ${connectorTone(p.connector)}`}>
                      {connectorLabel(p.connector)}
                    </span>
                  </td>
                  <td><span className={`badge ${statusTone(p.status)}`}>{p.status}</span></td>
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
      <style>{`
        .payments-toolbar {
          display: flex;
          gap: 16px;
          margin-bottom: 24px;
          align-items: center;
          flex-wrap: wrap;
        }

        .payments-table-card {
          overflow-x: auto;
          -webkit-overflow-scrolling: touch;
        }

        .payment-chip,
        .connector-chip {
          display: inline-flex;
          align-items: center;
          min-height: 26px;
          padding: 4px 10px;
          border-radius: 999px;
          border: 1px solid var(--fp-border);
          background: var(--fp-bg-elevated);
          color: var(--fp-text-muted);
          font-size: 12px;
          font-weight: 600;
          text-transform: capitalize;
          white-space: nowrap;
        }

        .connector-chip.test {
          background: rgba(34, 197, 94, 0.14);
          border-color: rgba(34, 197, 94, 0.3);
          color: #86efac;
        }

        .connector-chip.card {
          background: rgba(59, 130, 246, 0.14);
          border-color: rgba(59, 130, 246, 0.3);
          color: #93c5fd;
        }

        .connector-chip.upi {
          background: rgba(245, 158, 11, 0.14);
          border-color: rgba(245, 158, 11, 0.3);
          color: #fbbf24;
        }

        .connector-chip.live {
          background: rgba(139, 92, 246, 0.14);
          border-color: rgba(139, 92, 246, 0.3);
          color: #c4b5fd;
        }

        @media (max-width: 760px) {
          .payments-toolbar,
          .payments-toolbar .pill-tabs,
          .payments-toolbar input {
            width: 100% !important;
          }

          .payments-toolbar .pill-tabs {
            overflow-x: auto;
            padding-bottom: 2px;
          }
        }
      `}</style>
    </div>
  );
}
