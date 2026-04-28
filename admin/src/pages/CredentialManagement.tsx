import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Credential = { id: string; provider: string; key_name: string; masked_value: string; is_valid?: boolean };

export default function CredentialManagement() {
  const [rows, setRows] = useState<Credential[]>([]);
  const [form, setForm] = useState({ provider: '', key_name: '', value: '' });
  const load = () => apiRequest<Credential[]>('GET', '/v1/admin/provider-credentials').then(setRows).catch(() => setRows([]));
  useEffect(() => {
    void load();
  }, []);
  async function save() {
    await apiRequest('POST', '/v1/admin/provider-credentials', form);
    setForm({ provider: '', key_name: '', value: '' });
    load();
  }
  return (
    <section className="page-shell">
      <header className="page-header"><h1>Provider Credentials</h1><button onClick={save}>Save</button></header>
      <div className="filter-grid">
        <input placeholder="Provider" value={form.provider} onChange={e => setForm({ ...form, provider: e.target.value })} />
        <input placeholder="Key name" value={form.key_name} onChange={e => setForm({ ...form, key_name: e.target.value })} />
        <input placeholder="Secret value" type="password" value={form.value} onChange={e => setForm({ ...form, value: e.target.value })} />
      </div>
      <div className="table-wrap"><table><tbody>{rows.map(r => <tr key={r.id}><td>{r.provider}</td><td>{r.key_name}</td><td>{r.masked_value}</td><td>{r.is_valid ? 'Valid' : 'Needs check'}</td></tr>)}</tbody></table></div>
    </section>
  );
}
