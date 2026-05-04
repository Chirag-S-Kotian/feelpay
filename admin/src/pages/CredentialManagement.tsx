import { useEffect, useMemo, useState } from 'react';
import { CheckCircle, KeyRound, Landmark, ShieldCheck } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type Credential = {
  id: string;
  provider: string;
  key_name: string;
  masked_value: string;
  last_validated?: string | null;
  is_valid?: boolean | null;
  updated_at: string;
};

type BankAccount = {
  id: string;
  merchant_id: string;
  merchant_name: string;
  merchant_email: string;
  account_holder_name: string;
  ifsc_code: string;
  account_type: string;
  bank_name?: string | null;
  verification_status: string;
  is_primary?: boolean | null;
  created_at: string;
  verified_at?: string | null;
};

const CONNECTOR_PROVIDERS = [
  'razorpay',
  'cashfree',
  'payu',
  'phonepe',
  'juspay',
  'npci_upi',
  'hdfc',
  'icici',
  'axis',
  'yesbank',
  'simulator',
];

const CONNECTOR_KEYS = [
  'key_id',
  'key_secret',
  'merchant_id',
  'client_id',
  'client_secret',
  'webhook_secret',
  'upi_vpa',
  'payout_account',
];

function formatDate(value?: string | null) {
  if (!value) return '-';
  return new Date(value).toLocaleString();
}

export default function CredentialManagement() {
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [bankAccounts, setBankAccounts] = useState<BankAccount[]>([]);
  const [form, setForm] = useState({ provider: 'simulator', key_name: 'key_id', value: '' });
  const [bankStatus, setBankStatus] = useState('');
  const [bankQuery, setBankQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({ limit: '100' });
      if (bankStatus) params.set('status', bankStatus);
      if (bankQuery.trim()) params.set('q', bankQuery.trim());
      const [credentialData, bankData] = await Promise.all([
        apiRequest<{ items: Credential[] }>('GET', '/admin/provider-credentials'),
        apiRequest<{ items: BankAccount[] }>('GET', `/admin/bank-accounts?${params.toString()}`),
      ]);
      setCredentials(credentialData.items || []);
      setBankAccounts(bankData.items || []);
    } catch (err) {
      console.error('Failed to load connector workspace', err);
      setCredentials([]);
      setBankAccounts([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [bankQuery, bankStatus]);

  const providerCount = useMemo(
    () => new Set(credentials.map(row => row.provider)).size,
    [credentials],
  );

  const verifiedBankCount = useMemo(
    () => bankAccounts.filter(row => row.verification_status === 'verified').length,
    [bankAccounts],
  );

  const save = async () => {
    setSaving(true);
    setMessage(null);
    try {
      await apiRequest('POST', '/admin/provider-credentials', form);
      setForm({ ...form, value: '' });
      setMessage('Credential saved and encrypted.');
      await load();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : 'Save failed');
    } finally {
      setSaving(false);
    }
  };

  const validate = async (id: string) => {
    setMessage(null);
    try {
      await apiRequest('POST', `/admin/provider-credentials/${id}/validate`);
      setMessage('Credential marked valid.');
      await load();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : 'Validation failed');
    }
  };

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Banks & Connectors</h1>
          <p>White-label rail setup for payment, payout, UPI, and bank integrations</p>
        </div>
      </div>

      <div className="stats-grid">
        <article className="stat-card">
          <div className="stat-label">Connector Providers</div>
          <div className="stat-value">{providerCount}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">Stored Secrets</div>
          <div className="stat-value">{credentials.length}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">Merchant Bank Accounts</div>
          <div className="stat-value">{bankAccounts.length}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">Verified Banks</div>
          <div className="stat-value success">{verifiedBankCount}</div>
        </article>
      </div>

      <div className="grid grid-2">
        <section className="card">
          <h2><KeyRound size={18} /> Add Connector Secret</h2>
          {message && <div className="alert alert-success">{message}</div>}
          <div className="filter-grid">
            <select
              value={form.provider}
              onChange={event => setForm({ ...form, provider: event.target.value })}
              aria-label="Provider"
            >
              {CONNECTOR_PROVIDERS.map(provider => (
                <option key={provider} value={provider}>{provider}</option>
              ))}
            </select>
            <select
              value={form.key_name}
              onChange={event => setForm({ ...form, key_name: event.target.value })}
              aria-label="Credential key"
            >
              {CONNECTOR_KEYS.map(key => (
                <option key={key} value={key}>{key}</option>
              ))}
            </select>
            <input
              type="password"
              placeholder="Secret value"
              value={form.value}
              onChange={event => setForm({ ...form, value: event.target.value })}
            />
          </div>
          <button className="btn-primary" onClick={save} disabled={saving || !form.value.trim()}>
            {saving ? 'Saving...' : 'Save encrypted secret'}
          </button>
        </section>

        <section className="card">
          <h2><ShieldCheck size={18} /> Integration Checklist</h2>
          <div className="check-list">
            <div><CheckCircle size={16} /> Store connector keys per provider</div>
            <div><CheckCircle size={16} /> Validate webhook secrets before live traffic</div>
            <div><CheckCircle size={16} /> Verify merchant settlement bank accounts</div>
            <div><CheckCircle size={16} /> Keep simulator available for tenant onboarding</div>
          </div>
        </section>
      </div>

      <section className="table-card" style={{ marginTop: 18 }}>
        <div className="page-header" style={{ marginBottom: 12 }}>
          <h2><KeyRound size={18} /> Connector Credentials</h2>
        </div>
        {loading ? (
          <div>Loading connectors...</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Provider</th>
                <th>Key</th>
                <th>Secret</th>
                <th>Status</th>
                <th>Last Validated</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {credentials.map(row => (
                <tr key={row.id}>
                  <td>{row.provider}</td>
                  <td>{row.key_name}</td>
                  <td>{row.masked_value}</td>
                  <td><span className={`badge ${row.is_valid ? 'success' : 'pending'}`}>{row.is_valid ? 'valid' : 'needs check'}</span></td>
                  <td>{formatDate(row.last_validated)}</td>
                  <td><button onClick={() => validate(row.id)}>Mark valid</button></td>
                </tr>
              ))}
              {credentials.length === 0 && (
                <tr><td colSpan={6}>No connector credentials configured</td></tr>
              )}
            </tbody>
          </table>
        )}
      </section>

      <section className="table-card" style={{ marginTop: 18 }}>
        <div className="page-header" style={{ marginBottom: 12 }}>
          <h2><Landmark size={18} /> Merchant Bank Accounts</h2>
        </div>
        <div className="filter-bar">
          <input
            placeholder="Search merchant, IFSC, holder"
            value={bankQuery}
            onChange={event => setBankQuery(event.target.value)}
          />
          <select value={bankStatus} onChange={event => setBankStatus(event.target.value)}>
            <option value="">All verification states</option>
            <option value="pending">Pending</option>
            <option value="verified">Verified</option>
            <option value="failed">Failed</option>
          </select>
        </div>
        <table>
          <thead>
            <tr>
              <th>Merchant</th>
              <th>Holder</th>
              <th>Bank</th>
              <th>IFSC</th>
              <th>Type</th>
              <th>Status</th>
              <th>Primary</th>
            </tr>
          </thead>
          <tbody>
            {bankAccounts.map(row => (
              <tr key={row.id}>
                <td>{row.merchant_name}<br /><span className="muted-text">{row.merchant_email}</span></td>
                <td>{row.account_holder_name}</td>
                <td>{row.bank_name || '-'}</td>
                <td>{row.ifsc_code}</td>
                <td>{row.account_type}</td>
                <td><span className={`badge ${row.verification_status}`}>{row.verification_status}</span></td>
                <td>{row.is_primary ? 'Yes' : '-'}</td>
              </tr>
            ))}
            {bankAccounts.length === 0 && (
              <tr><td colSpan={7}>No bank accounts found</td></tr>
            )}
          </tbody>
        </table>
      </section>
    </div>
  );
}
