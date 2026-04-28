import { useState } from 'react';
import { apiRequest } from '../hooks/useApi';

export default function TransactionDetail() {
  const [id, setId] = useState('');
  const [reason, setReason] = useState('');
  const [message, setMessage] = useState('');

  async function run(action: string) {
    await apiRequest('POST', `/v1/admin/transactions/${id}/actions`, { action, reason });
    setMessage('Action applied');
  }

  return (
    <section className="page-shell">
      <header className="page-header"><h1>Transaction Actions</h1></header>
      <div className="filter-grid">
        <input placeholder="Transaction ID" value={id} onChange={event => setId(event.target.value)} />
        <input placeholder="Reason" value={reason} onChange={event => setReason(event.target.value)} />
      </div>
      <div className="button-row">
        <button onClick={() => run('flag_suspicious')}>Flag</button>
        <button onClick={() => run('freeze')}>Freeze</button>
        <button onClick={() => run('unfreeze')}>Unfreeze</button>
        <button onClick={() => run('manual_confirm')}>Manual Confirm</button>
      </div>
      {message && <p>{message}</p>}
    </section>
  );
}
