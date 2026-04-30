import { useState, useEffect } from 'react';
import { Link, Plus, Copy, ExternalLink, Trash2 } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface PaymentLink {
  id: string;
  link_id: string;
  amount: number;
  currency: string;
  description?: string;
  status: string;
  short_url?: string;
  full_url: string;
  expires_at?: string;
  created_at: string;
  customer_name?: string;
  customer_email?: string;
  view_count: number;
  attempt_count: number;
}

interface PaymentLinksResponse {
  data: PaymentLink[];
  meta: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export default function PaymentLinks() {
  const [links, setLinks] = useState<PaymentLink[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [amount, setAmount] = useState('');
  const [createError, setCreateError] = useState('');
  const [description, setDescription] = useState('');
  const [customerEmail, setCustomerEmail] = useState('');
  const [customerName, setCustomerName] = useState('');
  const [expiryHours, setExpiryHours] = useState('24');

  useEffect(() => {
    fetchLinks();
  }, []);

  const fetchLinks = async () => {
    try {
      const data = await apiRequest<PaymentLinksResponse>('GET', '/v1/payment-links');
      setLinks(data.data || []);
    } catch (err) {
      console.error('Failed to fetch payment links:', err);
    } finally {
      setLoading(false);
    }
  };

  const createLink = async () => {
    setCreateError('');
    if (!amount || Number(amount) <= 0) {
      setCreateError('Enter valid amount greater than 0');
      return;
    }
    
    try {
      const expiresAt = new Date();
      expiresAt.setHours(expiresAt.getHours() + parseInt(expiryHours));
      
      await apiRequest('POST', '/v1/payment-links', {
        amount: parseInt(amount) * 100, // Convert to paise
        description,
        customer_email: customerEmail || undefined,
        customer_name: customerName || undefined,
        expires_at: expiresAt.toISOString(),
      });
      
      setShowCreate(false);
      setAmount('');
      setDescription('');
      setCustomerEmail('');
      setCustomerName('');
      setCreateError('');
      fetchLinks();
    } catch (err) {
      console.error('Failed to create payment link:', err);
      setCreateError(err instanceof Error ? err.message : 'Failed to create payment link');
    }
  };

  const cancelLink = async (id: string) => {
    if (!confirm('Are you sure you want to cancel this payment link?')) return;
    
    try {
      await apiRequest('POST', `/v1/payment-links/${id}/cancel`);
      fetchLinks();
    } catch (err) {
      console.error('Failed to cancel payment link:', err);
    }
  };

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId((curr) => (curr === id ? null : curr)), 1200);
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency: 'INR',
    }).format(amount / 100);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-IN', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusBadge = (status: string) => {
    const classes: Record<string, string> = {
      active: 'badge success',
      paid: 'badge captured',
      expired: 'badge failed',
      cancelled: 'badge refunded',
    };
    return classes[status] || 'badge';
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Payment Links</h1>
          <p>Create and manage shareable payment links</p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
          <Plus size={18} /> Create Link
        </button>
      </div>

      {showCreate && (
        <div className="card payment-link-create-card">
          <div className="payment-link-create-header">
            <h3>Create Payment Link</h3>
            <p>Generate branded checkout URL for quick payment collection.</p>
          </div>
          {createError && (
            <div className="payment-link-create-error">{createError}</div>
          )}
          <div className="payment-link-form-grid">
            <div className="payment-link-field">
              <label className="form-label">Amount (₹)</label>
              <div className="payment-link-input-wrap">
                <span className="payment-link-input-prefix">₹</span>
                <input
                  type="number"
                  className="form-input payment-link-input-with-prefix"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="Enter amount"
                />
              </div>
            </div>
            <div className="payment-link-field">
              <label className="form-label">Expiry (hours)</label>
              <select
                className="form-input"
                value={expiryHours}
                onChange={(e) => setExpiryHours(e.target.value)}
              >
                <option value="1">1 hour</option>
                <option value="6">6 hours</option>
                <option value="12">12 hours</option>
                <option value="24">24 hours</option>
                <option value="48">48 hours</option>
                <option value="168">7 days</option>
              </select>
            </div>
            <div className="payment-link-field payment-link-span-2">
              <label className="form-label">Description</label>
              <input
                type="text"
                className="form-input"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="What is this payment for?"
              />
            </div>
            <div className="payment-link-field">
              <label className="form-label">Customer Name (optional)</label>
              <input
                type="text"
                className="form-input"
                value={customerName}
                onChange={(e) => setCustomerName(e.target.value)}
                placeholder="Customer name"
              />
            </div>
            <div className="payment-link-field">
              <label className="form-label">Customer Email (optional)</label>
              <input
                type="email"
                className="form-input"
                value={customerEmail}
                onChange={(e) => setCustomerEmail(e.target.value)}
                placeholder="customer@example.com"
              />
            </div>
          </div>
          <div className="payment-link-form-actions">
            <button className="btn btn-primary" onClick={createLink}>
              Create Link
            </button>
            <button className="btn btn-secondary" onClick={() => setShowCreate(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      {loading ? (
        <div className="card">Loading...</div>
      ) : links.length === 0 ? (
        <div className="card payment-link-empty-state">
          <div className="payment-link-empty-icon-wrap">
            <Link size={40} style={{ color: 'var(--fp-primary-soft)' }} />
          </div>
          <h3>No Payment Links Yet</h3>
          <p>
            Create your first payment link to start accepting payments
          </p>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Create Link
          </button>
        </div>
      ) : (
        <div className="card payment-link-list-card">
          <div className="payment-link-list-header">
            <h3>Active + Historical Links</h3>
            <div className="payment-link-list-stats">
              <span>{links.length} total</span>
              <span>{links.filter((l) => l.status === 'active').length} active</span>
            </div>
          </div>
          <table className="data-table">
            <thead>
              <tr>
                <th>Link ID</th>
                <th>Amount</th>
                <th>Description</th>
                <th>Customer</th>
                <th>Status</th>
                <th>Views / Attempts</th>
                <th>Created</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {links.map((link) => (
                <tr key={link.id}>
                  <td>
                    <div className="payment-link-id-cell">
                      <code>{link.link_id}</code>
                      {link.short_url && <small>{link.short_url}</small>}
                    </div>
                  </td>
                  <td>
                    <span className="payment-link-amount">{formatCurrency(link.amount)}</span>
                  </td>
                  <td>{link.description || '-'}</td>
                  <td>
                    {link.customer_name || link.customer_email ? (
                      <div className="payment-link-customer-cell">
                        <div>{link.customer_name}</div>
                        <small>{link.customer_email}</small>
                      </div>
                    ) : '-'}
                  </td>
                  <td>
                    <span className={getStatusBadge(link.status)}>{link.status}</span>
                  </td>
                  <td>
                    {link.view_count} / {link.attempt_count}
                  </td>
                  <td style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>
                    {formatDate(link.created_at)}
                  </td>
                  <td>
                    <div style={{ display: 'flex', gap: 8 }}>
                      <button
                        className="btn btn-sm btn-secondary"
                        title={copiedId === link.id ? 'Copied' : 'Copy link'}
                        onClick={() => copyToClipboard(link.full_url, link.id)}
                      >
                        {copiedId === link.id ? 'Copied' : <Copy size={14} />}
                      </button>
                      <a
                        href={link.full_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="btn btn-sm btn-secondary"
                        title="Open link"
                      >
                        <ExternalLink size={14} />
                      </a>
                      {link.status === 'active' && (
                        <button
                          className="btn btn-sm btn-danger"
                          title="Cancel link"
                          onClick={() => cancelLink(link.id)}
                        >
                          <Trash2 size={14} />
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <style>{`
        .payment-link-create-card {
          margin-bottom: 24px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .payment-link-create-header h3 {
          font-size: 18px;
          font-weight: 700;
          margin-bottom: 6px;
          letter-spacing: 0.01em;
        }

        .payment-link-create-header p {
          color: var(--fp-text-muted);
          font-size: 14px;
          margin-bottom: 18px;
        }

        .payment-link-form-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 16px;
        }

        .payment-link-create-error {
          margin: 0 0 14px;
          padding: 10px 12px;
          border-radius: 10px;
          font-size: 13px;
          border: 1px solid color-mix(in srgb, #ef4444 55%, transparent);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }

        .payment-link-field {
          display: flex;
          flex-direction: column;
          gap: 8px;
          min-width: 0;
        }

        .payment-link-field .form-label {
          display: block;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
          margin: 0;
          letter-spacing: 0.01em;
        }

        .payment-link-field .form-input {
          height: 42px;
          width: 100%;
          min-width: 0;
          padding: 0 12px;
          font-size: 14px;
          border-radius: 999px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text);
        }

        .payment-link-field .form-input:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .payment-link-field .form-input::placeholder {
          color: color-mix(in srgb, var(--fp-text-muted) 88%, #ffffff 12%);
        }

        .payment-link-input-wrap {
          position: relative;
        }

        .payment-link-input-prefix {
          position: absolute;
          left: 12px;
          top: 50%;
          transform: translateY(-50%);
          font-size: 13px;
          color: var(--fp-text-muted);
          pointer-events: none;
        }

        .payment-link-input-with-prefix {
          padding-left: 28px !important;
        }

        .payment-link-field select.form-input {
          appearance: none;
          background-image:
            linear-gradient(45deg, transparent 50%, #8a857d 50%),
            linear-gradient(135deg, #8a857d 50%, transparent 50%);
          background-position:
            calc(100% - 16px) calc(50% + 1px),
            calc(100% - 11px) calc(50% + 1px);
          background-size: 5px 5px, 5px 5px;
          background-repeat: no-repeat;
          padding-right: 30px;
        }

        .payment-link-span-2 {
          grid-column: span 2;
        }

        .payment-link-form-actions {
          display: flex;
          gap: 12px;
          margin-top: 20px;
        }

        .payment-link-empty-state {
          text-align: center;
          padding: 56px 20px;
          background: #fff9ec;
        }

        .payment-link-empty-icon-wrap {
          width: 74px;
          height: 74px;
          margin: 0 auto 16px;
          border-radius: 20px;
          display: grid;
          place-items: center;
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .payment-link-empty-state h3 {
          margin-bottom: 8px;
          font-size: 22px;
          letter-spacing: 0.01em;
        }

        .payment-link-empty-state p {
          color: var(--fp-text-muted);
          margin-bottom: 22px;
        }

        .payment-link-list-card {
          overflow: hidden;
        }

        .payment-link-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .payment-link-list-header h3 {
          margin: 0;
          font-size: 16px;
          letter-spacing: 0.01em;
        }

        .payment-link-list-stats {
          display: flex;
          gap: 8px;
        }

        .payment-link-list-stats span {
          padding: 5px 10px;
          border-radius: 999px;
          font-size: 12px;
          color: var(--fp-text-muted);
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .payment-link-id-cell code {
          display: block;
          font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
          font-size: 12px;
          color: var(--fp-primary-soft);
          margin-bottom: 2px;
        }

        .payment-link-id-cell small,
        .payment-link-customer-cell small {
          color: var(--fp-text-muted);
          font-size: 12px;
        }

        .payment-link-amount {
          font-weight: 700;
        }

        @media (max-width: 960px) {
          .payment-link-form-grid {
            grid-template-columns: 1fr;
          }

          .payment-link-span-2 {
            grid-column: span 1;
          }

          .payment-link-list-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 10px;
          }
        }
      `}</style>
    </div>
  );
}
