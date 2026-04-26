import { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

const COLORS = ['#22c55e', '#3b82f6', '#f59e0b', '#ef4444'];

export default function Analytics() {
  const [period, setPeriod] = useState('7d');
  const [stats, setStats] = useState({
    totalRevenue: 0,
    transactionCount: 0,
    successRate: 0,
    avgTransaction: 0,
  });
  const [revenueData, setRevenueData] = useState<{ date: string; revenue: number }[]>([]);
  const [methodData, setMethodData] = useState<{ name: string; value: number }[]>([]);

  useEffect(() => {
    // Simulated data - in production this would be an API call
    setStats({
      totalRevenue: 2845000,
      transactionCount: 1247,
      successRate: 98.2,
      avgTransaction: 2281,
    });

    setRevenueData([
      { date: 'Mon', revenue: 45000 },
      { date: 'Tue', revenue: 52000 },
      { date: 'Wed', revenue: 48000 },
      { date: 'Thu', revenue: 61000 },
      { date: 'Fri', revenue: 55000 },
      { date: 'Sat', revenue: 42000 },
      { date: 'Sun', revenue: 38000 },
    ]);

    setMethodData([
      { name: 'UPI', value: 45 },
      { name: 'Card', value: 30 },
      { name: 'Net Banking', value: 15 },
      { name: 'Wallet', value: 10 },
    ]);
  }, [period]);

  const formatCurrency = (val: number) => new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(val);

  return (
    <div className="page">
      <div className="page-header">
        <h1>Analytics</h1>
        <select value={period} onChange={(e) => setPeriod(e.target.value)} className="period-select">
          <option value="7d">Last 7 days</option>
          <option value="30d">Last 30 days</option>
          <option value="90d">Last 90 days</option>
        </select>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Total Revenue</div>
          <div className="stat-value">{formatCurrency(stats.totalRevenue)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Transactions</div>
          <div className="stat-value">{stats.transactionCount.toLocaleString()}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Success Rate</div>
          <div className="stat-value">{stats.successRate}%</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Avg. Transaction</div>
          <div className="stat-value">{formatCurrency(stats.avgTransaction)}</div>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-card">
          <h3>Revenue Trend</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={revenueData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="date" stroke="#9ca3af" />
              <YAxis stroke="#9ca3af" tickFormatter={(v) => `₹${v/1000}k`} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1f2937', border: 'none', borderRadius: '8px' }}
                formatter={(val: number) => [formatCurrency(val), 'Revenue']}
              />
              <Line type="monotone" dataKey="revenue" stroke="#22c55e" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className="chart-card">
          <h3>Payment Methods</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={methodData}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={100}
                paddingAngle={2}
                dataKey="value"
              >
{methodData.map((_entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip contentStyle={{ backgroundColor: '#1f2937', border: 'none', borderRadius: '8px' }} />
            </PieChart>
          </ResponsiveContainer>
          <div className="legend">
            {methodData.map((entry, index) => (
              <div key={entry.name} className="legend-item">
                <span className="legend-color" style={{ backgroundColor: COLORS[index] }}></span>
                {entry.name}: {entry.value}%
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
