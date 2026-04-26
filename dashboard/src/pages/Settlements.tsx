import { useState, useEffect } from 'react';
import { Search, Download } from 'lucide-react';

interface Settlement {
  id: string;
  date: string;
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

  useEffect(() => {
    // Simulated data
    setSettlements([
      { id: 'set_001', date: '2026-04-25', status: 'completed', amount: 125000, fee: 2500, tax: 450, net: 122050, utr: 'UTR123456789' },
      { id: 'set_002', date: '2026-04-23', status: 'completed', amount: 89000, fee: 1780, tax: 320, net: 86900, utr: 'UTR123456790' },
      { id: 'set_003', date: '2026-04-21', status: 'completed', amount: 156000, fee: 3120, tax: 562, net: 152318, utr: 'UTR123456791' },
      { id: 'set_004', date: '2026-04-26', status: 'pending', amount: 45000, fee: 900, tax: 162, net: 43938, utr: null },
    ]);

    setBalance({ available: 245000, pending: 45000 });
  }, []);

  const formatCurrency = (val: number) => new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(val);

  const filteredSettlements = filter === 'all' 
    ? settlements 
    : settlements.filter(s => s.status === filter);

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
          <input type="text" placeholder="Search by ID or UTR..." />
        </div>
        <select value={filter} onChange={(e) => setFilter(e.target.value)}>
          <option value="all">All Status</option>
          <option value="pending">Pending</option>
          <option value="completed">Completed</option>
          <option value="failed">Failed</option>
        </select>
      </div>

      <div className="table-card">
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
                <td>{settlement.date}</td>
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
