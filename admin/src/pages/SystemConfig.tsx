import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type ConfigRow = { key: string; value: unknown; description?: string };

export default function SystemConfig() {
  const [rows, setRows] = useState<ConfigRow[]>([]);
  const [key, setKey] = useState('');
  const [value, setValue] = useState('{}');
  const load = () => apiRequest<ConfigRow[]>('GET', '/v1/admin/system-config').then(setRows).catch(() => setRows([]));
  useEffect(() => {
    void load();
  }, []);
  async function save() {
    await apiRequest('PUT', `/v1/admin/system-config/${encodeURIComponent(key)}`, { value: JSON.parse(value) });
    load();
  }
  return (
    <section className="page-shell">
      <header className="page-header"><h1>System Config</h1><button onClick={save}>Save</button></header>
      <div className="filter-grid">
        <input placeholder="Key" value={key} onChange={e => setKey(e.target.value)} />
        <input placeholder="JSON value" value={value} onChange={e => setValue(e.target.value)} />
      </div>
      <div className="table-wrap"><table><tbody>{rows.map(r => <tr key={r.key}><td>{r.key}</td><td>{JSON.stringify(r.value)}</td><td>{r.description}</td></tr>)}</tbody></table></div>
    </section>
  );
}
