
const DEMO_STATS = [
  { label: 'Total Volume', value: '₹12,84,500', change: '+18.2%', positive: true, type: 'primary' },
  { label: 'Successful Txns', value: '1,247', change: '+12.5%', positive: true, type: 'success' },
  { label: 'Success Rate', value: '96.8%', change: '+0.3%', positive: true, type: 'info' },
  { label: 'Avg Ticket Size', value: '₹1,030', change: '-2.1%', positive: false, type: 'warning' },
];

const RECENT_TXNS = [
  { id: 'fp_8a3b2c1d', order: 'ORD-2024-1001', amount: 2499, method: 'UPI', status: 'captured', time: '2 min ago' },
  { id: 'fp_7e4f5a6b', order: 'ORD-2024-1002', amount: 15999, method: 'Card', status: 'captured', time: '8 min ago' },
  { id: 'fp_6d5c4b3a', order: 'ORD-2024-1003', amount: 499, method: 'UPI', status: 'failed', time: '12 min ago' },
  { id: 'fp_5c4d3e2f', order: 'ORD-2024-1004', amount: 3299, method: 'Net Banking', status: 'processing', time: '15 min ago' },
  { id: 'fp_4b3c2d1e', order: 'ORD-2024-1005', amount: 899, method: 'Wallet', status: 'captured', time: '22 min ago' },
  { id: 'fp_3a2b1c0d', order: 'ORD-2024-1006', amount: 7499, method: 'Card', status: 'refunded', time: '35 min ago' },
];

const CHART_DATA = [
  { day: 'Mon', volume: 185000, count: 178 },
  { day: 'Tue', volume: 234000, count: 212 },
  { day: 'Wed', volume: 198000, count: 195 },
  { day: 'Thu', volume: 267000, count: 248 },
  { day: 'Fri', volume: 312000, count: 285 },
  { day: 'Sat', volume: 245000, count: 220 },
  { day: 'Sun', volume: 189000, count: 175 },
];

function MiniChart({ data, color = 'var(--fp-primary)' }: { data: { volume: number }[]; color?: string }) {
  const max = Math.max(...data.map((d: { volume: number }) => d.volume));
  const points = data.map((d: { volume: number }, i: number) => {
    const x = (i / (data.length - 1)) * 200;
    const y = 50 - (d.volume / max) * 40;
    return `${x},${y}`;
  }).join(' ');

  return (
    <svg viewBox="0 0 200 55" style={{ width: '100%', height: 80, marginTop: 8 }}>
      <defs>
        <linearGradient id="chartGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.3" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polyline fill="none" stroke={color} strokeWidth="2.5" points={points}
        strokeLinecap="round" strokeLinejoin="round" />
      <polygon fill="url(#chartGrad)"
        points={`0,55 ${points} 200,55`} />
    </svg>
  );
}

export default function Dashboard() {
  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Dashboard</h1>
          <p>FeelPay Command Deck for India commerce flows</p>
        </div>
        <div style={{ display: 'flex', gap: 12 }}>
          <button className="btn btn-secondary">Export</button>
          <button className="btn btn-primary">+ New Payment</button>
        </div>
      </div>

      <div className="command-deck">
        <div className="command-deck__meta">LIVE CONTROL SURFACE</div>
        <div className="command-deck__title">Own Rail. Own Risk. Own Settlement.</div>
        <div className="command-deck__sub">
          UPI, Cards, NetBanking orchestration under single merchant ledger.
        </div>
      </div>

      {/* Stats */}
      <div className="stats-grid">
        {DEMO_STATS.map(s => (
          <div key={s.label} className={`stat-card ${s.type}`}>
            <div className="stat-label">{s.label}</div>
            <div className="stat-value">{s.value}</div>
            <div className={`stat-change ${s.positive ? 'positive' : 'negative'}`}>
              {s.positive ? '↑' : '↓'} {s.change} from last week
            </div>
          </div>
        ))}
      </div>

      {/* Chart */}
      <div className="chart-container">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <h3>Payment Volume (7 Days)</h3>
          <div className="pill-tabs">
            <button className="pill-tab active">Week</button>
            <button className="pill-tab">Month</button>
            <button className="pill-tab">Quarter</button>
          </div>
        </div>
        <MiniChart data={CHART_DATA} />
        <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 4, fontSize: 11, color: 'var(--fp-text-dim)' }}>
          {CHART_DATA.map(d => <span key={d.day}>{d.day}</span>)}
        </div>
      </div>

      {/* Recent Transactions */}
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Recent Transactions</h3>
        <table className="data-table">
          <thead>
            <tr>
              <th>Payment ID</th>
              <th>Order</th>
              <th>Amount</th>
              <th>Method</th>
              <th>Status</th>
              <th>Time</th>
            </tr>
          </thead>
          <tbody>
            {RECENT_TXNS.map(t => (
              <tr key={t.id}>
                <td style={{ fontFamily: 'monospace', fontSize: 13, color: 'var(--fp-primary-light)' }}>{t.id}</td>
                <td>{t.order}</td>
                <td style={{ fontWeight: 600 }}>₹{t.amount.toLocaleString('en-IN')}</td>
                <td>{t.method}</td>
                <td><span className={`badge ${t.status}`}>{t.status}</span></td>
                <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>{t.time}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
