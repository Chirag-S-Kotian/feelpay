import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type AuditRow = { id: string; timestamp: string; action: string; entity_type: string; admin_id: string; reason?: string };

export default function AuditLogViewer() {
  const [rows, setRows] = useState<AuditRow[]>([]);
  useEffect(() => {
    apiRequest<AuditRow[]>('GET', '/v1/admin/audit-logs').then(setRows).catch(() => setRows([]));
  }, []);
  return (
    <section className="page-shell">
      <header className="page-header"><h1>Audit Logs</h1></header>
      <div className="table-wrap"><table><tbody>{rows.map(r => <tr key={r.id}><td>{new Date(r.timestamp).toLocaleString()}</td><td>{r.action}</td><td>{r.entity_type}</td><td>{r.admin_id}</td><td>{r.reason}</td></tr>)}</tbody></table></div>
    </section>
  );
}
