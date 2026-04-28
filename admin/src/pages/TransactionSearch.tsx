import { useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Txn = {
  id: string;
  order_id: string;
  merchant_id: string;
  amount: number;
  currency: string;
  status: string;
  payment_method?: string;
  customer_email?: string;
  is_frozen: boolean;
  is_flagged: boolean;
  created_at: string;
};

export default function TransactionSearch() {
  const [filters, setFilters] = useState({ order_id: '', customer_email: '', status: '' });
  const [rows, setRows] = useState<Txn[]>([]);

  async function search() {
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => value && params.set(key, value));
    const data = await apiRequest<{ results: Txn[] }>('GET', `/v1/admin/transactions/search?${params}`);
    setRows(data.results);
  }

  return (
    <section className="page-shell">
      <header className="page-header">
        <h1>Transaction Search</h1>
        <button onClick={search}>Search</button>
      </header>
      <div className="filter-grid">
        <input placeholder="Order ID" value={filters.order_id} onChange={event => setFilters({ ...filters, order_id: event.target.value })} />
        <input placeholder="Customer email" value={filters.customer_email} onChange={event => setFilters({ ...filters, customer_email: event.target.value })} />
        <input placeholder="Status" value={filters.status} onChange={event => setFilters({ ...filters, status: event.target.value })} />
      </div>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Order</th>
              <th>Status</th>
              <th>Amount</th>
              <th>Customer</th>
              <th>Flags</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {rows.map(row => (
              <tr key={row.id}>
                <td>{row.order_id}</td>
                <td>{row.status}</td>
                <td>{row.currency} {(row.amount / 100).toFixed(2)}</td>
                <td>{row.customer_email}</td>
                <td>{[row.is_frozen && 'Frozen', row.is_flagged && 'Flagged'].filter(Boolean).join(', ')}</td>
                <td>{new Date(row.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
