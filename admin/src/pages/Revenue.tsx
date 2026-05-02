import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type RevenueSummary = {
  captured_amount: number;
  refunded_amount: number;
  dispute_amount: number;
  settlement_amount: number;
};

function formatInr(value: number) {
  return `₹${(value || 0).toLocaleString('en-IN')}`;
}

export default function Revenue() {
  const [data, setData] = useState<RevenueSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const res = await apiRequest<RevenueSummary>('GET', '/admin/revenue/summary');
        setData(res);
      } catch (err) {
        console.error('Failed to load revenue summary', err);
        setData(null);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Revenue</h1>
          <p>Financial summary from payments, disputes, and settlements</p>
        </div>
      </div>

      {loading ? (
        <div className="card">Loading revenue summary...</div>
      ) : !data ? (
        <div className="card">Revenue data unavailable.</div>
      ) : (
        <div className="stats-grid">
          <article className="stat-card">
            <div className="stat-label">Captured Amount</div>
            <div className="stat-value">{formatInr(data.captured_amount)}</div>
          </article>
          <article className="stat-card">
            <div className="stat-label">Refunded Amount</div>
            <div className="stat-value warning">{formatInr(data.refunded_amount)}</div>
          </article>
          <article className="stat-card">
            <div className="stat-label">Dispute Amount</div>
            <div className="stat-value warning">{formatInr(data.dispute_amount)}</div>
          </article>
          <article className="stat-card">
            <div className="stat-label">Settled Amount</div>
            <div className="stat-value success">{formatInr(data.settlement_amount)}</div>
          </article>
        </div>
      )}
    </div>
  );
}
