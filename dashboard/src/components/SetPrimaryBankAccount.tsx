import { useState } from 'react';
import { X, AlertCircle, CheckCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Props {
  accountId: string;
  onClose: () => void;
}

export default function SetPrimaryBankAccount({ accountId, onClose }: Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  const handleSetPrimary = async () => {
    setLoading(true);
    setError(null);

    try {
      await apiRequest(
        'PUT',
        `/v1/merchants/${merchantId}/bank-accounts/${accountId}/set-primary`,
        {}
      );
      setSuccess(true);
      setTimeout(onClose, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set primary account');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Set as Primary Account</h2>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="modal-body">
          {error && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {error}
            </div>
          )}

          {success && (
            <div className="alert alert-success">
              <CheckCircle size={16} />
              Account set as primary successfully!
            </div>
          )}

          {!success && (
            <>
              <p>
                This account will be used as the default settlement destination for all future payouts.
              </p>
              <p className="warning-text">
                This action cannot be undone immediately. You can change it later if needed.
              </p>

              <div className="form-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={onClose}
                  disabled={loading}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  className="btn-primary"
                  onClick={handleSetPrimary}
                  disabled={loading}
                >
                  {loading ? 'Setting...' : 'Set as Primary'}
                </button>
              </div>
            </>
          )}
        </div>
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
          max-width: 400px;
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
        .modal-body {
          padding: 20px;
        }
        .modal-body p {
          margin: 0 0 12px 0;
          font-size: 14px;
          line-height: 1.5;
        }
        .warning-text {
          color: var(--text-secondary);
          font-size: 13px;
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
