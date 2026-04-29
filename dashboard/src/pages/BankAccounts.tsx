import { useState, useEffect } from 'react';
import { Plus, Trash2, CheckCircle, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import AddBankAccountForm from '../components/AddBankAccountForm';
import PennyDropVerification from '../components/PennyDropVerification';
import SetPrimaryBankAccount from '../components/SetPrimaryBankAccount';

interface BankAccount {
  id: string;
  account_holder_name: string;
  account_number_masked: string;
  ifsc_code: string;
  account_type: 'savings' | 'current';
  verification_status: 'pending' | 'verified' | 'failed';
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

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  useEffect(() => {
    fetchBankAccounts();
  }, []);

  const fetchBankAccounts = async () => {
    try {
      const data = await apiRequest<{ bank_accounts: BankAccount[] }>(
        'GET',
        `/v1/merchants/${merchantId}/bank-accounts`
      );
      setAccounts(data.bank_accounts || []);
    } catch (err) {
      console.error('Failed to fetch bank accounts:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (accountId: string) => {
    if (!confirm('Delete this bank account?')) return;

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
          <div className="empty-state">
            <AlertCircle size={32} />
            <p>No bank accounts configured</p>
            <button
              className="btn-primary"
              onClick={() => setShowAddForm(true)}
            >
              Add First Account
            </button>
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
                    <span>IFSC:</span>
                    <strong className="mono">{account.ifsc_code}</strong>
                  </div>
                  <div className="detail-row">
                    <span>Type:</span>
                    <strong>{account.account_type}</strong>
                  </div>
                  <div className="detail-row">
                    <span>Status:</span>
                    <span className={`badge badge-${account.verification_status}`}>
                      {account.verification_status === 'verified' && <CheckCircle size={12} />}
                      {account.verification_status === 'failed' && <AlertCircle size={12} />}
                      {account.verification_status}
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
        .accounts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
          gap: 16px;
        }
        .account-card {
          border: 1px solid var(--border-color);
          border-radius: 8px;
          padding: 16px;
          background: var(--bg-secondary);
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
          color: var(--text-secondary);
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
          color: var(--text-secondary);
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
          background: #fee;
          border: 1px solid #f00;
          color: #f00;
          border-radius: 4px;
          cursor: pointer;
          padding: 8px 12px;
          font-size: 13px;
          transition: all 0.2s;
        }
        .btn-danger:hover {
          background: #fdd;
        }
      `}</style>
    </div>
  );
}
