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
  const [onboardingStatus, setOnboardingStatus] = useState<string | null>(null);

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
      const [payoutRes, accountRes, onboardingRes] = await Promise.all([
        apiRequest<Payout[]>('GET', '/v1/payouts', undefined, undefined, { cacheMs: 5_000 }).catch((err) => {
          if (err instanceof Error && err.message.includes('403')) {
            return [];
          }
          throw err;
        }),
        apiRequest<{ accounts?: BankAccount[] } | BankAccount[]>('GET', `/v1/merchants/${merchantId}/bank-accounts`, undefined, undefined, { cacheMs: 30_000 }),
        apiRequest<{ onboarding_status: string }>('GET', '/v1/onboarding/status', undefined, undefined, { cacheMs: 30_000 }).catch(() => ({ onboarding_status: 'pending' })),
      ]);
      
      setPayouts(payoutRes || []);
      
      let nextAccounts: BankAccount[] = [];
      if (Array.isArray(accountRes)) {
        nextAccounts = accountRes;
      } else if (accountRes && 'accounts' in accountRes) {
        nextAccounts = accountRes.accounts || [];
      } else if (accountRes && 'bank_accounts' in accountRes) {
        nextAccounts = (accountRes as any).bank_accounts || [];
      }
      setAccounts(nextAccounts);
      setOnboardingStatus(onboardingRes?.onboarding_status || 'pending');
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

      {onboardingStatus !== 'approved' && !loading && onboardingStatus !== null ? (
        <div className="kyc-warning-card">
          <div className="kyc-warning-blur"></div>
          <div className="kyc-warning-content">
            <div className="kyc-icon-wrapper">
              <AlertCircle size={36} className="kyc-warning-icon" />
            </div>
            <h2>Settlement Verification Pending</h2>
            <p className="kyc-warning-subtitle">
              Your FeelPay merchant account is in the <strong>{onboardingStatus}</strong> verification state.
            </p>
            <p className="kyc-warning-description">
              To comply with financial regulations and secure our merchant ecosystem, payouts are strictly disabled until your KYC profile is fully approved by our onboarding team. You can continue to link and configure your bank accounts securely in the meantime.
            </p>
            <div className="kyc-actions">
              <a href="/onboarding" className="btn btn-kyc-action">
                Complete Onboarding Profile
              </a>
              <a href="/bank-accounts" className="btn btn-kyc-secondary">
                <Landmark size={16} /> Manage Bank Accounts
              </a>
            </div>
          </div>
        </div>
      ) : (
        <>
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
        </>
      )}

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
        
        .kyc-warning-card {
          position: relative;
          background: rgba(255, 255, 255, 0.02);
          backdrop-filter: blur(20px);
          -webkit-backdrop-filter: blur(20px);
          border: 1px solid rgba(245, 158, 11, 0.25);
          box-shadow: 0 20px 50px rgba(0, 0, 0, 0.35);
          border-radius: 20px;
          padding: 48px 32px;
          text-align: center;
          max-width: 720px;
          margin: 40px auto;
          overflow: hidden;
          animation: fadeInUp 0.6s cubic-bezier(0.16, 1, 0.3, 1);
        }
        @keyframes fadeInUp {
          from { opacity: 0; transform: translateY(24px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .kyc-warning-blur {
          position: absolute;
          top: -50%;
          left: -50%;
          width: 200%;
          height: 200%;
          background: radial-gradient(circle, rgba(245, 158, 11, 0.07) 0%, transparent 60%);
          z-index: 1;
          pointer-events: none;
        }
        .kyc-warning-content {
          position: relative;
          z-index: 2;
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
        }
        .kyc-icon-wrapper {
          width: 76px;
          height: 76px;
          border-radius: 50%;
          background: rgba(245, 158, 11, 0.1);
          display: flex;
          align-items: center;
          justify-content: center;
          border: 1px solid rgba(245, 158, 11, 0.25);
          box-shadow: 0 0 30px rgba(245, 158, 11, 0.12);
          margin-bottom: 8px;
        }
        .kyc-warning-icon {
          color: #f59e0b;
          animation: pulse 2s infinite ease-in-out;
        }
        @keyframes pulse {
          0%, 100% { transform: scale(1); }
          50% { transform: scale(1.05); }
        }
        .kyc-warning-card h2 {
          font-size: 24px;
          font-weight: 700;
          color: #fff;
          margin: 0;
          letter-spacing: -0.02em;
        }
        .kyc-warning-subtitle {
          font-size: 15px;
          color: rgba(255, 255, 255, 0.7);
          margin: 0;
        }
        .kyc-warning-subtitle strong {
          color: #f59e0b;
          text-transform: uppercase;
          background: rgba(245, 158, 11, 0.1);
          padding: 3px 8px;
          border-radius: 4px;
          font-size: 13px;
          letter-spacing: 0.05em;
          border: 1px solid rgba(245, 158, 11, 0.2);
        }
        .kyc-warning-description {
          font-size: 14px;
          line-height: 1.6;
          color: rgba(255, 255, 255, 0.55);
          max-width: 580px;
          margin: 0 auto;
        }
        .kyc-actions {
          display: flex;
          gap: 16px;
          margin-top: 16px;
        }
        .btn-kyc-action {
          background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
          color: #000 !important;
          font-weight: 600;
          padding: 12px 24px;
          border-radius: 10px;
          font-size: 14px;
          border: none;
          cursor: pointer;
          transition: all 0.2s ease;
          box-shadow: 0 4px 15px rgba(245, 158, 11, 0.3);
          text-decoration: none;
        }
        .btn-kyc-action:hover {
          transform: translateY(-2px);
          box-shadow: 0 6px 20px rgba(245, 158, 11, 0.45);
          background: linear-gradient(135deg, #fbbf24 0%, #ea580c 100%);
        }
        .btn-kyc-secondary {
          background: rgba(255, 255, 255, 0.05);
          color: #fff !important;
          font-weight: 500;
          padding: 12px 24px;
          border-radius: 10px;
          font-size: 14px;
          border: 1px solid rgba(255, 255, 255, 0.1);
          cursor: pointer;
          transition: all 0.2s ease;
          text-decoration: none;
          display: flex;
          align-items: center;
          gap: 8px;
        }
        .btn-kyc-secondary:hover {
          background: rgba(255, 255, 255, 0.08);
          border-color: rgba(255, 255, 255, 0.2);
          transform: translateY(-2px);
        }
        
        @media (max-width: 900px) {
          .payout-grid, .payout-health-grid { grid-template-columns: 1fr; }
          .kyc-actions { flex-direction: column; width: 100%; }
          .btn-kyc-action, .btn-kyc-secondary { justify-content: center; }
        }
      `}</style>
    </div>
  );
}
