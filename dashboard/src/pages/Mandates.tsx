import { useEffect, useState } from 'react';
import { Play, Plus, XCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { formatPaise, parseInrToPaise } from '../utils/money';

type Mandate = {
  id: string;
  mandate_type: string;
  mandate_reference?: string | null;
  status: string;
  amount_limit: number;
  currency: string;
  frequency: string;
  payer_vpa?: string | null;
  payer_name?: string | null;
  created_at: string;
};

const money = (amount: number) => formatPaise(amount);

export default function Mandates() {
  const [mandates, setMandates] = useState<Mandate[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [form, setForm] = useState({ mandate_type: 'upi', amount_limit: '', frequency: 'monthly', payer_vpa: '', payer_name: '' });
  const [executeAmount, setExecuteAmount] = useState<Record<string, string>>({});

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      setMandates(await apiRequest<Mandate[]>('GET', '/v1/mandates', undefined, undefined, { cacheMs: 5_000 }));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load AutoPay records');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const createMandate = async () => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      await apiRequest('POST', '/v1/mandates', {
        mandate_type: form.mandate_type,
        amount_limit: parseInrToPaise(form.amount_limit) || 0,
        currency: 'INR',
        frequency: form.frequency,
        payer_vpa: form.payer_vpa.trim() || null,
        payer_name: form.payer_name.trim() || null,
      });
      setForm({ mandate_type: 'upi', amount_limit: '', frequency: 'monthly', payer_vpa: '', payer_name: '' });
      setMessage('AutoPay initiated.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create AutoPay mandate');
    } finally {
      setSaving(false);
    }
  };

  const runAction = async (id: string, action: 'execute' | 'cancel') => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      await apiRequest('POST', `/v1/mandates/${id}/${action}`, action === 'execute' ? { amount: parseInrToPaise(executeAmount[id] || '0') || 0 } : {});
      setMessage(action === 'execute' ? 'AutoPay executed.' : 'AutoPay cancelled.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${action} AutoPay mandate`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) return <div className="loading">Loading AutoPay...</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>AutoPay</h1>
          <p>UPI AutoPay & recurring mandate creation, execution, and cancellation.</p>
        </div>
      </div>
      {error && <div className="error-banner">{error}</div>}
      {message && <div className="success-banner">{message}</div>}

      <div className="card">
        <h3>Set up AutoPay</h3>
        <div className="form-grid">
          <select className="form-input" value={form.mandate_type} onChange={(e) => setForm({ ...form, mandate_type: e.target.value })}>
            <option value="upi">UPI AutoPay</option>
            <option value="nach">e-NACH</option>
            <option value="card">Card Recurring</option>
          </select>
          <input className="form-input" value={form.amount_limit} onChange={(e) => setForm({ ...form, amount_limit: e.target.value })} placeholder="Limit INR" type="number" min="1" />
          <select className="form-input" value={form.frequency} onChange={(e) => setForm({ ...form, frequency: e.target.value })}>
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
            <option value="monthly">Monthly</option>
            <option value="as_presented">As presented</option>
          </select>
          <input className="form-input" value={form.payer_vpa} onChange={(e) => setForm({ ...form, payer_vpa: e.target.value })} placeholder="payer@upi" />
          <input className="form-input" value={form.payer_name} onChange={(e) => setForm({ ...form, payer_name: e.target.value })} placeholder="Payer name" />
        </div>
        <button className="btn btn-primary" onClick={() => void createMandate()} disabled={saving || !form.amount_limit}>
          <Plus size={16} /> Set up AutoPay
        </button>
      </div>

      <div className="card">
        <h3>Active AutoPay Mandates</h3>
        <table className="data-table">
          <thead><tr><th>Reference / UMN</th><th>Payer</th><th>Limit</th><th>Frequency</th><th>Status</th><th>Actions</th></tr></thead>
          <tbody>
            {mandates.map((mandate) => (
              <tr key={mandate.id}>
                <td className="mono">{mandate.mandate_reference || mandate.id}</td>
                <td>{mandate.payer_name || mandate.payer_vpa || 'Not provided'}</td>
                <td>{money(mandate.amount_limit)}</td>
                <td>{mandate.frequency}</td>
                <td><span className={`badge ${mandate.status === 'active' ? 'captured' : 'pending'}`}>{mandate.status}</span></td>
                <td className="action-row">
                  <input className="form-input compact-input" value={executeAmount[mandate.id] || ''} onChange={(e) => setExecuteAmount({ ...executeAmount, [mandate.id]: e.target.value })} placeholder="INR" type="number" min="1" />
                  <button className="btn btn-secondary" onClick={() => void runAction(mandate.id, 'execute')} disabled={saving || mandate.status !== 'active' || !executeAmount[mandate.id]}><Play size={14} /> Execute</button>
                  <button className="btn btn-danger" onClick={() => void runAction(mandate.id, 'cancel')} disabled={saving || mandate.status === 'revoked'}><XCircle size={14} /> Cancel</button>
                </td>
              </tr>
            ))}
            {mandates.length === 0 && <tr><td colSpan={6}>No AutoPay mandates yet.</td></tr>}
          </tbody>
        </table>
      </div>
    </div>
  );
}
