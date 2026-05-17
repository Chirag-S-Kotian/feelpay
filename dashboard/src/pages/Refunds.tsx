import { useEffect, useState } from 'react';
import { RotateCcw, Search } from 'lucide-react';
import { SideDrawer } from '../components/SideDrawer';
import { apiRequest, MerchantAuthError } from '../hooks/useApi';

type Refund = {
  id: string;
  payment_order_id: string;
  order_id: string;
  amount: number;
  status: string;
  reason?: string | null;
  gateway_reference?: string | null;
  created_at: string;
  processed_at?: string | null;
  customer_email?: string | null;
  payment_status: string;
  refunded_amount: number;
  payment_amount: number;
};

function formatPaise(amount: number) {
  const paise = Math.trunc(amount);
  const sign = paise < 0 ? '-' : '';
  const absolute = Math.abs(paise);
  return `${sign}${Math.trunc(absolute / 100).toLocaleString('en-IN')}.${String(absolute % 100).padStart(2, '0')}`;
}

function statusTone(status: string) {
  const normalized = status.toLowerCase();
  if (['succeeded', 'processed', 'success'].includes(normalized)) return 'captured';
  if (['failed', 'cancelled'].includes(normalized)) return 'failed';
  if (['requested', 'processing', 'created'].includes(normalized)) return 'processing';
  return 'pending';
}

function SkeletonTable() {
  return (
    <div className="card refunds-table-card">
      <table className="data-table">
        <tbody>
          {[...Array(5)].map((_, i) => (
            <tr key={i}>
              <td><div className="skeleton" style={{ width: 120, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 100, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 160, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 80, height: 16 }} /></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function Refunds() {
  const [refunds, setRefunds] = useState<Refund[]>([]);
  const [selectedRefund, setSelectedRefund] = useState<Refund | null>(null);
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const loadRefunds = async () => {
      setLoading(true);
      setError('');
      try {
        const params = new URLSearchParams({ limit: '100' });
        if (filter !== 'all') params.set('status', filter);
        if (search.trim()) params.set('search', search.trim());
        const response = await apiRequest<{ refunds?: Refund[] }>(
          'GET',
          `/v1/refunds?${params.toString()}`,
          undefined,
          undefined,
          { useApiKey: true, cacheMs: 0 },
        );
        setRefunds(response.refunds || []);
      } catch (err) {
        if (err instanceof MerchantAuthError) return;
        setError(err instanceof Error ? err.message : 'Unable to load refunds');
      } finally {
        setLoading(false);
      }
    };
    void loadRefunds();
  }, [filter, search]);

  return (
    <>
      <SideDrawer
        isOpen={Boolean(selectedRefund)}
        onClose={() => setSelectedRefund(null)}
        title="Refund Details"
        subtitle={selectedRefund?.id}
      >
        {selectedRefund && (
          <div className="detail-section">
            <div className="detail-grid">
              <div className="detail-item"><label>Amount</label><span>₹{formatPaise(selectedRefund.amount)}</span></div>
              <div className="detail-item"><label>Status</label><span className={`badge ${statusTone(selectedRefund.status)}`}>{selectedRefund.status}</span></div>
              <div className="detail-item"><label>Order</label><span className="mono">{selectedRefund.order_id}</span></div>
              <div className="detail-item"><label>Payment</label><span className="mono">{selectedRefund.payment_order_id}</span></div>
              <div className="detail-item"><label>Customer</label><span>{selectedRefund.customer_email || 'Not provided'}</span></div>
              <div className="detail-item"><label>Gateway Reference</label><span>{selectedRefund.gateway_reference || 'Not assigned'}</span></div>
              <div className="detail-item"><label>Refunded Total</label><span>₹{formatPaise(selectedRefund.refunded_amount)} / ₹{formatPaise(selectedRefund.payment_amount)}</span></div>
              <div className="detail-item"><label>Processed</label><span>{selectedRefund.processed_at ? new Date(selectedRefund.processed_at).toLocaleString('en-IN') : 'Pending'}</span></div>
            </div>
            <div className="detail-item" style={{ marginTop: 16 }}>
              <label>Reason</label>
              <span>{selectedRefund.reason || 'No reason recorded'}</span>
            </div>
          </div>
        )}
      </SideDrawer>

      <div className="fade-in">
        <div className="page-header">
          <div>
            <h1>Refunds</h1>
            <p>Track refund lifecycle, payment mapping, and customer context</p>
          </div>
        </div>

        <div className="refunds-toolbar">
          <div className="pill-tabs">
            {['all', 'requested', 'processing', 'processed', 'succeeded', 'failed', 'cancelled'].map((item) => (
              <button
                key={item}
                type="button"
                className={`pill-tab ${filter === item ? 'active' : ''}`}
                onClick={() => setFilter(item)}
              >
                {item === 'all' ? 'All' : item.replace('_', ' ')}
              </button>
            ))}
          </div>
          <label className="search-box">
            <Search size={16} aria-hidden={true} />
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search refund, order, or email"
            />
          </label>
        </div>

        {error && <div className="error-banner" role="alert">{error}</div>}
        {loading ? <SkeletonTable /> : (
          <div className="card refunds-table-card">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Refund</th>
                  <th>Order</th>
                  <th>Customer</th>
                  <th>Amount</th>
                  <th>Status</th>
                  <th>Payment Status</th>
                  <th>Created At</th>
                </tr>
              </thead>
              <tbody>
                {refunds.map((refund) => (
                  <tr
                    key={refund.id}
                    className="clickable-row"
                    onClick={() => setSelectedRefund(refund)}
                  >
                    <td className="mono">{refund.id.slice(0, 12)}...</td>
                    <td className="mono">{refund.order_id}</td>
                    <td>{refund.customer_email || 'N/A'}</td>
                    <td>{formatPaise(refund.amount)}</td>
                    <td>
                      <span className={`badge badge-${refund.status}`}>
                        {refund.status}
                      </span>
                    </td>
                    <td>
                      <span className={`badge badge-${refund.payment_status}`}>
                        {refund.payment_status}
                      </span>
                    </td>
                    <td>{new Date(refund.created_at).toLocaleDateString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {refunds.length === 0 && (
              <div className="empty-state">
                <RotateCcw size={28} aria-hidden={true} />
                <strong>No refunds found</strong>
                <p>Refunds created from payment details will appear here with their payment mapping.</p>
              </div>
            )}
          </div>
        )}
      </div>

      <style>{`
        .refunds-toolbar {
          display: flex;
          align-items: center;
          gap: 16px;
          margin-bottom: 24px;
          flex-wrap: wrap;
        }
        .refunds-table-card {
          overflow-x: auto;
          -webkit-overflow-scrolling: touch;
        }
        @media (max-width: 760px) {
          .refunds-toolbar,
          .refunds-toolbar .pill-tabs,
          .search-box {
            width: 100%;
          }
        }
      `}</style>
    </>
  );
}
