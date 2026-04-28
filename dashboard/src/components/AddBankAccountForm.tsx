import { useState } from 'react';
import { X, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Props {
  onClose: () => void;
}

export default function AddBankAccountForm({ onClose }: Props) {
  const [formData, setFormData] = useState({
    account_holder_name: '',
    account_number: '',
    ifsc_code: '',
    account_type: 'savings' as const,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  const merchantId = localStorage.getItem('feelpay_merchant_id') || 'test-merchant';

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

    setLoading(true);

    try {
      await apiRequest(
        'POST',
        `/api/v1/merchants/${merchantId}/bank-accounts`,
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
          <h2>Add Bank Account</h2>
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
            <select
              value={formData.account_type}
              onChange={e => setFormData({ ...formData, account_type: e.target.value as any })}
            >
              <option value="savings">Savings</option>
              <option value="current">Current</option>
            </select>
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
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .modal-content {
          background: var(--bg-primary);
          border-radius: 8px;
          box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
          max-width: 500px;
          width: 90%;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px;
          border-bottom: 1px solid var(--border-color);
        }
        .modal-header h2 {
          margin: 0;
          font-size: 18px;
        }
        .btn-close {
          background: none;
          border: none;
          cursor: pointer;
          padding: 4px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--text-secondary);
        }
        .form {
          padding: 20px;
        }
        .form-group {
          margin-bottom: 16px;
        }
        .form-group label {
          display: block;
          margin-bottom: 6px;
          font-weight: 500;
          font-size: 14px;
        }
        .form-group input,
        .form-group select {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          font-size: 14px;
          font-family: inherit;
        }
        .error-text {
          display: block;
          color: #f00;
          font-size: 12px;
          margin-top: 4px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 24px;
        }
        .form-actions button {
          flex: 1;
        }
      `}</style>
    </div>
  );
}
