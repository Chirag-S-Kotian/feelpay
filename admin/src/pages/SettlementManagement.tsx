import { useState } from 'react';
import { apiRequest } from '../hooks/useApi';

export default function SettlementManagement() {
  const [id, setId] = useState('');
  const [utr, setUtr] = useState('');
  const [reason, setReason] = useState('');

  async function action(payload: unknown) {
    await apiRequest('POST', `/v1/admin/settlements/${id}/actions`, payload);
  }

  return (
    <section className="page-shell">
      <header className="page-header"><h1>Settlement Management</h1></header>
      <div className="filter-grid">
        <input placeholder="Settlement ID" value={id} onChange={event => setId(event.target.value)} />
        <input placeholder="UTR" value={utr} onChange={event => setUtr(event.target.value)} />
        <input placeholder="Reason" value={reason} onChange={event => setReason(event.target.value)} />
      </div>
      <div className="button-row">
        <button onClick={() => action({ action: 'approve', reason })}>Approve</button>
        <button onClick={() => action({ action: { hold: { reason } }, reason })}>Hold</button>
        <button onClick={() => action({ action: { mark_paid: { utr } }, reason })}>Mark Paid</button>
      </div>
    </section>
  );
}
