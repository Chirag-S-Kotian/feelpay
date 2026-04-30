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
    account_number: '',
    ifsc_code: '',
    account_type: 'savings' as AccountType,
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
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div>
            <div className="modal-kicker">Settlement Setup</div>
            <h2>Add Bank Account</h2>
          </div>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="form">
          {errors.submit && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {errors.submit}
            </div>
          )}

          <div className="form-group">
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

          <div className="form-group">
            <label>Account Number *</label>
            <input
              type="text"
              value={formData.account_number}
              onChange={e => setFormData({ ...formData, account_number: e.target.value })}
              placeholder="1234567890123456"
            />
            {errors.account_number && (
              <span className="error-text">{errors.account_number}</span>
            )}
          </div>

          <div className="form-group">
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

          <div className="form-group">
            <label>Account Type *</label>
            <div className="account-type-toggle" role="group" aria-label="Account type">
              <button
                type="button"
                className={`type-btn ${formData.account_type === 'savings' ? 'active' : ''}`}
                onClick={() => setFormData({ ...formData, account_type: 'savings' })}
              >
                Savings
              </button>
              <button
                type="button"
                className={`type-btn ${formData.account_type === 'current' ? 'active' : ''}`}
                onClick={() => setFormData({ ...formData, account_type: 'current' })}
              >
                Current
              </button>
            </div>
          </div>

          <div className="form-actions">
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
        .modal-overlay {
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
        .modal-content {
          background:
            radial-gradient(140% 170% at 8% -2%, rgba(91, 154, 247, 0.22), transparent 48%),
            radial-gradient(130% 160% at 100% 0%, rgba(245, 158, 11, 0.14), transparent 46%),
            linear-gradient(180deg, rgba(13, 31, 61, 0.98), rgba(10, 24, 46, 0.98));
          border: 1px solid rgba(107, 168, 255, 0.34);
          border-radius: 18px;
          box-shadow: 0 18px 64px rgba(0, 0, 0, 0.46);
          max-width: 620px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 22px;
          border-bottom: 1px solid rgba(107, 168, 255, 0.24);
        }
        .modal-kicker {
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.16em;
          font-weight: 700;
          color: #9bc3ff;
          margin-bottom: 4px;
        }
        .modal-header h2 {
          margin: 0;
          font-size: 26px;
          line-height: 1.05;
          font-weight: 800;
          color: #f0f6ff;
        }
        .btn-close {
          background: rgba(14, 34, 63, 0.9);
          border: 1px solid rgba(91, 154, 247, 0.35);
          border-radius: 10px;
          cursor: pointer;
          padding: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: #c4dcff;
        }
        .btn-close:hover {
          border-color: rgba(127, 186, 255, 0.72);
          color: #eef5ff;
        }
        .form {
          padding: 20px 22px 24px;
          display: flex;
          flex-direction: column;
          gap: 14px;
        }
        .form-group label {
          display: block;
          margin-bottom: 7px;
          font-weight: 700;
          font-size: 13px;
          color: #c6dcfb;
          letter-spacing: 0.02em;
        }
        .form-group input {
          width: 100%;
          padding: 11px 13px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          border-radius: 12px;
          font-size: 14px;
          font-family: inherit;
          color: #eaf3ff;
          background: linear-gradient(180deg, rgba(17, 41, 77, 0.94), rgba(12, 29, 56, 0.94));
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .form-group input:focus {
          border-color: rgba(122, 182, 255, 0.95);
          box-shadow: 0 0 0 3px rgba(77, 141, 235, 0.2);
        }
        .form-group input::placeholder {
          color: #93b2de;
        }
        .account-type-toggle {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 10px;
          padding: 5px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          border-radius: 12px;
          background: rgba(12, 29, 56, 0.7);
        }
        .type-btn {
          border: 1px solid rgba(97, 162, 255, 0.24);
          background: rgba(18, 46, 84, 0.6);
          color: #b8d3f6;
          border-radius: 9px;
          min-height: 42px;
          font-size: 14px;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.2s ease;
        }
        .type-btn:hover {
          border-color: rgba(127, 186, 255, 0.52);
          color: #e8f2ff;
        }
        .type-btn.active {
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
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 10px;
          padding-top: 2px;
        }
        .form-actions button {
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
