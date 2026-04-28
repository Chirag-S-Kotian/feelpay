import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Tenant = { id: string; name: string; slug: string; status: string };

export default function TenantManagement() {
  const [rows, setRows] = useState<Tenant[]>([]);
  const [form, setForm] = useState({ name: '', slug: '', status: 'active' });
  const load = () => apiRequest<Tenant[]>('GET', '/v1/admin/tenants').then(setRows).catch(() => setRows([]));
  useEffect(() => {
    void load();
  }, []);
  async function create() {
    await apiRequest('POST', '/v1/admin/tenants', form);
    setForm({ name: '', slug: '', status: 'active' });
    load();
  }
  return (
    <section className="page-shell">
      <header className="page-header"><h1>Tenant Management</h1><button onClick={create}>Create</button></header>
      <div className="filter-grid">
        <input placeholder="Name" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
        <input placeholder="Slug" value={form.slug} onChange={e => setForm({ ...form, slug: e.target.value })} />
      </div>
      <div className="table-wrap"><table><tbody>{rows.map(r => <tr key={r.id}><td>{r.name}</td><td>{r.slug}</td><td>{r.status}</td></tr>)}</tbody></table></div>
    </section>
  );
}
