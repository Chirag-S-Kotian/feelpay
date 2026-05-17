import { useState, useEffect } from 'react';
import { Search } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { SideDrawer } from '../components/SideDrawer';
import { formatPaise, parseInrToPaise } from '../utils/money';

type Payment = {
  id: string;
  order_id: string;
  amount: number;
  payment_method?: string | null;
  connector?: string | null;
  status: string;
  customer_email?: string | null;
  refunded_amount?: number | null;
  created_at: string;
};

function paiseInputValue(amount: number) {
  const absolute = Math.max(0, Math.trunc(amount));
  return `${Math.trunc(absolute / 100)}.${String(absolute % 100).padStart(2, '0')}`;
}

function percentageOfPaise(amount: number, basisPoints: number) {
  return Math.trunc((Math.trunc(amount) * basisPoints) / 10_000);
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
  };
  return labels[connector] || connector.replace(/_/g, ' ');
}

function connectorTone(connector?: string | null) {
  if (!connector) return 'unknown';
  if (connector === 'simulator') return 'test';
  if (connector.includes('card')) return 'card';
  if (connector.includes('upi')) return 'upi';
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
  const [selectedPayment, setSelectedPayment] = useState<Payment | null>(null);
  const [timeline, setTimeline] = useState<any[]>([]);
  const [webhooks, setWebhooks] = useState<any[]>([]);
  const [timelineLoading, setTimelineLoading] = useState(false);
  const [refundAmount, setRefundAmount] = useState('');
  const [refundReason, setRefundReason] = useState('');
  const [refundError, setRefundError] = useState('');
  const [refundNotice, setRefundNotice] = useState('');
  const [refundSubmitting, setRefundSubmitting] = useState(false);

  const refundableAmount = selectedPayment
    ? Math.max(0, Math.trunc(selectedPayment.amount) - Math.trunc(selectedPayment.refunded_amount || 0))
    : 0;
  const refundAllowed = Boolean(
    selectedPayment &&
    refundableAmount > 0 &&
    ['captured', 'settled', 'partially_refunded'].includes(selectedPayment.status.toLowerCase()),
  );

  useEffect(() => {
    if (!selectedPayment) return;
    const loadDetails = async () => {
      setTimelineLoading(true);
      try {
        const [detailRes, timelineRes] = await Promise.all([
          apiRequest<{ payment: any; webhooks: any[] }>('GET', `/v1/payments/${selectedPayment.id}/details`, undefined, undefined, { useApiKey: true }),
          apiRequest<any[]>('GET', `/v1/payments/${selectedPayment.id}/timeline`, undefined, undefined, { useApiKey: true }),
        ]);
        setTimeline(timelineRes || []);
        setWebhooks(detailRes.webhooks || []);
        if (detailRes.payment) {
          setSelectedPayment((current) =>
            current?.id === selectedPayment.id ? { ...current, ...detailRes.payment } : current,
          );
        }
      } catch (e) {
        console.error('Failed to load payment details', e);
      } finally {
        setTimelineLoading(false);
      }
    };
    void loadDetails();
  }, [selectedPayment?.id]);

  useEffect(() => {
    if (!selectedPayment) return;
    setRefundAmount(paiseInputValue(Math.max(0, selectedPayment.amount - (selectedPayment.refunded_amount || 0))));
    setRefundReason('');
    setRefundError('');
    setRefundNotice('');
  }, [selectedPayment?.id]);

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

  const submitRefund = async () => {
    if (!selectedPayment) return;
    const amountPaise = parseInrToPaise(refundAmount);
    if (!amountPaise || amountPaise <= 0) {
      setRefundError('Enter a valid refund amount.');
      return;
    }
    if (amountPaise > refundableAmount) {
      setRefundError('Refund amount exceeds the remaining refundable balance.');
      return;
    }
    if (!refundReason.trim()) {
      setRefundError('Refund reason is required.');
      return;
    }
    setRefundSubmitting(true);
    setRefundError('');
    setRefundNotice('');
    try {
      const response = await apiRequest<{
        status: string;
        refunded_amount: number;
        total_refunded: number;
      }>(
        'POST',
        `/v1/payments/${selectedPayment.id}/refund`,
        { amount: amountPaise, reason: refundReason.trim() },
        undefined,
        { useApiKey: true, idempotencyKey: `refund-${selectedPayment.id}-${crypto.randomUUID()}` },
      );
      const updatedPayment = {
        ...selectedPayment,
        status: response.status,
        refunded_amount: response.total_refunded,
      };
      setSelectedPayment(updatedPayment);
      setPayments((items) => items.map((item) => (item.id === selectedPayment.id ? updatedPayment : item)));
      setRefundNotice(`Refund submitted for ${formatPaise(response.refunded_amount)}.`);
      setRefundReason('');
      setRefundAmount(paiseInputValue(Math.max(0, selectedPayment.amount - response.total_refunded)));
    } catch (err) {
      setRefundError(err instanceof Error ? err.message : 'Failed to submit refund.');
    } finally {
      setRefundSubmitting(false);
    }
  };

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
    <>
      {/* Detail Drawer */}
      <SideDrawer
        isOpen={!!selectedPayment}
        onClose={() => setSelectedPayment(null)}
        title="Payment Details"
        subtitle={selectedPayment?.id}
      >
        {selectedPayment && (
          <>
            <div className="detail-section">
              <div style={{ display: 'flex', gap: 12, marginBottom: 24 }}>
                <div style={{ flex: 1, padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12, border: '1px solid var(--fp-border)' }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 4 }}>Amount</label>
                  <div style={{ fontSize: 20, fontWeight: 700 }}>{formatPaise(selectedPayment.amount)}</div>
                </div>
                <div style={{ flex: 1, padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12, border: '1px solid var(--fp-border)' }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 4 }}>Status</label>
                  <span className={`badge ${statusTone(selectedPayment.status)}`}>{selectedPayment.status}</span>
                </div>
              </div>

              <div className="detail-grid">
                <div className="detail-item">
                  <label>Order ID</label>
                  <span className="mono">{selectedPayment.order_id}</span>
                </div>
                <div className="detail-item">
                  <label>Customer</label>
                  <span>{selectedPayment.customer_email || 'No email'}</span>
                </div>
                <div className="detail-item">
                  <label>Method</label>
                  <span className="payment-chip">{selectedPayment.payment_method || 'Unknown'}</span>
                </div>
                <div className="detail-item">
                  <label>Connector</label>
                  <span className={`connector-chip ${connectorTone(selectedPayment.connector)}`}>
                    {connectorLabel(selectedPayment.connector)}
                  </span>
                </div>
              </div>
            </div>

            <div className="detail-section">
              <h4>Lifecycle Timeline</h4>
              {timelineLoading ? <div className="skeleton" style={{ height: 100 }} /> : (
                <div className="timeline">
                  {timeline.map((event, i) => (
                    <div key={i} className="timeline-item">
                      <div className="timeline-dot" style={{ borderColor: event.status_after === 'captured' ? 'var(--fp-success)' : 'var(--fp-primary)' }} />
                      <div className="timeline-content">
                        <h5>{event.event_type.replace(/_/g, ' ')}</h5>
                        <p>{new Date(event.created_at).toLocaleString()}</p>
                        {event.status_after && <p style={{ fontSize: 11, color: 'var(--fp-primary-soft)' }}>Status changed to {event.status_after}</p>}
                      </div>
                    </div>
                  ))}
                  {timeline.length === 0 && <p style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>No events recorded for this payment.</p>}
                </div>
              )}
            </div>

            <div className="detail-section">
              <h4>Fee Breakdown</h4>
              <div className="fee-card" style={{ padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12 }}>
                <div className="fee-row">
                  <span>Gross Amount</span>
                  <span>{formatPaise(selectedPayment.amount)}</span>
                </div>
                <div className="fee-row">
                  <span>FeelPay Fee (2%)</span>
                  <span className="warning">- {formatPaise(percentageOfPaise(selectedPayment.amount, 200))}</span>
                </div>
                <div className="fee-row">
                  <span>GST (18% of fee)</span>
                  <span className="warning">- {formatPaise(percentageOfPaise(percentageOfPaise(selectedPayment.amount, 200), 1800))}</span>
                </div>
                <div className="fee-row total">
                  <span>Net Settlement</span>
                  <span className="success">{formatPaise(selectedPayment.amount - percentageOfPaise(selectedPayment.amount, 200) - percentageOfPaise(percentageOfPaise(selectedPayment.amount, 200), 1800))}</span>
                </div>
              </div>
            </div>

            <div className="detail-section">
              <h4>Refund</h4>
              <div className="refund-panel">
                <div className="fee-row">
                  <span>Refundable Balance</span>
                  <span>{formatPaise(refundableAmount)}</span>
                </div>
                {refundNotice && <div className="success-banner">{refundNotice}</div>}
                {refundError && <div className="error-banner">{refundError}</div>}
                <label className="refund-field">
                  <span>Amount (INR)</span>
                  <input
                    value={refundAmount}
                    onChange={(event) => setRefundAmount(event.target.value)}
                    inputMode="decimal"
                    disabled={!refundAllowed || refundSubmitting}
                  />
                </label>
                <label className="refund-field">
                  <span>Reason</span>
                  <textarea
                    value={refundReason}
                    onChange={(event) => setRefundReason(event.target.value)}
                    rows={3}
                    disabled={!refundAllowed || refundSubmitting}
                    placeholder="Required for audit and support follow-up"
                  />
                </label>
                <div className="refund-actions">
                  <button
                    className="btn-secondary"
                    type="button"
                    onClick={() => setRefundAmount(paiseInputValue(refundableAmount))}
                    disabled={!refundAllowed || refundSubmitting}
                  >
                    Full Refund
                  </button>
                  <button
                    className="btn-primary"
                    type="button"
                    onClick={() => void submitRefund()}
                    disabled={!refundAllowed || refundSubmitting}
                  >
                    {refundSubmitting ? 'Submitting...' : 'Submit Refund'}
                  </button>
                </div>
                {!refundAllowed && (
                  <p className="muted">Refunds are available only for captured or partially refunded payments with remaining balance.</p>
                )}
              </div>
            </div>

            {webhooks.length > 0 && (
              <div className="detail-section">
                <h4>Webhook Deliveries</h4>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {webhooks.map((w, i) => (
                    <div key={i} style={{ padding: 12, background: 'var(--fp-bg-elevated)', borderRadius: 8, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div>
                        <div style={{ fontSize: 13, fontWeight: 500 }}>{w.event_type}</div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted)' }}>{new Date(w.created_at).toLocaleString()}</div>
                      </div>
                      <span className={`badge ${w.status === 'success' ? 'captured' : 'failed'}`} style={{ fontSize: 10 }}>{w.status}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </SideDrawer>

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
          <label className="search-box">
            <Search size={16} aria-hidden={true} />
            <input
              type="text"
              placeholder="Search by ID, order, or email..."
              value={search}
              onChange={e => setSearch(e.target.value)}
            />
          </label>
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
                  <tr key={p.id} onClick={() => setSelectedPayment(p)} style={{ cursor: 'pointer' }}>
                    <td style={{ fontFamily: 'monospace', fontSize: 13, color: 'var(--fp-primary-soft)' }}>{p.id}</td>
                    <td style={{ fontWeight: 500 }}>{p.order_id}</td>
                    <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>{p.customer_email || '-'}</td>
                    <td style={{ fontWeight: 600 }}>{formatPaise(p.amount)}</td>
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

        .refund-panel {
          display: grid;
          gap: 12px;
          padding: 16px;
          background: var(--fp-bg-elevated);
          border: 1px solid var(--fp-border);
          border-radius: 12px;
        }

        .refund-field {
          display: grid;
          gap: 6px;
          color: var(--fp-text-muted);
          font-size: 12px;
          font-weight: 600;
        }

        .refund-field input,
        .refund-field textarea {
          width: 100%;
          padding: 10px 12px;
          border-radius: 8px;
          border: 1px solid var(--fp-border);
          background: var(--fp-bg);
          color: var(--fp-text);
          font: inherit;
        }

        .refund-actions {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }

        @media (max-width: 760px) {
          .payments-toolbar,
          .payments-toolbar .pill-tabs,
          .search-box {
            width: 100% !important;
          }

          .payments-toolbar .pill-tabs {
            overflow-x: auto;
            padding-bottom: 2px;
          }
        }
      `}</style>
    </div>
    </>
  );
}
