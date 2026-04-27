import { useState, useEffect } from 'react';
import { apiRequest } from '../hooks/useApi';
import { Search, Filter, Eye, Ban, CheckCircle, XCircle, ExternalLink } from 'lucide-react';

interface Merchant {
  id: string;
  name: string;
  email: string;
  business_name: string | null;
  business_type: string | null;
  kyc_status: string;
  onboarding_status: string;
  created_at: string;
  total_volume?: number;
  transaction_count?: number;
}

interface MerchantsResponse {
  data: Merchant[];
  meta: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export default function Merchants() {
  const [merchants, setMerchants] = useState<Merchant[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [selectedMerchant, setSelectedMerchant] = useState<Merchant | null>(null);

  useEffect(() => {
    fetchMerchants();
  }, [statusFilter]);

  const fetchMerchants = async () => {
    try {
      const params = new URLSearchParams();
      if (statusFilter !== 'all') {
        params.append('status', statusFilter);
      }
      
      const data = await apiRequest<MerchantsResponse>('GET', `/admin/merchants?${params.toString()}`);
      setMerchants(data.data || []);
    } catch (err) {
      console.error('Failed to fetch merchants:', err);
    } finally {
      setLoading(false);
    }
  };

  const updateMerchantStatus = async (id: string, action: 'approve' | 'reject' | 'suspend') => {
    try {
      await apiRequest('POST', `/admin/merchants/${id}/${action}`);
      fetchMerchants();
      setSelectedMerchant(null);
    } catch (err) {
      console.error(`Failed to ${action} merchant:`, err);
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-IN', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency: 'INR',
      maximumFractionDigits: 0,
    }).format(amount / 100);
  };

  const getStatusBadge = (status: string) => {
    const classes: Record<string, string> = {
      active: 'badge success',
      approved: 'badge success',
      pending: 'badge warning',
      rejected: 'badge failed',
      suspended: 'badge refunded',
      under_review: 'badge processing',
    };
    return classes[status] || 'badge';
  };

  const filteredMerchants = merchants.filter((m) => {
    if (!search) return true;
    const searchLower = search.toLowerCase();
    return (
      m.name.toLowerCase().includes(searchLower) ||
      m.email.toLowerCase().includes(searchLower) ||
      m.business_name?.toLowerCase().includes(searchLower) ||
      m.id.includes(search)
    );
  });

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Merchant Management</h1>
          <p>Review, approve, and manage all platform merchants</p>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 24 }}>
        <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
          <div style={{ position: 'relative', flex: 1 }}>
            <Search
              size={18}
              style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)', color: 'var(--fp-text-dim)' }}
            />
            <input
              type="text"
              className="form-input"
              style={{ paddingLeft: 40 }}
              placeholder="Search by name, email, or ID..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <select
            className="form-input"
            style={{ width: 200 }}
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
          >
            <option value="all">All Status</option>
            <option value="pending">Pending</option>
            <option value="under_review">Under Review</option>
            <option value="approved">Approved</option>
            <option value="rejected">Rejected</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>
      </div>

      {loading ? (
        <div className="card">Loading...</div>
      ) : (
        <div className="table-card">
          <table>
            <thead>
              <tr>
                <th>Merchant</th>
                <th>Email</th>
                <th>Business Type</th>
                <th>KYC Status</th>
                <th>Onboarding</th>
                <th>Volume</th>
                <th>Joined</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredMerchants.map((m) => (
                <tr key={m.id}>
                  <td>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                      <div
                        className="logo-icon"
                        style={{ width: 32, height: 32, fontSize: 12 }}
                      >
                        {(m.business_name || m.name).charAt(0)}
                      </div>
                      <div>
                        <div style={{ fontWeight: 500 }}>{m.business_name || m.name}</div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted)' }}>
                          {m.id.slice(0, 8)}...
                        </div>
                      </div>
                    </div>
                  </td>
                  <td>{m.email}</td>
                  <td>{m.business_type || 'N/A'}</td>
                  <td>
                    <span className={getStatusBadge(m.kyc_status)}>{m.kyc_status}</span>
                  </td>
                  <td>
                    <span className={getStatusBadge(m.onboarding_status)}>{m.onboarding_status}</span>
                  </td>
                  <td style={{ fontWeight: 600 }}>
                    {m.total_volume ? formatCurrency(m.total_volume) : '₹0'}
                  </td>
                  <td style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>
                    {formatDate(m.created_at)}
                  </td>
                  <td>
                    <div style={{ display: 'flex', gap: 8 }}>
                      <button
                        className="btn btn-sm btn-secondary"
                        title="View details"
                        onClick={() => setSelectedMerchant(m)}
                      >
                        <Eye size={14} />
                      </button>
                      {m.onboarding_status === 'pending' && (
                        <>
                          <button
                            className="btn btn-sm btn-primary"
                            title="Approve"
                            onClick={() => updateMerchantStatus(m.id, 'approve')}
                          >
                            <CheckCircle size={14} />
                          </button>
                          <button
                            className="btn btn-sm btn-danger"
                            title="Reject"
                            onClick={() => updateMerchantStatus(m.id, 'reject')}
                          >
                            <XCircle size={14} />
                          </button>
                        </>
                      )}
                      {m.onboarding_status === 'approved' && (
                        <button
                          className="btn btn-sm btn-danger"
                          title="Suspend"
                          onClick={() => updateMerchantStatus(m.id, 'suspend')}
                        >
                          <Ban size={14} />
                        </button>
                      )}
                      {m.onboarding_status === 'suspended' && (
                        <button
                          className="btn btn-sm btn-primary"
                          title="Reactivate"
                          onClick={() => updateMerchantStatus(m.id, 'approve')}
                        >
                          <CheckCircle size={14} />
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
              {filteredMerchants.length === 0 && (
                <tr>
                  <td colSpan={8} style={{ textAlign: 'center', padding: 40 }}>
                    No merchants found
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* Merchant Detail Modal */}
      {selectedMerchant && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setSelectedMerchant(null)}
        >
          <div
            className="card"
            style={{ width: 600, maxHeight: '80vh', overflow: 'auto' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 20 }}>
              {selectedMerchant.business_name || selectedMerchant.name}
            </h3>
            
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Merchant ID</label>
                <div style={{ fontFamily: 'monospace' }}>{selectedMerchant.id}</div>
              </div>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Email</label>
                <div>{selectedMerchant.email}</div>
              </div>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Business Type</label>
                <div>{selectedMerchant.business_type || 'N/A'}</div>
              </div>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>KYC Status</label>
                <div>
                  <span className={getStatusBadge(selectedMerchant.kyc_status)}>
                    {selectedMerchant.kyc_status}
                  </span>
                </div>
              </div>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Onboarding Status</label>
                <div>
                  <span className={getStatusBadge(selectedMerchant.onboarding_status)}>
                    {selectedMerchant.onboarding_status}
                  </span>
                </div>
              </div>
              <div>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Joined</label>
                <div>{formatDate(selectedMerchant.created_at)}</div>
              </div>
            </div>

            <div style={{ display: 'flex', gap: 12, marginTop: 24 }}>
              <button
                className="btn btn-secondary"
                style={{ flex: 1 }}
                onClick={() => setSelectedMerchant(null)}
              >
                Close
              </button>
              <button
                className="btn btn-primary"
                style={{ flex: 1 }}
                onClick={() => {
                  window.open(`/admin/merchants/${selectedMerchant.id}`, '_blank');
                }}
              >
                <ExternalLink size={16} /> Full Profile
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}