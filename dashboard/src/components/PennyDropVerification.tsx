import { useState, useEffect } from 'react';
import { X, AlertCircle, CheckCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Props {
  accountId: string;
  onClose: () => void;
}

interface VerificationData {
  penny_drop_reference: string;
  penny_drop_amount_paise: number;
}

export default function PennyDropVerification({ accountId, onClose }: Props) {
  const [verificationData, setVerificationData] = useState<VerificationData | null>(null);
  const [verifyAmount, setVerifyAmount] = useState('');
  const [loading, setLoading] = useState(true);
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const merchantId = localStorage.getItem('feelpay_merchant_id') || 'test-merchant';

  useEffect(() => {
    fetchVerificationData();
  }, []);

  const fetchVerificationData = async () => {
    try {
      const data = await apiRequest<VerificationData>(
        'GET',
        `/api/v1/merchants/${merchantId}/bank-accounts/${accountId}/verification-data`
      );
      setVerificationData(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load verification data');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!verifyAmount) {
      setError('Please enter the penny drop amount');
      return;
    }

    setVerifying(true);
    setError(null);

    try {
      await apiRequest(
        'POST',
        `/api/v1/merchants/${merchantId}/bank-accounts/${accountId}/verify`,
        {
          penny_drop_amount_paise: parseInt(verifyAmount),
        }
      );
      setSuccess(true);
      setTimeout(onClose, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed');
    } finally {
      setVerifying(false);
    }
  };

  if (loading) {
    return (
      <div className="modal-overlay" onClick={onClose}>
        <div className="modal-content" onClick={e => e.stopPropagation()}>
          <div className="loading">Loading verification data...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Verify Bank Account</h2>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleVerify} className="form">
          {error && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {error}
            </div>
          )}

          {success && (
            <div className="alert alert-success">
              <CheckCircle size={16} />
              Account verified successfully!
            </div>
          )}

          <div className="verification-info">
            <h3>Penny Drop Verification</h3>
            <p>
              A small amount (₹1-2) has been deposited to your bank account. Enter the exact amount
              you received to verify the account.
            </p>

            {verificationData && (
              <div className="info-box">
                <div className="info-row">
                  <span>Reference Code:</span>
                  <strong className="mono">{verificationData.penny_drop_reference}</strong>
                </div>
                <div className="info-row">
                  <span>Expected Amount:</span>
                  <strong>₹{(verificationData.penny_drop_amount_paise / 100).toFixed(2)}</strong>
                </div>
              </div>
            )}
          </div>

          <div className="form-group">
            <label>Enter Penny Drop Amount (in paise) *</label>
            <input
              type="number"
              value={verifyAmount}
              onChange={e => setVerifyAmount(e.target.value)}
              placeholder="e.g., 150 for ₹1.50"
              disabled={success}
            />
            <small>Enter the amount in paise (e.g., 150 for ₹1.50)</small>
          </div>

          <div className="form-actions">
            <button type="button" className="btn-secondary" onClick={onClose} disabled={success}>
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={verifying || success}>
              {verifying ? 'Verifying...' : 'Verify Account'}
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
        .verification-info {
          margin-bottom: 20px;
          padding: 16px;
          background: var(--bg-secondary);
          border-radius: 8px;
        }
        .verification-info h3 {
          margin: 0 0 8px 0;
          font-size: 14px;
        }
        .verification-info p {
          margin: 0 0 12px 0;
          font-size: 13px;
          color: var(--text-secondary);
        }
        .info-box {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }
        .info-row {
          display: flex;
          justify-content: space-between;
          font-size: 13px;
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
        .form-group input {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 4px;
          font-size: 14px;
          font-family: inherit;
        }
        .form-group small {
          display: block;
          margin-top: 4px;
          font-size: 12px;
          color: var(--text-secondary);
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
