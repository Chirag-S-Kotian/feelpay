import { useState, useEffect, useMemo } from 'react';
import { Plus, Trash2, CheckCircle, AlertCircle, ShieldCheck, Landmark } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import AddBankAccountForm from '../components/AddBankAccountForm';
import PennyDropVerification from '../components/PennyDropVerification';
import SetPrimaryBankAccount from '../components/SetPrimaryBankAccount';

interface BankAccount {
  id: string;
  account_holder_name: string;
  account_number_masked: string;
  bank_name?: string | null;
  ifsc_code: string;
  account_type: 'savings' | 'current';
  currency?: string;
  verification_status: 'pending' | 'verified' | 'failed' | 'deactivated';
  is_primary: boolean;
  created_at: string;
  verified_at?: string;
}

export default function BankAccounts() {
  const [accounts, setAccounts] = useState<BankAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [verifyingAccountId, setVerifyingAccountId] = useState<string | null>(null);
  const [settingPrimaryId, setSettingPrimaryId] = useState<string | null>(null);

  const merchantId = useMemo(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  }, []);

  useEffect(() => {
    fetchBankAccounts();
  }, []);

  const fetchBankAccounts = async () => {
    if (!merchantId) {
      setLoading(false);
      return;
    }
    try {
      const data = await apiRequest<{ bank_accounts?: BankAccount[]; accounts?: BankAccount[] }>(
        'GET',
        `/v1/merchants/${merchantId}/bank-accounts`,
        undefined,
        undefined,
        { cacheMs: 45_000 }
      );
      setAccounts(data.bank_accounts || data.accounts || []);
    } catch (err) {
      console.error('Failed to fetch bank accounts:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (accountId: string) => {
    if (!confirm('Deactivate this bank account? Existing payout history remains available.')) return;

    try {
      await apiRequest('DELETE', `/v1/merchants/${merchantId}/bank-accounts/${accountId}`);
      setAccounts(accounts.filter(a => a.id !== accountId));
    } catch (err) {
      console.error('Failed to delete account:', err);
    }
  };

  const handleFormClose = () => {
    setShowAddForm(false);
    fetchBankAccounts();
  };

  const handleVerifyClose = () => {
    setVerifyingAccountId(null);
    fetchBankAccounts();
  };

  const handleSetPrimaryClose = () => {
    setSettingPrimaryId(null);
    fetchBankAccounts();
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">Loading bank accounts...</div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Bank Account Management</h1>
        <p>Manage settlement bank accounts and verification</p>
      </div>

      {showAddForm && <AddBankAccountForm onClose={handleFormClose} />}

      {verifyingAccountId && (
        <PennyDropVerification
          accountId={verifyingAccountId}
          onClose={handleVerifyClose}
        />
      )}

      {settingPrimaryId && (
        <SetPrimaryBankAccount
          accountId={settingPrimaryId}
          onClose={handleSetPrimaryClose}
        />
      )}

      <div className="card">
        <div className="card-header">
          <h2>Bank Accounts</h2>
          <button
            className="btn-primary"
            onClick={() => setShowAddForm(true)}
          >
            <Plus size={16} />
            Add Bank Account
          </button>
        </div>

        {accounts.length === 0 ? (
          <div className="bank-empty">
            <div className="bank-empty-left">
              <div className="bank-empty-icon">
                <Landmark size={28} />
              </div>
              <h3>No bank accounts configured</h3>
              <p>Add and verify a sandbox bank account to unlock test payouts, settlements, and go-live readiness.</p>
              <div className="bank-empty-cta">
                <button
                  className="btn-primary"
                  onClick={() => setShowAddForm(true)}
                >
                  <Plus size={16} />
                  Add First Account
                </button>
              </div>
            </div>
            <div className="bank-empty-right">
              <div className="bank-empty-check">
                <ShieldCheck size={16} />
                Verification with penny-drop validation
              </div>
              <div className="bank-empty-check">
                <ShieldCheck size={16} />
                Primary account failover controls
              </div>
              <div className="bank-empty-check">
                <ShieldCheck size={16} />
                Full payout audit trail support
              </div>
            </div>
          </div>
        ) : (
          <div className="accounts-grid">
            {accounts.map(account => (
              <div key={account.id} className="account-card">
                <div className="account-header">
                  <div>
                    <h3>{account.account_holder_name}</h3>
                    <p className="account-number">{account.account_number_masked}</p>
                  </div>
                  {account.is_primary && (
                    <span className="badge badge-primary">Primary</span>
                  )}
                </div>

                <div className="account-details">
                  <div className="detail-row">
                    <span>Bank:</span>
                    <strong>{account.bank_name || '-'}</strong>
                  </div>
                  <div className="detail-row">
                    <span>IFSC:</span>
                    <strong className="mono">{account.ifsc_code}</strong>
                  </div>
                  <div className="detail-row">
                    <span>Type:</span>
                    <strong>{account.account_type}</strong>
                  </div>
                  <div className="detail-row">
                    <span>Currency:</span>
                    <strong>{account.currency || 'INR'}</strong>
                  </div>
                  <div className="detail-row">
                    <span>Status:</span>
                    <span className={`badge badge-${account.verification_status}`}>
                      {account.verification_status === 'verified' && <CheckCircle size={12} />}
                      {account.verification_status === 'failed' && <AlertCircle size={12} />}
                      {account.verification_status === 'verified' ? 'sandbox verified' : account.verification_status}
                    </span>
                  </div>
                  {account.verified_at && (
                    <div className="detail-row">
                      <span>Verified:</span>
                      <span>{new Date(account.verified_at).toLocaleDateString()}</span>
                    </div>
                  )}
                </div>

                <div className="account-actions">
                  {account.verification_status === 'pending' && (
                    <button
                      className="btn-secondary"
                      onClick={() => setVerifyingAccountId(account.id)}
                    >
                      Verify
                    </button>
                  )}
                  {!account.is_primary && account.verification_status === 'verified' && (
                    <button
                      className="btn-secondary"
                      onClick={() => setSettingPrimaryId(account.id)}
                    >
                      Set as Primary
                    </button>
                  )}
                  <button
                    className="btn-danger"
                    onClick={() => handleDelete(account.id)}
                    aria-label="Deactivate bank account"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .card {
          border-radius: 16px;
          border: 1px solid rgba(98, 160, 250, 0.3);
          background:
            radial-gradient(150% 190% at 0% 0%, rgba(91, 154, 247, 0.15), transparent 42%),
            linear-gradient(180deg, rgba(17, 39, 72, 0.96), rgba(10, 25, 47, 0.96));
          overflow: hidden;
        }
        .card-header h2 {
          font-size: 32px;
          line-height: 1.06;
          letter-spacing: -0.02em;
        }
        .bank-empty {
          min-height: 260px;
          display: grid;
          grid-template-columns: 1.2fr 1fr;
          gap: 24px;
          padding: 8px 4px 2px;
        }
        .bank-empty-left {
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          justify-content: center;
          gap: 10px;
        }
        .bank-empty-icon {
          width: 54px;
          height: 54px;
          border-radius: 14px;
          display: grid;
          place-items: center;
          color: #e8f2ff;
          border: 1px solid rgba(114, 174, 255, 0.44);
          background: linear-gradient(135deg, rgba(245, 158, 11, 0.32), rgba(139, 92, 246, 0.34));
        }
        .bank-empty h3 {
          font-size: 24px;
          line-height: 1.05;
          margin: 4px 0 0;
          letter-spacing: -0.02em;
        }
        .bank-empty p {
          color: #a4bfdf;
          font-size: 14px;
          max-width: 440px;
        }
        .bank-empty-cta {
          margin-top: 4px;
        }
        .bank-empty-right {
          border: 1px solid rgba(103, 165, 255, 0.24);
          background: rgba(14, 34, 63, 0.62);
          border-radius: 14px;
          padding: 14px;
          display: flex;
          flex-direction: column;
          justify-content: center;
          gap: 10px;
        }
        .bank-empty-check {
          display: flex;
          align-items: center;
          gap: 8px;
          color: #c3daf9;
          font-size: 13px;
          line-height: 1.35;
        }
        .bank-empty-check svg {
          flex: 0 0 auto;
          color: #7fd6a6;
        }
        .accounts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
          gap: 16px;
        }
        .account-card {
          border: 1px solid rgba(95, 158, 250, 0.28);
          border-radius: 12px;
          padding: 16px;
          background: linear-gradient(180deg, rgba(16, 38, 70, 0.86), rgba(10, 24, 45, 0.86));
          display: flex;
          flex-direction: column;
          gap: 12px;
        }
        .account-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 12px;
        }
        .account-header h3 {
          margin: 0;
          font-size: 16px;
        }
        .account-number {
          margin: 4px 0 0 0;
          font-size: 13px;
          color: #9fbcdf;
          font-family: monospace;
        }
        .account-details {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px 0;
          border-top: 1px solid var(--border-color);
          border-bottom: 1px solid var(--border-color);
        }
        .detail-row {
          display: flex;
          justify-content: space-between;
          font-size: 13px;
        }
        .detail-row span:first-child {
          color: #9fbcdf;
        }
        .account-actions {
          display: flex;
          gap: 8px;
        }
        .account-actions button {
          flex: 1;
          padding: 8px 12px;
          font-size: 13px;
        }
        .btn-danger {
          background: rgba(247, 83, 96, 0.16);
          border: 1px solid rgba(247, 83, 96, 0.45);
          color: #ffb1b8;
          border-radius: 10px;
          cursor: pointer;
          padding: 8px 12px;
          font-size: 13px;
          transition: all 0.2s;
        }
        .btn-danger:hover {
          background: rgba(247, 83, 96, 0.24);
        }
        @media (max-width: 900px) {
          .bank-empty {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
}
