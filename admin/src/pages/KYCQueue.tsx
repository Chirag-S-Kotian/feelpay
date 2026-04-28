import { useState, useEffect } from 'react';
import { apiRequest } from '../hooks/useApi';
import { Clock, CheckCircle, XCircle, FileText } from 'lucide-react';

interface KYCSubmission {
  id: string;
  merchant_id: string;
  merchant_name: string;
  merchant_email: string;
  verification_type: string;
  document_number: string;
  status: string;
  submitted_at: string;
  document_url?: string;
}

interface KYCQueueResponse {
  data: KYCSubmission[];
  meta: {
    page: number;
    per_page: number;
    total: number;
  };
}

export default function KYCQueue() {
  const [submissions, setSubmissions] = useState<KYCSubmission[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedSubmission, setSelectedSubmission] = useState<KYCSubmission | null>(null);
  const [rejectionReason, setRejectionReason] = useState('');
  const [flaggedFields, setFlaggedFields] = useState<string[]>([]);

  useEffect(() => {
    fetchQueue();
  }, []);

  const fetchQueue = async () => {
    try {
      const data = await apiRequest<KYCQueueResponse>('GET', '/admin/kyc/queue');
      setSubmissions(data.data || []);
    } catch (err) {
      console.error('Failed to fetch KYC queue:', err);
    } finally {
      setLoading(false);
    }
  };

  const approveKYC = async (id: string) => {
    try {
      await apiRequest('POST', `/admin/kyc/${id}/approve`);
      fetchQueue();
      setSelectedSubmission(null);
    } catch (err) {
      console.error('Failed to approve KYC:', err);
    }
  };

  const rejectKYC = async (id: string) => {
    if (!rejectionReason) {
      alert('Please provide a rejection reason');
      return;
    }
    
    try {
      await apiRequest('POST', `/admin/kyc/${id}/reject`, {
        reason: rejectionReason,
        flagged_fields: flaggedFields,
      });
      fetchQueue();
      setSelectedSubmission(null);
      setRejectionReason('');
      setFlaggedFields([]);
    } catch (err) {
      console.error('Failed to reject KYC:', err);
    }
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
      pending: 'badge warning',
      under_review: 'badge processing',
      approved: 'badge success',
      rejected: 'badge failed',
      resubmit_required: 'badge refunded',
    };
    return classes[status] || 'badge';
  };

  const toggleFlaggedField = (field: string) => {
    setFlaggedFields((prev) =>
      prev.includes(field) ? prev.filter((f) => f !== field) : [...prev, field]
    );
  };

  const fieldOptions = [
    'pan_number',
    'pan_name',
    'gstin',
    'business_name',
    'business_address',
    'bank_account',
    'ifsc_code',
    'cancelled_cheque',
  ];

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>KYC Queue</h1>
          <p>Review and process merchant KYC submissions</p>
        </div>
        <div className="stat-card" style={{ padding: '12px 16px' }}>
          <Clock size={18} style={{ marginRight: 8 }} />
          {submissions.filter((s) => s.status === 'pending').length} Pending
        </div>
      </div>

      {loading ? (
        <div className="card">Loading...</div>
      ) : submissions.length === 0 ? (
        <div className="card" style={{ textAlign: 'center', padding: 48 }}>
          <CheckCircle size={48} style={{ color: 'var(--fp-success)', marginBottom: 16 }} />
          <h3 style={{ marginBottom: 8 }}>All Caught Up!</h3>
          <p style={{ color: 'var(--fp-text-muted)' }}>
            No pending KYC submissions to review
          </p>
        </div>
      ) : (
        <div className="table-card">
          <table>
            <thead>
              <tr>
                <th>Merchant</th>
                <th>Type</th>
                <th>Document</th>
                <th>Status</th>
                <th>Submitted</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {submissions.map((s) => (
                <tr key={s.id}>
                  <td>
                    <div>
                      <div style={{ fontWeight: 500 }}>{s.merchant_name}</div>
                      <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>
                        {s.merchant_email}
                      </div>
                    </div>
                  </td>
                  <td>
                    <span className="badge">{s.verification_type.toUpperCase()}</span>
                  </td>
                  <td style={{ fontFamily: 'monospace' }}>{s.document_number}</td>
                  <td>
                    <span className={getStatusBadge(s.status)}>{s.status}</span>
                  </td>
                  <td style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>
                    {formatDate(s.submitted_at)}
                  </td>
                  <td>
                    <div style={{ display: 'flex', gap: 8 }}>
                      <button
                        className="btn btn-sm btn-secondary"
                        onClick={() => setSelectedSubmission(s)}
                      >
                        <FileText size={14} /> Review
                      </button>
                      {s.status === 'pending' && (
                        <>
                          <button
                            className="btn btn-sm btn-primary"
                            onClick={() => approveKYC(s.id)}
                          >
                            <CheckCircle size={14} />
                          </button>
                          <button
                            className="btn btn-sm btn-danger"
                            onClick={() => setSelectedSubmission(s)}
                          >
                            <XCircle size={14} />
                          </button>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Review Modal */}
      {selectedSubmission && (
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
          onClick={() => setSelectedSubmission(null)}
        >
          <div
            className="card"
            style={{ width: 500, maxHeight: '80vh', overflow: 'auto' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ fontSize: 18, fontWeight: 600, marginBottom: 20 }}>
              Review KYC Submission
            </h3>

            <div style={{ marginBottom: 20 }}>
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Merchant</label>
                <div style={{ fontWeight: 500 }}>{selectedSubmission.merchant_name}</div>
              </div>
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Verification Type</label>
                <div>{selectedSubmission.verification_type.toUpperCase()}</div>
              </div>
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>Document Number</label>
                <div style={{ fontFamily: 'monospace' }}>{selectedSubmission.document_number}</div>
              </div>
            </div>

            {selectedSubmission.status === 'pending' && (
              <>
                <div style={{ marginBottom: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 8, display: 'block' }}>
                    Flag Fields for Resubmission (optional)
                  </label>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                    {fieldOptions.map((field) => (
                      <label
                        key={field}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: 4,
                          padding: '4px 8px',
                          border: '1px solid var(--fp-border)',
                          borderRadius: 4,
                          cursor: 'pointer',
                          background: flaggedFields.includes(field) ? 'var(--fp-primary)' : 'transparent',
                          color: flaggedFields.includes(field) ? 'white' : 'inherit',
                          fontSize: 12,
                        }}
                      >
                        <input
                          type="checkbox"
                          checked={flaggedFields.includes(field)}
                          onChange={() => toggleFlaggedField(field)}
                          style={{ display: 'none' }}
                        />
                        {field.replace(/_/g, ' ')}
                      </label>
                    ))}
                  </div>
                </div>

                <div style={{ marginBottom: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 8, display: 'block' }}>
                    Rejection Reason (required if rejecting)
                  </label>
                  <textarea
                    className="form-input"
                    rows={3}
                    value={rejectionReason}
                    onChange={(e) => setRejectionReason(e.target.value)}
                    placeholder="Enter reason for rejection..."
                  />
                </div>
              </>
            )}

            <div style={{ display: 'flex', gap: 12 }}>
              <button
                className="btn btn-secondary"
                style={{ flex: 1 }}
                onClick={() => setSelectedSubmission(null)}
              >
                Cancel
              </button>
              {selectedSubmission.status === 'pending' && (
                <>
                  <button
                    className="btn btn-danger"
                    style={{ flex: 1 }}
                    onClick={() => rejectKYC(selectedSubmission.id)}
                  >
                    <XCircle size={16} /> Reject
                  </button>
                  <button
                    className="btn btn-primary"
                    style={{ flex: 1 }}
                    onClick={() => approveKYC(selectedSubmission.id)}
                  >
                    <CheckCircle size={16} /> Approve
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
