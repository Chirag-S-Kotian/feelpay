import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type AdminUser = { id: string; email: string; name: string; role: string; is_active: boolean };

export default function AdminUserManagement() {
  const [rows, setRows] = useState<AdminUser[]>([]);
  const [form, setForm] = useState({ email: 'ops.manager@feelpay.local', password: 'AdminSecurePassword2026!', name: 'Operations Manager', role: 'admin' });
  const load = () => apiRequest<AdminUser[]>('GET', '/v1/admin/users').then(setRows).catch(() => setRows([]));
  useEffect(() => {
    void load();
  }, []);
  async function create() {
    await apiRequest('POST', '/v1/admin/users', form);
    setForm({ email: 'ops.manager@feelpay.local', password: 'AdminSecurePassword2026!', name: 'Operations Manager', role: 'admin' });
    load();
  }
  return (
    <section className="page-shell">
      <header className="page-header"><h1>Admin Users</h1><button onClick={create}>Create</button></header>
      <div className="filter-grid">
        <input placeholder="Email" name="admin-email" autoComplete="username" value={form.email} onChange={e => setForm({ ...form, email: e.target.value })} />
        <input placeholder="Name" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
        <input placeholder="Password" type="password" autoComplete="new-password" name="new-admin-password" value={form.password} onChange={e => setForm({ ...form, password: e.target.value })} />
      </div>
      <div className="table-wrap"><table><tbody>{rows.map(r => <tr key={r.id}><td>{r.email}</td><td>{r.name}</td><td>{r.role}</td><td>{r.is_active ? 'Active' : 'Inactive'}</td></tr>)}</tbody></table></div>
    </section>
  );
}
