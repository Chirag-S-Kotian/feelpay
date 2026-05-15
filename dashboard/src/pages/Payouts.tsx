import { useEffect, useMemo, useState } from 'react';
import { AlertCircle, Landmark, RefreshCw, Send } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { formatPaise } from '../utils/money';

type Payout = {
  id: string;
  amount: number;
  currency: string;
  status: string;
  method: string;
  purpose?: string;
  reference?: string;
  bank_account_id?: string | null;
  payout_reference?: string | null;
  gateway_reference?: string | null;
  error_message?: string | null;
  processed_at?: string | null;
  created_at: string;
};

type BankAccount = {
  id: string;
  account_holder_name: string;
  account_number?: string;
  account_number_masked?: string;
  ifsc_code: string;
  bank_name?: string | null;
  is_verified?: boolean;
  verification_status?: string;
  is_primary?: boolean;
};

function money(amount: number, currency = 'INR') {
  return currency === 'INR' ? formatPaise(amount) : `${currency} ${formatPaise(amount, { symbol: false })}`;
}

function statusClass(status: string) {
  if (['success', 'processed', 'completed', 'paid'].includes(status)) return 'badge success';
  if (['failed', 'reversed'].includes(status)) return 'badge failed';
  if (['processing', 'queued', 'approved', 'on_hold', 'created'].includes(status)) return 'badge pending';
  return 'badge';
}

function accountLabel(account?: BankAccount) {
  if (!account) return 'Select bank account';
  const raw = account.account_number_masked || account.account_number || '';
  const suffix = raw ? raw.slice(-4).padStart(4, '*') : '----';
  return `${account.account_holder_name} • ${suffix} • ${account.ifsc_code}`;
}

export default function Payouts() {
  const [payouts, setPayouts] = useState<Payout[]>([]);
  const [accounts, setAccounts] = useState<BankAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const merchantId = useMemo(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  }, []);

  const accountById = useMemo(() => new Map(accounts.map((account) => [account.id, account])), [accounts]);
  const totalPaid = payouts
    .filter((payout) => ['success', 'processed', 'completed', 'paid'].includes(payout.status))
    .reduce((sum, payout) => sum + payout.amount, 0);
  const pendingCount = payouts.filter((payout) => ['created', 'approved', 'queued', 'processing', 'on_hold'].includes(payout.status)).length;
  const verifiedAccounts = accounts.filter((account) => account.verification_status === 'verified' || account.is_verified);

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      const [payoutRes, accountRes] = await Promise.all([
        apiRequest<Payout[]>('GET', '/v1/payouts', undefined, undefined, { cacheMs: 5_000 }),
        apiRequest<{ accounts?: BankAccount[] }>('GET', `/v1/merchants/${merchantId}/bank-accounts`, undefined, undefined, { cacheMs: 30_000 }),
      ]);
      setPayouts(payoutRes || []);
      const nextAccounts = accountRes.accounts || [];
      setAccounts(nextAccounts);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to load payouts.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Payouts</h1>
          <p>Track internal payout lifecycle and confirmed references.</p>
        </div>
        <button className="btn btn-secondary" onClick={() => void load()}>
          <RefreshCw size={16} /> Refresh
        </button>
      </div>

      {error && <div className="card payout-alert danger"><AlertCircle size={16} /> {error}</div>}

      <div className="payout-grid">
        <section className="card">
          <h3 className="section-title"><Send size={17} /> Payout Access</h3>
          {verifiedAccounts.length === 0 ? (
            <div className="payout-empty">
              <Landmark size={26} />
              <strong>Add verified bank account</strong>
              <p>No verified settlement account exists yet. Add a bank account before internal admins can queue payouts.</p>
              <a className="btn btn-primary" href="/bank-accounts">Add verified bank account</a>
            </div>
          ) : (
          <div className="payout-empty">
            <Landmark size={26} />
            <strong>Payout creation is internal-admin only</strong>
            <p>Merchant users can monitor queued, approved, processing, paid, failed, reversed, cancelled, and on-hold payouts here. Confirmed references appear only after an internal admin records a manual, test, or provider reference.</p>
          </div>
          )}
        </section>

        <section className="card payout-summary">
          <h3 className="section-title"><Landmark size={17} /> Payout Health</h3>
          <div className="payout-health-grid">
            <div className="payout-health-tile"><span>Total paid</span><strong>{money(totalPaid)}</strong></div>
            <div className="payout-health-tile"><span>Pending</span><strong>{pendingCount}</strong></div>
            <div className="payout-health-tile"><span>Verified accounts</span><strong>{verifiedAccounts.length}</strong></div>
            <div className="payout-health-tile"><span>Failed or reversed</span><strong>{payouts.filter((payout) => ['failed', 'reversed'].includes(payout.status)).length}</strong></div>
          </div>
        </section>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        {loading ? (
          <div className="loading">Loading payouts...</div>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Payout ID</th>
                <th>Amount</th>
                <th>Method</th>
                <th>Bank Account</th>
                <th>Status</th>
                <th>Reference</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {payouts.map((payout) => (
                <tr key={payout.id}>
                  <td className="mono">{payout.id.slice(0, 18)}...</td>
                  <td>{money(payout.amount, payout.currency)}</td>
                  <td>{payout.method.toUpperCase()}</td>
                  <td>{accountLabel(accountById.get(payout.bank_account_id || ''))}</td>
                  <td><span className={statusClass(payout.status)}>{payout.status}</span></td>
                  <td>{payout.gateway_reference || payout.payout_reference || '-'}</td>
                  <td>{new Date(payout.created_at).toLocaleString('en-IN')}</td>
                </tr>
              ))}
              {payouts.length === 0 && (
                <tr><td colSpan={7}>No payouts yet.</td></tr>
              )}
            </tbody>
          </table>
        )}
      </div>

      <style>{`
        .payout-grid { display: grid; grid-template-columns: minmax(0, 1.35fr) minmax(280px, 0.65fr); gap: 16px; }
        .payout-alert { margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }
        .payout-alert.success { border-color: rgba(34, 197, 94, 0.4); color: #9af0b8; }
        .payout-alert.danger { border-color: rgba(239, 68, 68, 0.42); color: #fecaca; }
        .payout-health-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
        .payout-health-tile {
          border: 1px solid var(--fp-border);
          background: var(--fp-bg-elevated);
          border-radius: 10px;
          padding: 10px;
          display: grid;
          gap: 5px;
        }
        .payout-health-tile span { font-size: 12px; color: var(--fp-text-muted); }
        .payout-health-tile strong { font-size: 22px; line-height: 1.1; color: var(--fp-text); }
        .payout-empty { min-height: 210px; display: grid; align-content: center; justify-items: start; gap: 10px; color: var(--fp-text-muted); }
        .payout-empty strong { color: var(--fp-text); font-size: 18px; }
        .mono { font-family: Fira Code, monospace; font-size: 12px; }
        @media (max-width: 900px) { .payout-grid, .payout-health-grid { grid-template-columns: 1fr; } }
      `}</style>
    </div>
  );
}
