import { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { apiRequest } from '../hooks/useApi';

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
    const load = async () => {
      const [summary, volume, payments] = await Promise.all([
        apiRequest<{
          total_volume: number;
          total_transactions: number;
          success_rate: number;
        }>('GET', '/v1/analytics/summary', undefined, undefined, { useApiKey: true }),
        apiRequest<{ chart: { day: string; volume: number }[] }>('GET', '/v1/analytics/volume', undefined, undefined, { useApiKey: true }),
        apiRequest<{ data: { payment_method?: string | null }[] }>('GET', '/v1/payments?limit=200', undefined, undefined, { useApiKey: true }),
      ]);

      const transactionCount = summary.total_transactions || 0;
      const totalRevenue = summary.total_volume || 0;
      setStats({
        totalRevenue,
        transactionCount,
        successRate: Number(summary.success_rate || 0),
        avgTransaction: transactionCount > 0 ? Math.round(totalRevenue / transactionCount) : 0,
      });

      setRevenueData((volume.chart || []).map((point) => ({ date: point.day, revenue: point.volume })));

      const methodCounts = new Map<string, number>();
      for (const p of payments.data || []) {
        const method = (p.payment_method || 'unknown').replaceAll('_', ' ');
        methodCounts.set(method, (methodCounts.get(method) || 0) + 1);
      }
      const total = (payments.data || []).length || 1;
      const methods = [...methodCounts.entries()].map(([name, value]) => ({
        name: name.toUpperCase(),
        value: Math.round((value / total) * 100),
      }));
      setMethodData(methods.length > 0 ? methods : [{ name: 'NO DATA', value: 100 }]);
    };
    void load();
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
