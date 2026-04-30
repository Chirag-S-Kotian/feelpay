import { Suspense, lazy, useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

const AnalyticsCharts = lazy(() =>
  import('../components/AnalyticsCharts').then((mod) => ({ default: mod.AnalyticsCharts })),
);

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
        }>('GET', '/v1/analytics/summary', undefined, undefined, { useApiKey: true, cacheMs: 20_000 }),
        apiRequest<{ chart: { day: string; volume: number }[] }>('GET', '/v1/analytics/volume', undefined, undefined, { useApiKey: true, cacheMs: 20_000 }),
        apiRequest<{ data: { payment_method?: string | null }[] }>('GET', '/v1/payments?limit=120', undefined, undefined, { useApiKey: true, cacheMs: 20_000 }),
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

      <Suspense
        fallback={
          <div className="charts-grid">
            <div className="chart-card"><div className="skeleton" style={{ height: 300 }}></div></div>
            <div className="chart-card"><div className="skeleton" style={{ height: 300 }}></div></div>
          </div>
        }
      >
        <AnalyticsCharts revenueData={revenueData} methodData={methodData} formatCurrency={formatCurrency} />
      </Suspense>
    </div>
  );
}
