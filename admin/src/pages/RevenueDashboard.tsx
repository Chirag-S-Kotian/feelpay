import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Breakdown = { label: string; transaction_count: number; volume: number; revenue: number };

export default function RevenueDashboard() {
  const [data, setData] = useState<{ by_payment_method: Breakdown[]; by_fee_plan: Breakdown[] }>({ by_payment_method: [], by_fee_plan: [] });

  useEffect(() => {
    apiRequest<typeof data>('GET', '/v1/admin/revenue').then(setData).catch(() => undefined);
  }, []);

  return (
    <section className="page-shell">
      <header className="page-header"><h1>Revenue Dashboard</h1></header>
      <div className="metric-grid">
        {[...data.by_payment_method, ...data.by_fee_plan].map(item => (
          <article className="metric-card" key={`${item.label}-${item.revenue}`}>
            <span>{item.label}</span>
            <strong>INR {(item.revenue / 100).toFixed(2)}</strong>
            <small>{item.transaction_count} transactions</small>
          </article>
        ))}
      </div>
    </section>
  );
}
