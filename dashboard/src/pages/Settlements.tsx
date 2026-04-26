import { useState, useEffect } from 'react';
import { Search, Download } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Settlement {
  id: string;
  date: string | null;
  status: string;
  amount: number;
  fee: number;
  tax: number;
  net: number;
  utr: string | null;
}

export default function Settlements() {
  const [settlements, setSettlements] = useState<Settlement[]>([]);
  const [balance, setBalance] = useState<{ available: number; pending: number }>({ available: 0, pending: 0 });
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    const load = async () => {
      try {
        const [settlementRes, balanceRes] = await Promise.all([
          apiRequest<{
            settlements: Array<{
              id: string;
              settlement_date: string | null;
              status: string;
              amount: number;
              fee_amount: number;
              tax_amount: number;
              net_amount: number;
              utr: string | null;
            }>;
          }>('GET', '/v1/settlements', undefined, { useApiKey: true }),
          apiRequest<{ available: number; pending: number }>('GET', '/v1/settlements/balance', undefined, { useApiKey: true }),
        ]);

        setSettlements(
          settlementRes.settlements.map((s) => ({
            id: s.id,
            date: s.settlement_date,
            status: s.status,
            amount: s.amount,
            fee: s.fee_amount,
            tax: s.tax_amount,
            net: s.net_amount,
            utr: s.utr,
          })),
        );
        setBalance(balanceRes);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load settlements');
      }
    };
    load();
  }, []);

  const formatCurrency = (val: number) => new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(val);

  const filteredSettlements = settlements.filter((s) => {
    const statusMatch = filter === 'all' || s.status === filter;
    const q = search.trim().toLowerCase();
    const searchMatch =
      !q ||
      s.id.toLowerCase().includes(q) ||
      (s.utr || '').toLowerCase().includes(q);
    return statusMatch && searchMatch;
  });

  return (
    <div className="page">
      <div className="page-header">
        <h1>Settlements</h1>
        <button className="btn-secondary">
          <Download size={16} /> Export CSV
        </button>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Available Balance</div>
          <div className="stat-value success">{formatCurrency(balance.available)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Pending Settlement</div>
          <div className="stat-value warning">{formatCurrency(balance.pending)}</div>
        </div>
      </div>

      <div className="filter-bar">
        <div className="search-box">
          <Search size={16} />
          <input type="text" placeholder="Search by ID or UTR..." value={search} onChange={(e) => setSearch(e.target.value)} />
        </div>
        <select value={filter} onChange={(e) => setFilter(e.target.value)}>
          <option value="all">All Status</option>
          <option value="pending">Pending</option>
          <option value="completed">Completed</option>
          <option value="failed">Failed</option>
        </select>
      </div>

      <div className="table-card">
        {error && <div className="error-message">{error}</div>}
        <table>
          <thead>
            <tr>
              <th>Settlement ID</th>
              <th>Date</th>
              <th>Status</th>
              <th>Amount</th>
              <th>Fee</th>
              <th>Tax</th>
              <th>Net</th>
              <th>UTR</th>
            </tr>
          </thead>
          <tbody>
            {filteredSettlements.map((settlement) => (
              <tr key={settlement.id}>
                <td className="mono">{settlement.id}</td>
                <td>{settlement.date || '-'}</td>
                <td>
                  <span className={`status-badge ${settlement.status}`}>
                    {settlement.status}
                  </span>
                </td>
                <td>{formatCurrency(settlement.amount)}</td>
                <td>{formatCurrency(settlement.fee)}</td>
                <td>{formatCurrency(settlement.tax)}</td>
                <td className="net">{formatCurrency(settlement.net)}</td>
                <td className="mono">{settlement.utr || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
