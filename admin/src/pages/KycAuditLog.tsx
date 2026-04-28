import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type KycAuditRow = {
  id: string;
  merchant_id: string;
  action: string;
  actor_id: string;
  reason?: string | null;
  created_at: string;
};

export default function KycAuditLog() {
  const [rows, setRows] = useState<KycAuditRow[]>([]);
  const [action, setAction] = useState('');

  useEffect(() => {
    const qs = action ? `?action=${encodeURIComponent(action)}` : '';
    apiRequest<KycAuditRow[]>('GET', `/v1/admin/kyc-audit-log${qs}`).then(setRows).catch(() => setRows([]));
  }, [action]);

  return (
    <section className="page-shell">
      <header className="page-header">
        <h1>KYC Audit Log</h1>
        <select value={action} onChange={event => setAction(event.target.value)}>
          <option value="">All actions</option>
          <option value="document_upload">Document uploads</option>
          <option value="document_viewed">Document views</option>
          <option value="status_change">Status changes</option>
          <option value="field_flagged">Field flags</option>
          <option value="note_added">Notes</option>
        </select>
      </header>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Time</th>
              <th>Action</th>
              <th>Merchant</th>
              <th>Actor</th>
              <th>Reason</th>
            </tr>
          </thead>
          <tbody>
            {rows.map(row => (
              <tr key={row.id}>
                <td>{new Date(row.created_at).toLocaleString()}</td>
                <td>{row.action}</td>
                <td>{row.merchant_id}</td>
                <td>{row.actor_id}</td>
                <td>{row.reason || ''}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
