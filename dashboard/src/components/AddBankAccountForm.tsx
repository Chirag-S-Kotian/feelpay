import { useState } from 'react';
import { X, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Props {
  onClose: () => void;
}

export default function AddBankAccountForm({ onClose }: Props) {
  type AccountType = 'savings' | 'current';
  const [formData, setFormData] = useState({
    account_holder_name: '',
    bank_name: '',
    account_number: '',
    confirm_account_number: '',
    ifsc_code: '',
    account_type: 'savings' as AccountType,
    currency: 'INR',
    is_primary: false,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  const validate = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.account_holder_name.trim()) {
      newErrors.account_holder_name = 'Account holder name is required';
    }

    if (!formData.account_number.trim()) {
      newErrors.account_number = 'Account number is required';
    } else if (!/^\d{9,18}$/.test(formData.account_number)) {
      newErrors.account_number = 'Invalid account number format';
    }
    if (formData.confirm_account_number !== formData.account_number) {
      newErrors.confirm_account_number = 'Account numbers do not match';
    }

    if (!formData.ifsc_code.trim()) {
      newErrors.ifsc_code = 'IFSC code is required';
    } else if (!/^[A-Z]{4}0[A-Z0-9]{6}$/.test(formData.ifsc_code)) {
      newErrors.ifsc_code = 'Invalid IFSC code format';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;
    if (!merchantId) {
      setErrors({ submit: 'Merchant session missing. Login again.' });
      return;
    }

    setLoading(true);

    try {
      await apiRequest(
        'POST',
        `/v1/merchants/${merchantId}/bank-accounts`,
        formData
      );
      onClose();
    } catch (err) {
      setErrors({
        submit: err instanceof Error ? err.message : 'Failed to add bank account',
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="ba-modal-overlay" onClick={onClose}>
      <div className="ba-modal-content" onClick={e => e.stopPropagation()}>
        <div className="ba-modal-header">
          <div>
            <div className="ba-modal-kicker">Settlement Setup</div>
            <h2>Add Bank Account</h2>
          </div>
          <button className="ba-btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="ba-form">
          {errors.submit && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {errors.submit}
            </div>
          )}

          <div className="ba-form-group">
            <label>Account Holder Name *</label>
            <input
              type="text"
              value={formData.account_holder_name}
              onChange={e => setFormData({ ...formData, account_holder_name: e.target.value })}
              placeholder="John Doe"
            />
            {errors.account_holder_name && (
              <span className="error-text">{errors.account_holder_name}</span>
            )}
          </div>

          <div className="ba-form-group">
            <label>Account Number *</label>
            <input
              type="text"
              inputMode="numeric"
              autoComplete="off"
              value={formData.account_number}
              onChange={e => setFormData({ ...formData, account_number: e.target.value })}
              placeholder="1234567890123456"
            />
            {errors.account_number && (
              <span className="error-text">{errors.account_number}</span>
            )}
          </div>

          <div className="ba-form-group">
            <label>Confirm Account Number *</label>
            <input
              type="text"
              inputMode="numeric"
              autoComplete="off"
              value={formData.confirm_account_number}
              onChange={e => setFormData({ ...formData, confirm_account_number: e.target.value })}
              placeholder="Re-enter account number"
            />
            {errors.confirm_account_number && (
              <span className="error-text">{errors.confirm_account_number}</span>
            )}
          </div>

          <div className="ba-form-group">
            <label>Bank Name</label>
            <input
              type="text"
              value={formData.bank_name}
              onChange={e => setFormData({ ...formData, bank_name: e.target.value })}
              placeholder="HDFC Bank"
            />
          </div>

          <div className="ba-form-group">
            <label>IFSC Code *</label>
            <input
              type="text"
              value={formData.ifsc_code}
              onChange={e => setFormData({ ...formData, ifsc_code: e.target.value.toUpperCase() })}
              placeholder="HDFC0001234"
            />
            {errors.ifsc_code && (
              <span className="error-text">{errors.ifsc_code}</span>
            )}
          </div>

          <div className="ba-form-group">
            <label>Account Type *</label>
            <div className="ba-account-type-toggle" role="group" aria-label="Account type">
              <button
                type="button"
                className={`ba-type-btn ${formData.account_type === 'savings' ? 'active' : ''}`}
                onClick={() => setFormData({ ...formData, account_type: 'savings' })}
              >
                Savings
              </button>
              <button
                type="button"
                className={`ba-type-btn ${formData.account_type === 'current' ? 'active' : ''}`}
                onClick={() => setFormData({ ...formData, account_type: 'current' })}
              >
                Current
              </button>
            </div>
          </div>

          <div className="ba-form-group">
            <label>Currency</label>
            <select value={formData.currency} onChange={e => setFormData({ ...formData, currency: e.target.value })}>
              <option value="INR">INR</option>
            </select>
          </div>

          <label className="ba-checkbox">
            <input
              type="checkbox"
              checked={formData.is_primary}
              onChange={e => setFormData({ ...formData, is_primary: e.target.checked })}
            />
            Make this the primary settlement account after verification
          </label>

          <div className="ba-sandbox-note">
            Test mode uses sandbox verification only. No real bank call or money movement is initiated.
          </div>

          <div className="ba-form-actions">
            <button type="button" className="btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? 'Adding...' : 'Add Account'}
            </button>
          </div>
        </form>
      </div>

      <style>{`
        .ba-modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(4, 10, 24, 0.78);
          backdrop-filter: blur(5px);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2200;
          padding: 16px;
        }
        .ba-modal-content {
          background: #fff9ec;
          border: 1px solid #d5c9af;
          border-radius: 18px;
          box-shadow: 0 18px 64px rgba(0, 0, 0, 0.28);
          max-width: 620px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
        }
        .ba-modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 22px;
          border-bottom: 1px solid #d5c9af;
        }
        .ba-modal-kicker {
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.16em;
          font-weight: 700;
          color: #6b6862;
          margin-bottom: 4px;
        }
        .ba-modal-header h2 {
          margin: 0;
          font-size: 26px;
          line-height: 1.05;
          font-weight: 800;
          color: #141413;
        }
        .ba-btn-close {
          background: #2d4a73;
          border: 1px solid #2d4a73;
          border-radius: 10px;
          cursor: pointer;
          padding: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: #fff9ec;
        }
        .ba-btn-close:hover {
          filter: brightness(1.08);
        }
        .ba-form {
          padding: 20px 22px 24px;
          display: flex;
          flex-direction: column;
          gap: 14px;
        }
        .ba-form-group label {
          display: block;
          margin-bottom: 7px;
          font-weight: 700;
          font-size: 13px;
          color: #141413;
          letter-spacing: 0.02em;
        }
        .ba-form-group input {
          width: 100%;
          padding: 11px 13px;
          border: 1px solid #d5c9af;
          border-radius: 12px;
          font-size: 14px;
          font-family: inherit;
          color: #141413;
          background: #fff9ec;
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .ba-form-group input:focus {
          border-color: #eb001b;
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }
        .ba-form-group input::placeholder {
          color: #8a857d;
        }
        .ba-account-type-toggle {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 10px;
          padding: 5px;
          border: 1px solid #d5c9af;
          border-radius: 12px;
          background: #f5efe2;
        }
        .ba-type-btn {
          border: 1px solid #d5c9af;
          background: #fff9ec;
          color: #6b6862;
          border-radius: 9px;
          min-height: 42px;
          font-size: 14px;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s ease;
        }
        .ba-type-btn:hover {
          border-color: #2d4a73;
          color: #141413;
        }
        .ba-type-btn.active {
          background: linear-gradient(135deg, #f8a429, #9f5cff);
          color: #fff;
          box-shadow: 0 4px 18px rgba(120, 74, 203, 0.38);
          border-color: rgba(255, 215, 155, 0.24);
        }
        .error-text {
          display: block;
          color: #ffabb4;
          font-size: 12px;
          margin-top: 6px;
        }
        .ba-form-actions {
          display: flex;
          gap: 12px;
          margin-top: 10px;
          padding-top: 2px;
        }
        .ba-form-actions button {
          flex: 1;
          min-height: 42px;
        }
        .alert.alert-error {
          display: flex;
          align-items: center;
          gap: 8px;
          border: 1px solid rgba(247, 83, 96, 0.42);
          background: rgba(247, 83, 96, 0.14);
          color: #ffb2ba;
          border-radius: 10px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
}
