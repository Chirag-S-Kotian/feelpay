import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { ArrowRight, CheckCircle2, Code2, CreditCard, ShieldCheck, WalletCards, Zap } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type Feature = { title: string; text: string };
type Metric = { label: string; value: string };
type PricingPlan = {
  name: string;
  price: string;
  caption: string;
  features: string[];
  featured?: boolean;
};
type LandingConfig = {
  brand?: { name?: string; tagline?: string };
  hero?: {
    kicker?: string;
    headline?: string;
    body?: string;
    primary_cta?: string;
    secondary_cta?: string;
  };
  metrics?: Metric[];
  features?: Feature[];
  pricing?: PricingPlan[];
};

const fallbackConfig: LandingConfig = {
  brand: { name: 'FeelPay', tagline: 'Payments that build trust' },
  hero: {
    kicker: 'Configured from admin',
    headline: 'Merchant landing content loads from system config.',
    body: 'Add key landing_page in Admin > System Config to control public copy, pricing, metrics, and feature cards.',
    primary_cta: 'Start testing',
    secondary_cta: 'Merchant login',
  },
  metrics: [
    { label: 'Content source', value: 'Admin config' },
    { label: 'Cache target', value: '<1s' },
    { label: 'Session policy', value: '24h' },
  ],
  features: [
    { title: 'Checkout', text: 'Preview payment methods from merchant configuration.' },
    { title: 'Payouts', text: 'Show payout capability only when configured by admin content.' },
    { title: 'Developers', text: 'Link API keys, webhook testing, and logs to real dashboard flows.' },
    { title: 'Trust', text: 'Public page exposes no secrets and renders text only.' },
  ],
  pricing: [
    {
      name: 'Configure pricing',
      price: 'Admin',
      caption: 'Edit system_config.landing_page',
      features: ['No hardcoded commercial claims', 'Safe public defaults', 'Redis cached response'],
      featured: true,
    },
  ],
};

const iconFor = (title: string) => {
  const value = title.toLowerCase();
  if (value.includes('checkout') || value.includes('card')) return CreditCard;
  if (value.includes('payout') || value.includes('wallet')) return WalletCards;
  if (value.includes('developer') || value.includes('api')) return Code2;
  return ShieldCheck;
};

export default function Landing() {
  const [config, setConfig] = useState<LandingConfig>(fallbackConfig);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await apiRequest<LandingConfig>('GET', '/v1/public/landing-page', undefined, undefined, { cacheMs: 60_000 });
        setConfig(data);
      } catch {
        setConfig(fallbackConfig);
      }
    };
    void load();
  }, []);

  const brand = { ...fallbackConfig.brand, ...config.brand };
  const hero = { ...fallbackConfig.hero, ...config.hero };
  const metrics = config.metrics?.length ? config.metrics : fallbackConfig.metrics || [];
  const features = config.features?.length ? config.features : fallbackConfig.features || [];
  const pricing = config.pricing?.length ? config.pricing : fallbackConfig.pricing || [];

  return (
    <main className="landing-page">
      <nav className="landing-nav">
        <Link to="/landing" className="landing-brand">
          <img src="/feelpay.png" alt={brand.name || 'FeelPay'} />
          <span>{brand.name}</span>
        </Link>
        <div>
          <Link to="/login" className="landing-link">Login</Link>
          <Link to="/signup" className="landing-cta">{hero.primary_cta}</Link>
        </div>
      </nav>

      <section className="landing-hero">
        <div className="landing-copy">
          <p className="landing-kicker">{hero.kicker}</p>
          <h1>{hero.headline}</h1>
          <p>{hero.body}</p>
          <div className="landing-actions">
            <Link to="/signup" className="landing-cta large">
              {hero.primary_cta} <ArrowRight size={18} />
            </Link>
            <Link to="/login" className="landing-secondary">{hero.secondary_cta}</Link>
          </div>
        </div>

        <div className="landing-terminal" aria-label={`${brand.name} platform preview`}>
          <div className="landing-card-visual">
            <div className="trust-rings">
              <span></span>
              <span></span>
            </div>
            <img src="/feelpay.png" alt={brand.name || 'FeelPay'} />
            <strong>{brand.tagline}</strong>
          </div>
          {metrics.map((metric) => (
            <div className="landing-metric-row" key={`${metric.label}-${metric.value}`}>
              <span>{metric.label}</span>
              <strong>{metric.value}</strong>
            </div>
          ))}
        </div>
      </section>

      <section className="landing-rails">
        {features.map((feature) => {
          const Icon = iconFor(feature.title);
          return (
            <article key={feature.title}>
              <Icon size={22} />
              <h3>{feature.title}</h3>
              <p>{feature.text}</p>
            </article>
          );
        })}
      </section>

      <section className="landing-pricing" id="pricing">
        <div className="landing-section-head">
          <p className="landing-kicker">Pricing</p>
          <h2>Pricing managed from admin.</h2>
        </div>
        <div className="pricing-grid">
          {pricing.map((plan) => (
            <article key={plan.name} className={`pricing-card ${plan.featured ? 'featured' : ''}`}>
              {plan.featured && <span className="pricing-badge"><Zap size={13} /> Highlighted</span>}
              <h3>{plan.name}</h3>
              <strong>{plan.price}</strong>
              <p>{plan.caption}</p>
              <ul>
                {(plan.features || []).map((feature) => (
                  <li key={feature}><CheckCircle2 size={15} /> {feature}</li>
                ))}
              </ul>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}
