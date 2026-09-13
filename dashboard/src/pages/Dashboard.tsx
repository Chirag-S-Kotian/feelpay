import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';
import { formatPaise } from '../utils/money';

type Summary = {
  total_transactions: number;
  total_volume: number;
  successful: number;
  failed: number;
  success_rate: string | number;
};

type VolumePoint = { day: string; volume: number; count: number };

type PaymentRow = {
  id: string;
  order_id: string;
  amount: number;
  payment_method: string;
  status: string;
  created_at: string;
};

type PaymentsResponse = { data: PaymentRow[] };
type AlertItem = {
  type: string;
  severity: 'warning' | 'error' | 'info';
  message: string;
  action_url?: string;
};

function MiniChart({ data, color = 'var(--fp-primary)' }: { data: VolumePoint[]; color?: string }) {
  if (!data.length) return null;
  const max = Math.max(...data.map((d) => d.volume), 1);
  const points = data
    .map((d, i) => {
      const x = data.length > 1 ? (i / (data.length - 1)) * 200 : 0;
      const y = 50 - (d.volume / max) * 40;
      return `${x},${y}`;
    })
    .join(' ');

  return (
    <svg viewBox="0 0 200 55" style={{ width: '100%', height: 80, marginTop: 8 }}>
      <defs>
        <linearGradient id="chartGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.3" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polyline fill="none" stroke={color} strokeWidth="2.5" points={points} strokeLinecap="round" strokeLinejoin="round" />
      <polygon fill="url(#chartGrad)" points={`0,55 ${points} 200,55`} />
    </svg>
  );
}

export default function Dashboard() {
  const navigate = useNavigate();
  const [summary, setSummary] = useState<Summary | null>(null);
  const [volume, setVolume] = useState<VolumePoint[]>([]);
  const [payments, setPayments] = useState<PaymentRow[]>([]);
  const [alerts, setAlerts] = useState<AlertItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError('');
      try {
        const [s, v, p, a] = await Promise.all([
          apiRequest<Summary>('GET', '/v1/analytics/summary', undefined, null, { useApiKey: true }),
          apiRequest<{ chart: VolumePoint[] }>('GET', '/v1/analytics/volume', undefined, null, { useApiKey: true }),
          apiRequest<PaymentsResponse>('GET', '/v1/payments?limit=6', undefined, null, { useApiKey: true }),
          apiRequest<{ alerts: AlertItem[] }>('GET', '/v1/analytics/alerts', undefined, null, { useApiKey: true }).catch(() => ({ alerts: [] })),
        ]);
        setSummary(s);
        setVolume(v.chart || []);
        setPayments((p.data || []).slice(0, 6));
        setAlerts(a.alerts || []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load dashboard');
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const stats = useMemo(() => {
    if (!summary) {
      return [
        { label: 'Total Volume', value: formatPaise(0) },
        { label: 'Successful Txns', value: '0' },
        { label: 'Success Rate', value: '0%' },
        { label: 'Average Payment', value: formatPaise(0) },
      ];
    }
    const avg = summary.total_transactions > 0 ? Math.trunc(summary.total_volume / summary.total_transactions) : 0;
    return [
      { label: 'Total Volume', value: formatPaise(summary.total_volume) },
      { label: 'Successful Txns', value: `${summary.successful.toLocaleString('en-IN')}` },
      { label: 'Success Rate', value: `${summary.success_rate}%` },
      { label: 'Average Payment', value: formatPaise(avg) },
    ];
  }, [summary]);

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Dashboard</h1>
          <p>Live merchant payment overview</p>
        </div>
        <div style={{ display: 'flex', gap: 12 }}>
          <button className="btn btn-secondary" onClick={() => navigate('/pricing')}>Pricing</button>
          <button className="btn btn-secondary" onClick={() => navigate('/exports')}>Export</button>
          <button className="btn btn-primary" onClick={() => navigate('/payments')}>View Payments</button>
        </div>
      </div>

      {error && <div className="card" style={{ color: 'var(--fp-danger)', marginBottom: 16 }}>{error}</div>}
      {alerts.length > 0 && (
        <div className="card" style={{ marginBottom: 16 }}>
          <h3 style={{ marginBottom: 12 }}>Action Required</h3>
          <div style={{ display: 'grid', gap: 8 }}>
            {alerts.map((item) => (
              <button
                key={item.type}
                className="btn btn-secondary"
                style={{ justifyContent: 'flex-start', textAlign: 'left' }}
                onClick={() => item.action_url && navigate(item.action_url.replace('/dashboard', ''))}
              >
                {item.message}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="stats-grid">
        {stats.map((s) => (
          <div key={s.label} className="stat-card">
            <div className="stat-label">{s.label}</div>
            <div className="stat-value">{s.value}</div>
          </div>
        ))}
      </div>

      <div className="chart-container">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <h3>Payment Volume (7 Days)</h3>
        </div>
        {loading ? <div>Loading chart...</div> : <MiniChart data={volume} />}
        <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 4, fontSize: 11, color: 'var(--fp-text-dim)' }}>
          {volume.map((d) => (
            <span key={d.day}>{d.day}</span>
          ))}
        </div>
      </div>

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
            {payments.map((t) => (
              <tr key={t.id}>
                <td style={{ fontFamily: 'monospace', fontSize: 13, color: 'var(--fp-primary-soft)' }}>{t.id}</td>
                <td>{t.order_id}</td>
                <td style={{ fontWeight: 600 }}>{formatPaise(t.amount)}</td>
                <td>{t.payment_method}</td>
                <td><span className={`badge ${t.status}`}>{t.status}</span></td>
                <td style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>{new Date(t.created_at).toLocaleString('en-IN')}</td>
              </tr>
            ))}
            {!payments.length && !loading && (
              <tr>
                <td colSpan={6} style={{ color: 'var(--fp-text-dim)' }}>No transactions found</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
