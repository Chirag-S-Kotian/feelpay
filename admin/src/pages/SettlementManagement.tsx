import { useState } from 'react';
import { apiRequest } from '../hooks/useApi';

export default function SettlementManagement() {
  const [id, setId] = useState('');
  const [utr, setUtr] = useState('');
  const [reason, setReason] = useState('');

  async function action(payload: unknown) {
    if (!id) return;
    await apiRequest('POST', `/v1/admin/settlements/${id}/actions`, payload);
  }

  return (
    <section className="page-shell">
      <header className="page-header">
        <div>
          <h1>Settlement Operations</h1>
          <p style={{ color: 'var(--text-muted, #64748b)', fontSize: '13px', margin: 0 }}>
            Manage manual settlement overrides, holds, and payout confirmations
          </p>
        </div>
      </header>

      <div className="card filter-grid">
        <label className="form-field">
          <span>Settlement ID</span>
          <input
            placeholder="set_..."
            value={id}
            onChange={event => setId(event.target.value)}
          />
        </label>
        <label className="form-field">
          <span>UTR Reference</span>
          <input
            placeholder="Bank UTR reference"
            value={utr}
            onChange={event => setUtr(event.target.value)}
          />
        </label>
        <label className="form-field">
          <span>Reason / Note</span>
          <input
            placeholder="Operational reason"
            value={reason}
            onChange={event => setReason(event.target.value)}
          />
        </label>
        <div className="filter-actions">
          <button className="btn-primary" type="button" onClick={() => action({ action: 'approve', reason })}>
            Approve
          </button>
          <button className="btn-secondary" type="button" onClick={() => action({ action: { hold: { reason } }, reason })}>
            Hold
          </button>
          <button className="btn-secondary" type="button" onClick={() => action({ action: { mark_paid: { utr } }, reason })}>
            Mark Paid
          </button>
        </div>
      </div>
    </section>
  );
}
