import { useEffect, useMemo, useState } from 'react';
import { CheckCircle2, Sparkles } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type PricingPlan = {
  name: string;
  price: string;
  caption: string;
  features: string[];
  featured?: boolean;
};

type LandingConfig = {
  pricing?: PricingPlan[];
};

const fallbackPlans: PricingPlan[] = [
  {
    name: 'Starter',
    price: '0.90% + GST',
    caption: 'Ideal for early merchants',
    features: ['Hosted checkout', 'UPI/cards/wallets', 'Settlement reports'],
  },
  {
    name: 'Growth',
    price: '0.75% + GST',
    caption: 'For scaling teams',
    features: ['Everything in Starter', 'Payouts', 'Webhook retries', 'Priority support'],
    featured: true,
  },
  {
    name: 'Scale',
    price: 'Custom',
    caption: 'High volume / platform use-cases',
    features: ['Custom routing', 'SLA support', 'Risk controls', 'Custom reconciliation'],
  },
];

export default function Pricing() {
  const [plans, setPlans] = useState<PricingPlan[]>(fallbackPlans);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError('');
      try {
        const config = await apiRequest<LandingConfig>('GET', '/v1/public/landing-page', undefined, undefined, { cacheMs: 20_000 });
        if (config.pricing?.length) {
          setPlans(config.pricing);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unable to load pricing config');
      } finally {
        setLoading(false);
      }
    };
    void load();
  }, []);

  const featuredCount = useMemo(() => plans.filter(plan => plan.featured).length, [plans]);

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Pricing Plans</h1>
          <p>Commercial plans synced from landing config for merchant visibility.</p>
        </div>
      </div>

      {error && <div className="card" style={{ color: 'var(--fp-danger)', marginBottom: 12 }}>{error}</div>}

      <div className="card" style={{ marginBottom: 14 }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', color: 'var(--fp-text-muted)' }}>
          <Sparkles size={16} />
          <span>{loading ? 'Syncing pricing…' : `Plans loaded: ${plans.length} | Highlighted: ${featuredCount}`}</span>
        </div>
      </div>

      <section className="pricing-grid">
        {plans.map((plan) => (
          <article key={plan.name} className={`pricing-card ${plan.featured ? 'featured' : ''}`}>
            {plan.featured && <span className="pricing-badge">Recommended</span>}
            <h3>{plan.name}</h3>
            <strong>{plan.price}</strong>
            <p>{plan.caption}</p>
            <ul>
              {plan.features?.map((feature) => (
                <li key={feature}>
                  <CheckCircle2 size={15} />
                  {feature}
                </li>
              ))}
            </ul>
          </article>
        ))}
      </section>
    </div>
  );
}
