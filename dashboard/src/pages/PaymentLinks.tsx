import { useState, useEffect } from 'react';
import { Link, Plus, Copy, ExternalLink, Trash2, Clock, DollarSign } from 'lucide-react';
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
  const [amount, setAmount] = useState('');
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
    if (!amount) return;
    
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
      fetchLinks();
    } catch (err) {
      console.error('Failed to create payment link:', err);
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

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
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
        <div className="card" style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Create Payment Link</h3>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 16 }}>
            <div>
              <label className="form-label">Amount (₹)</label>
              <input
                type="number"
                className="form-input"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="Enter amount"
              />
            </div>
            <div>
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
            <div style={{ gridColumn: 'span 2' }}>
              <label className="form-label">Description</label>
              <input
                type="text"
                className="form-input"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="What is this payment for?"
              />
            </div>
            <div>
              <label className="form-label">Customer Name (optional)</label>
              <input
                type="text"
                className="form-input"
                value={customerName}
                onChange={(e) => setCustomerName(e.target.value)}
                placeholder="Customer name"
              />
            </div>
            <div>
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
          <div style={{ display: 'flex', gap: 12, marginTop: 20 }}>
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
        <div className="card" style={{ textAlign: 'center', padding: 48 }}>
          <Link size={48} style={{ color: 'var(--fp-text-dim)', marginBottom: 16 }} />
          <h3 style={{ marginBottom: 8 }}>No Payment Links Yet</h3>
          <p style={{ color: 'var(--fp-text-muted)', marginBottom: 24 }}>
            Create your first payment link to start accepting payments
          </p>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Create Link
          </button>
        </div>
      ) : (
        <div className="card">
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
                  <td style={{ fontFamily: 'monospace', fontSize: 13 }}>
                    {link.link_id}
                  </td>
                  <td style={{ fontWeight: 600 }}>{formatCurrency(link.amount)}</td>
                  <td>{link.description || '-'}</td>
                  <td>
                    {link.customer_name || link.customer_email ? (
                      <div>
                        <div>{link.customer_name}</div>
                        <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>
                          {link.customer_email}
                        </div>
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
                        title="Copy link"
                        onClick={() => copyToClipboard(link.full_url)}
                      >
                        <Copy size={14} />
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
    </div>
  );
}