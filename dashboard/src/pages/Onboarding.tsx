import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';

const steps = [
  { id: 1, name: 'Business Info', description: 'Company details' },
  { id: 2, name: 'Documents', description: 'PAN & GSTIN' },
  { id: 3, name: 'Bank Account', description: 'Settlement details' },
  { id: 4, name: 'Review', description: 'Submit application' },
];

export default function Onboarding() {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [formData, setFormData] = useState({
    businessName: '',
    businessType: '',
    pan: '',
    gstin: '',
    bankName: '',
    accountNumber: '',
    ifscCode: '',
  });

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      await apiRequest('POST', '/v1/onboarding/kyc', {
        verification_type: 'pan',
        document_number: formData.pan,
      });

      if (formData.gstin) {
        await apiRequest('POST', '/v1/onboarding/kyc', {
          verification_type: 'gstin',
          document_number: formData.gstin,
        });
      }

      await apiRequest('POST', '/v1/onboarding/bank-account', {
        account_holder_name: formData.businessName,
        account_number: formData.accountNumber,
        ifsc_code: formData.ifscCode,
        bank_name: formData.bankName,
      });

      setSuccess('Onboarding submitted and verified.');
      setTimeout(() => navigate('/'), 1000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Onboarding failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <h1>Merchant Onboarding</h1>
      
      <div className="stepper">
        {steps.map((step) => (
          <div 
            key={step.id} 
            className={`step ${currentStep >= step.id ? 'active' : ''}`}
            onClick={() => setCurrentStep(step.id)}
          >
            <div className="step-number">{step.id}</div>
            <div className="step-info">
              <div className="step-name">{step.name}</div>
              <div className="step-desc">{step.description}</div>
            </div>
          </div>
        ))}
      </div>

      <form onSubmit={handleSubmit} className="form">
        {error && <div className="error-message">{error}</div>}
        {success && <div className="env-badge">{success}</div>}

        {currentStep === 1 && (
          <div className="form-step">
            <h2>Business Information</h2>
            <div className="form-group">
              <label>Business Name</label>
              <input 
                type="text" 
                value={formData.businessName}
                onChange={(e) => setFormData({...formData, businessName: e.target.value})}
                placeholder="Your company name"
              />
            </div>
            <div className="form-group">
              <label>Business Type</label>
              <select 
                value={formData.businessType}
                onChange={(e) => setFormData({...formData, businessType: e.target.value})}
              >
                <option value="">Select type</option>
                <option value="proprietorship">Proprietorship</option>
                <option value="partnership">Partnership</option>
                <option value="private">Private Limited</option>
                <option value="public">Public Limited</option>
              </select>
            </div>
          </div>
        )}

        {currentStep === 2 && (
          <div className="form-step">
            <h2>Documents</h2>
            <div className="form-group">
              <label>PAN Number</label>
              <input 
                type="text" 
                value={formData.pan}
                onChange={(e) => setFormData({...formData, pan: e.target.value.toUpperCase()})}
                placeholder="AAAPL1234C"
                maxLength={10}
              />
            </div>
            <div className="form-group">
              <label>GSTIN (Optional)</label>
              <input 
                type="text" 
                value={formData.gstin}
                onChange={(e) => setFormData({...formData, gstin: e.target.value.toUpperCase()})}
                placeholder="24AABCU9603R1ZM"
                maxLength={15}
              />
            </div>
          </div>
        )}

        {currentStep === 3 && (
          <div className="form-step">
            <h2>Bank Account</h2>
            <div className="form-group">
              <label>Bank Name</label>
              <input 
                type="text" 
                value={formData.bankName}
                onChange={(e) => setFormData({...formData, bankName: e.target.value})}
                placeholder="HDFC Bank"
              />
            </div>
            <div className="form-group">
              <label>Account Number</label>
              <input 
                type="text" 
                value={formData.accountNumber}
                onChange={(e) => setFormData({...formData, accountNumber: e.target.value})}
                placeholder="1234567890"
              />
            </div>
            <div className="form-group">
              <label>IFSC Code</label>
              <input 
                type="text" 
                value={formData.ifscCode}
                onChange={(e) => setFormData({...formData, ifscCode: e.target.value.toUpperCase()})}
                placeholder="HDFC0001234"
                maxLength={11}
              />
            </div>
          </div>
        )}

        {currentStep === 4 && (
          <div className="form-step">
            <h2>Review & Submit</h2>
            <div className="review-box">
              <p><strong>Business:</strong> {formData.businessName}</p>
              <p><strong>Type:</strong> {formData.businessType}</p>
              <p><strong>PAN:</strong> {formData.pan}</p>
              <p><strong>Bank:</strong> {formData.bankName} ({formData.accountNumber.slice(-4)})</p>
            </div>
          </div>
        )}

        <div className="form-actions">
          {currentStep > 1 && (
            <button type="button" onClick={() => setCurrentStep(s => s - 1)} className="btn-secondary">
              Back
            </button>
          )}
          {currentStep < 4 ? (
            <button type="button" onClick={() => setCurrentStep(s => s + 1)} className="btn-primary" disabled={loading}>
              Continue
            </button>
          ) : (
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? 'Submitting...' : 'Submit Application'}
            </button>
          )}
        </div>
      </form>
    </div>
  );
}
