import { useEffect, useState } from 'react';
import { Pause, Play, Plus, XCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { formatPaise, parseInrToPaise } from '../utils/money';

type Plan = {
  id: string;
  name: string;
  amount: number;
  currency: string;
  billing_cycle: string;
  trial_days?: number | null;
};

type Subscription = {
  id: string;
  customer_id: string;
  plan_name: string;
  amount: number;
  currency: string;
  billing_cycle: string;
  status: string;
  current_period_end?: string | null;
  cancel_at_period_end: boolean;
};

const money = (amount: number) => formatPaise(amount);

export default function Subscriptions() {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [planForm, setPlanForm] = useState({ name: '', amount: '', billing_cycle: 'monthly', trial_days: '' });
  const [subForm, setSubForm] = useState({ customer_id: '', plan_id: '', plan_name: '', amount: '', billing_cycle: 'monthly' });

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      const [planRes, subRes] = await Promise.all([
        apiRequest<Plan[]>('GET', '/v1/subscriptions/plans', undefined, undefined, { cacheMs: 5_000 }),
        apiRequest<Subscription[]>('GET', '/v1/subscriptions', undefined, undefined, { cacheMs: 5_000 }),
      ]);
      setPlans(planRes);
      setSubscriptions(subRes);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load subscriptions');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const createPlan = async () => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      await apiRequest('POST', '/v1/subscriptions/plans', {
        name: planForm.name.trim(),
        amount: parseInrToPaise(planForm.amount) || 0,
        currency: 'INR',
        billing_cycle: planForm.billing_cycle,
        trial_days: planForm.trial_days ? Number(planForm.trial_days) : null,
      });
      setPlanForm({ name: '', amount: '', billing_cycle: 'monthly', trial_days: '' });
      setMessage('Plan created.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create plan');
    } finally {
      setSaving(false);
    }
  };

  const createSubscription = async () => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      const selectedPlan = plans.find((plan) => plan.id === subForm.plan_id);
      await apiRequest('POST', '/v1/subscriptions', {
        customer_id: subForm.customer_id.trim(),
        plan_id: subForm.plan_id || null,
        plan_name: selectedPlan?.name || subForm.plan_name.trim(),
        amount: selectedPlan?.amount || parseInrToPaise(subForm.amount) || 0,
        currency: 'INR',
        billing_cycle: selectedPlan?.billing_cycle || subForm.billing_cycle,
      });
      setSubForm({ customer_id: '', plan_id: '', plan_name: '', amount: '', billing_cycle: 'monthly' });
      setMessage('Subscription created.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create subscription');
    } finally {
      setSaving(false);
    }
  };

  const updateSubscription = async (id: string, action: 'pause' | 'resume' | 'cancel') => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      await apiRequest('POST', `/v1/subscriptions/${id}/${action}`, action === 'cancel' ? { cancel_at_period_end: false } : {});
      setMessage(`Subscription ${action}d.`);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${action} subscription`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) return <div className="loading">Loading subscriptions...</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Subscriptions</h1>
          <p>Recurring plans, customer subscriptions, and billing lifecycle controls.</p>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}
      {message && <div className="success-banner">{message}</div>}

      <div className="grid-2">
        <div className="card">
          <h3>Create Plan</h3>
          <div className="form-grid">
            <input className="form-input" value={planForm.name} onChange={(e) => setPlanForm({ ...planForm, name: e.target.value })} placeholder="Plan name" />
            <input className="form-input" value={planForm.amount} onChange={(e) => setPlanForm({ ...planForm, amount: e.target.value })} placeholder="Amount INR" type="number" min="1" />
            <select className="form-input" value={planForm.billing_cycle} onChange={(e) => setPlanForm({ ...planForm, billing_cycle: e.target.value })}>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
              <option value="yearly">Yearly</option>
            </select>
            <input className="form-input" value={planForm.trial_days} onChange={(e) => setPlanForm({ ...planForm, trial_days: e.target.value })} placeholder="Trial days" type="number" min="0" />
          </div>
          <button className="btn btn-primary" onClick={() => void createPlan()} disabled={saving || !planForm.name || !planForm.amount}>
            <Plus size={16} /> Create Plan
          </button>
        </div>

        <div className="card">
          <h3>Create Subscription</h3>
          <div className="form-grid">
            <input className="form-input" value={subForm.customer_id} onChange={(e) => setSubForm({ ...subForm, customer_id: e.target.value })} placeholder="Customer UUID" />
            <select className="form-input" value={subForm.plan_id} onChange={(e) => setSubForm({ ...subForm, plan_id: e.target.value })}>
              <option value="">Custom plan</option>
              {plans.map((plan) => <option key={plan.id} value={plan.id}>{plan.name} - {money(plan.amount)}</option>)}
            </select>
            {!subForm.plan_id && (
              <>
                <input className="form-input" value={subForm.plan_name} onChange={(e) => setSubForm({ ...subForm, plan_name: e.target.value })} placeholder="Plan name" />
                <input className="form-input" value={subForm.amount} onChange={(e) => setSubForm({ ...subForm, amount: e.target.value })} placeholder="Amount INR" type="number" min="1" />
              </>
            )}
          </div>
          <button className="btn btn-primary" onClick={() => void createSubscription()} disabled={saving || !subForm.customer_id || (!subForm.plan_id && (!subForm.plan_name || !subForm.amount))}>
            <Plus size={16} /> Create Subscription
          </button>
        </div>
      </div>

      <div className="card">
        <h3>Subscription Plans</h3>
        <table className="data-table">
          <thead><tr><th>Name</th><th>Amount</th><th>Cycle</th><th>Trial</th></tr></thead>
          <tbody>
            {plans.map((plan) => (
              <tr key={plan.id}><td>{plan.name}</td><td>{money(plan.amount)}</td><td>{plan.billing_cycle}</td><td>{plan.trial_days || 0} days</td></tr>
            ))}
            {plans.length === 0 && <tr><td colSpan={4}>No plans yet.</td></tr>}
          </tbody>
        </table>
      </div>

      <div className="card">
        <h3>Active Subscriptions</h3>
        <table className="data-table">
          <thead><tr><th>Plan</th><th>Customer</th><th>Amount</th><th>Status</th><th>Renews</th><th>Actions</th></tr></thead>
          <tbody>
            {subscriptions.map((sub) => (
              <tr key={sub.id}>
                <td>{sub.plan_name}</td>
                <td className="mono">{sub.customer_id}</td>
                <td>{money(sub.amount)}</td>
                <td><span className={`badge ${sub.status === 'active' ? 'captured' : 'pending'}`}>{sub.status}</span></td>
                <td>{sub.current_period_end ? new Date(sub.current_period_end).toLocaleDateString() : 'Not set'}</td>
                <td className="action-row">
                  <button className="btn btn-secondary" onClick={() => void updateSubscription(sub.id, 'pause')} disabled={saving || sub.status !== 'active'}><Pause size={14} /> Pause</button>
                  <button className="btn btn-secondary" onClick={() => void updateSubscription(sub.id, 'resume')} disabled={saving || sub.status === 'active'}><Play size={14} /> Resume</button>
                  <button className="btn btn-danger" onClick={() => void updateSubscription(sub.id, 'cancel')} disabled={saving || sub.status === 'cancelled'}><XCircle size={14} /> Cancel</button>
                </td>
              </tr>
            ))}
            {subscriptions.length === 0 && <tr><td colSpan={6}>No subscriptions yet.</td></tr>}
          </tbody>
        </table>
      </div>
    </div>
  );
}
