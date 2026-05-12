import { useEffect, useState } from 'react';
import { Copy, Plus } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type VirtualAccount = {
  id: string;
  account_number: string;
  ifsc_code: string;
  bank_name?: string | null;
  beneficiary_name?: string | null;
  status: string;
  created_at: string;
};

export default function VirtualAccounts() {
  const [accounts, setAccounts] = useState<VirtualAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [beneficiaryName, setBeneficiaryName] = useState('');
  const [customerId, setCustomerId] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      setAccounts(await apiRequest<VirtualAccount[]>('GET', '/v1/virtual-accounts', undefined, undefined, { cacheMs: 5_000 }));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load virtual accounts');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const createAccount = async () => {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      await apiRequest('POST', '/v1/virtual-accounts', {
        customer_id: customerId.trim() || null,
        beneficiary_name: beneficiaryName.trim() || null,
      });
      setBeneficiaryName('');
      setCustomerId('');
      setMessage('Virtual account created.');
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create virtual account');
    } finally {
      setSaving(false);
    }
  };

  const copyAccount = (account: VirtualAccount) => {
    navigator.clipboard.writeText(`${account.account_number} / ${account.ifsc_code}`);
    setMessage('Account details copied.');
  };

  if (loading) return <div className="loading">Loading virtual accounts...</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Virtual Accounts</h1>
          <p>Dedicated collection accounts for bank-transfer reconciliation.</p>
        </div>
      </div>
      {error && <div className="error-banner">{error}</div>}
      {message && <div className="success-banner">{message}</div>}

      <div className="card">
        <h3>Create Collection Account</h3>
        <div className="form-grid">
          <input className="form-input" value={beneficiaryName} onChange={(e) => setBeneficiaryName(e.target.value)} placeholder="Beneficiary name" />
          <input className="form-input" value={customerId} onChange={(e) => setCustomerId(e.target.value)} placeholder="Customer UUID optional" />
        </div>
        <button className="btn btn-primary" onClick={() => void createAccount()} disabled={saving}>
          <Plus size={16} /> Create Virtual Account
        </button>
      </div>

      <div className="card">
        <table className="data-table">
          <thead><tr><th>Beneficiary</th><th>Account</th><th>IFSC</th><th>Bank</th><th>Status</th><th>Action</th></tr></thead>
          <tbody>
            {accounts.map((account) => (
              <tr key={account.id}>
                <td>{account.beneficiary_name || 'FeelPay Beneficiary'}</td>
                <td className="mono">{account.account_number}</td>
                <td className="mono">{account.ifsc_code}</td>
                <td>{account.bank_name || 'FeelPay Virtual Bank'}</td>
                <td><span className={`badge ${account.status === 'active' ? 'captured' : 'pending'}`}>{account.status}</span></td>
                <td><button className="btn btn-secondary" onClick={() => copyAccount(account)}><Copy size={14} /> Copy</button></td>
              </tr>
            ))}
            {accounts.length === 0 && <tr><td colSpan={6}>No virtual accounts yet.</td></tr>}
          </tbody>
        </table>
      </div>
    </div>
  );
}
