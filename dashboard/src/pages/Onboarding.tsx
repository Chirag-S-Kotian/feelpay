import { useEffect, useMemo, useState } from 'react';
import { CheckCircle2, Circle, ShieldCheck, Building2, Landmark, Rocket } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type Verification = {
  id: string;
  verification_type: string;
  status: string;
};

type OnboardingStatus = {
  onboarding_status: string;
  tier: string;
  daily_limit: number;
  monthly_limit: number;
  verifications: Verification[];
};

type Limits = {
  per_transaction_limit: number;
  daily_limit: number;
  monthly_limit: number;
  settlement_cycle: string;
};

const FLOW_STEPS = [
  { id: 1, name: 'Business Profile', icon: Building2 },
  { id: 2, name: 'KYC Verification', icon: ShieldCheck },
  { id: 3, name: 'Settlement Account', icon: Landmark },
  { id: 4, name: 'Go-Live Review', icon: Rocket },
] as const;

const BUSINESS_TYPES = [
  { value: 'pvt_ltd', label: 'Private Limited' },
  { value: 'llp', label: 'LLP' },
  { value: 'sole_proprietor', label: 'Sole Proprietor' },
  { value: 'partnership', label: 'Partnership' },
  { value: 'trust', label: 'Trust' },
  { value: 'ngo', label: 'NGO' },
];

export default function Onboarding() {
  const [step, setStep] = useState<number>(1);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [status, setStatus] = useState<OnboardingStatus | null>(null);
  const [limits, setLimits] = useState<Limits | null>(null);
  const [digilockerStatus, setDigilockerStatus] = useState<string>('not_started');
  const [digilockerEnabled, setDigilockerEnabled] = useState<boolean>(true);

  const [formData, setFormData] = useState({
    businessName: '',
    businessType: '',
    pan: '',
    gstin: '',
    addressLine1: '',
    city: '',
    state: '',
    postalCode: '',
    bankName: '',
    accountHolderName: '',
    accountNumber: '',
    ifscCode: '',
  });

  const completed = useMemo(() => {
    const verifications = status?.verifications || [];
    const hasPan = verifications.some(v => v.verification_type === 'pan' && v.status === 'verified');
    const hasGstin = formData.gstin.trim().length === 0
      || verifications.some(v => v.verification_type === 'gstin' && v.status === 'verified');
    const hasBank = verifications.some(v => v.verification_type === 'bank') || status?.onboarding_status !== 'pending';

    return {
      business:
        formData.businessName.trim().length > 0 &&
        formData.businessType.trim().length > 0,
      kyc: hasPan && hasGstin,
      bank: hasBank,
      review: status?.onboarding_status === 'approved' || status?.onboarding_status === 'in_progress',
    };
  }, [status, formData]);

  const load = async () => {
    setLoading(true);
    setError('');
    try {
      const [statusRes, profileRes, limitsResult, digiResult] = await Promise.all([
        apiRequest<OnboardingStatus>('GET', '/v1/onboarding/status', undefined, undefined, { cacheMs: 20_000 }),
        apiRequest<{ business_name?: string; business_type?: string }>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 30_000 }),
        apiRequest<Limits>('GET', '/v1/onboarding/limits', undefined, undefined, { cacheMs: 20_000 })
          .then((value) => ({ ok: true as const, value }))
          .catch(() => ({ ok: false as const })),
        apiRequest<{ status: string }>('GET', '/v1/onboarding/digilocker/status', undefined, undefined, { cacheMs: 20_000 })
          .then((value) => ({ ok: true as const, value }))
          .catch((err: unknown) => ({ ok: false as const, err })),
      ]);
      const limitsRes: Limits = limitsResult.ok
        ? limitsResult.value
        : {
            per_transaction_limit: 0,
            daily_limit: statusRes.daily_limit || 0,
            monthly_limit: statusRes.monthly_limit || 0,
            settlement_cycle: 'T+2',
          };
      if (digiResult.ok) {
        setDigilockerStatus(digiResult.value.status || 'not_started');
        setDigilockerEnabled(true);
      } else {
        const msg = digiResult.err instanceof Error ? digiResult.err.message : '';
        if (msg.includes('disabled')) {
          setDigilockerEnabled(false);
          setDigilockerStatus('disabled');
        } else {
          setDigilockerStatus('not_started');
        }
      }
      setStatus(statusRes);
      setLimits(limitsRes);
      setFormData(prev => ({
        ...prev,
        businessName: profileRes.business_name || prev.businessName,
        businessType: profileRes.business_type || prev.businessType,
      }));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load onboarding data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const validateStep = (targetStep: number): string | null => {
    if (targetStep === 2) {
      if (!formData.businessName.trim()) return 'Business name required';
      if (!formData.businessType.trim()) return 'Business type required';
    }
    if (targetStep === 3) {
      if (!/^[A-Z]{5}[0-9]{4}[A-Z]$/.test(formData.pan.trim().toUpperCase())) return 'Valid PAN required';
      if (formData.gstin.trim() && !/^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[A-Z0-9]{1}Z[A-Z0-9]{1}$/.test(formData.gstin.trim().toUpperCase())) {
        return 'GSTIN format invalid';
      }
    }
    if (targetStep === 4) {
      if (!formData.accountHolderName.trim()) return 'Account holder name required';
      if (!/^\d{8,20}$/.test(formData.accountNumber.trim())) return 'Valid account number required';
      if (!/^[A-Z]{4}0[A-Z0-9]{6}$/.test(formData.ifscCode.trim().toUpperCase())) return 'Valid IFSC required';
    }
    return null;
  };

  const saveBusiness = async () => {
    await apiRequest('POST', '/v1/onboarding/business-info', {
      business_name: formData.businessName.trim(),
      business_type: formData.businessType,
      pan: formData.pan.trim().toUpperCase() || 'AAAAA0000A',
      gstin: formData.gstin.trim().toUpperCase() || null,
      address: {
        line1: formData.addressLine1,
        city: formData.city,
        state: formData.state,
        postal_code: formData.postalCode,
      },
    });
  };

  const saveKyc = async () => {
    await apiRequest('POST', '/v1/onboarding/kyc', {
      verification_type: 'pan',
      document_number: formData.pan.trim().toUpperCase(),
    });
    if (formData.gstin.trim()) {
      await apiRequest('POST', '/v1/onboarding/kyc', {
        verification_type: 'gstin',
        document_number: formData.gstin.trim().toUpperCase(),
      });
    }
  };

  const saveBank = async () => {
    await apiRequest('POST', '/v1/onboarding/bank-account', {
      account_holder_name: formData.accountHolderName.trim(),
      account_number: formData.accountNumber.trim(),
      ifsc_code: formData.ifscCode.trim().toUpperCase(),
      bank_name: formData.bankName.trim() || undefined,
    });
  };

  const handleNext = async () => {
    setError('');
    setSuccess('');
    const validationError = validateStep(step + 1);
    if (validationError) {
      setError(validationError);
      return;
    }

    setSaving(true);
    try {
      if (step === 1) await saveBusiness();
      if (step === 2) await saveKyc();
      if (step === 3) await saveBank();

      await apiRequest('POST', '/v1/onboarding/registration/step', {
        step: step + 1,
        data: {
          ui_step: step + 1,
          updated_at: new Date().toISOString(),
        },
      });

      await load();
      setStep(s => Math.min(4, s + 1));
      setSuccess(step === 3 ? 'Onboarding submitted. Verification in progress.' : 'Step saved.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save step');
    } finally {
      setSaving(false);
    }
  };

  const reviewItems = [
    { label: 'Business profile', ok: completed.business },
    { label: 'PAN / GST verification', ok: completed.kyc },
    { label: 'DigiLocker verification', ok: digilockerStatus === 'verified' || !digilockerEnabled },
    { label: 'Settlement account', ok: completed.bank },
    { label: 'Activation state', ok: completed.review },
  ];

  if (loading) return <div className="loading">Loading onboarding...</div>;

  const connectDigiLocker = async () => {
    setError('');
    setSuccess('');
    try {
      const data = await apiRequest<{ redirect_url?: string }>('POST', '/v1/onboarding/digilocker/initiate', {
        purpose: 'merchant_kyc',
      });
      if (data.redirect_url) {
        window.open(data.redirect_url, '_blank', 'noopener,noreferrer');
      }
      setSuccess('DigiLocker flow initiated. Complete auth, then click Refresh Verification Status.');
      setDigilockerStatus('pending');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to initiate DigiLocker');
    }
  };

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Merchant Onboarding</h1>
          <p>Progressive onboarding like payment gateways: collect minimal info first, verify, then activate.</p>
        </div>
      </div>

      {error && <div className="alert alert-error">{error}</div>}
      {success && <div className="env-badge">{success}</div>}

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="onboarding-steps">
          {FLOW_STEPS.map((s) => {
            const Icon = s.icon;
            const active = step === s.id;
            return (
              <button
                key={s.id}
                type="button"
                onClick={() => s.id <= step && setStep(s.id)}
                className={`onboarding-step ${active ? 'active' : ''}`}
                disabled={s.id > step}
              >
                <Icon size={14} />
                {s.name}
              </button>
            );
          })}
        </div>
      </div>

      <div className="card">
        {step === 1 && (
          <>
            <h3>Business Profile</h3>
            <div className="input-grid">
              <div className="input-group">
                <label>Business Name</label>
                <input value={formData.businessName} onChange={(e) => setFormData({ ...formData, businessName: e.target.value })} />
              </div>
              <div className="input-group">
                <label>Business Type</label>
                <select value={formData.businessType} onChange={(e) => setFormData({ ...formData, businessType: e.target.value })}>
                  <option value="">Select type</option>
                  {BUSINESS_TYPES.map(bt => (
                    <option key={bt.value} value={bt.value}>{bt.label}</option>
                  ))}
                </select>
              </div>
              <div className="input-group">
                <label>Address Line 1</label>
                <input value={formData.addressLine1} onChange={(e) => setFormData({ ...formData, addressLine1: e.target.value })} />
              </div>
              <div className="input-group">
                <label>City</label>
                <input value={formData.city} onChange={(e) => setFormData({ ...formData, city: e.target.value })} />
              </div>
              <div className="input-group">
                <label>State</label>
                <input value={formData.state} onChange={(e) => setFormData({ ...formData, state: e.target.value })} />
              </div>
              <div className="input-group">
                <label>Postal Code</label>
                <input value={formData.postalCode} onChange={(e) => setFormData({ ...formData, postalCode: e.target.value })} />
              </div>
            </div>
          </>
        )}

        {step === 2 && (
          <>
            <h3>KYC Verification</h3>
            <div className="input-grid">
              <div className="input-group">
                <label>PAN</label>
                <input value={formData.pan} maxLength={10} onChange={(e) => setFormData({ ...formData, pan: e.target.value.toUpperCase() })} placeholder="ABCDE1234F" />
              </div>
              <div className="input-group">
                <label>GSTIN (optional)</label>
                <input value={formData.gstin} maxLength={15} onChange={(e) => setFormData({ ...formData, gstin: e.target.value.toUpperCase() })} placeholder="22AAAAA0000A1Z5" />
              </div>
            </div>
            <div className="card" style={{ marginTop: 12 }}>
              <h4 style={{ marginBottom: 6 }}>DigiLocker Auto-KYC</h4>
              <p className="mini-help" style={{ marginTop: 0 }}>
                Enterprise path: fetch verified docs via DigiLocker to accelerate approval.
              </p>
              <div className="review-row" style={{ marginBottom: 8 }}>
                {digilockerStatus === 'verified' ? <CheckCircle2 size={16} color="#5ad189" /> : <Circle size={16} color="#9fb8da" />}
                <span>Status: {digilockerStatus}</span>
              </div>
              <button
                type="button"
                className="btn-secondary"
                onClick={() => void connectDigiLocker()}
                disabled={!digilockerEnabled || saving}
              >
                {digilockerEnabled ? 'Connect DigiLocker' : 'DigiLocker Disabled by Infra'}
              </button>
            </div>
            <p className="mini-help">Collect mandatory details first (PAN), optional next (GSTIN), then verify async.</p>
          </>
        )}

        {step === 3 && (
          <>
            <h3>Settlement Bank Account</h3>
            <div className="input-grid">
              <div className="input-group">
                <label>Account Holder Name</label>
                <input value={formData.accountHolderName} onChange={(e) => setFormData({ ...formData, accountHolderName: e.target.value })} />
              </div>
              <div className="input-group">
                <label>Bank Name</label>
                <input value={formData.bankName} onChange={(e) => setFormData({ ...formData, bankName: e.target.value })} />
              </div>
              <div className="input-group">
                <label>Account Number</label>
                <input value={formData.accountNumber} onChange={(e) => setFormData({ ...formData, accountNumber: e.target.value })} />
              </div>
              <div className="input-group">
                <label>IFSC Code</label>
                <input value={formData.ifscCode} maxLength={11} onChange={(e) => setFormData({ ...formData, ifscCode: e.target.value.toUpperCase() })} />
              </div>
            </div>
          </>
        )}

        {step === 4 && (
          <>
            <h3>Go-Live Review</h3>
            <div className="review-list">
              {reviewItems.map((item) => (
                <div key={item.label} className="review-row">
                  {item.ok ? <CheckCircle2 size={16} color="#5ad189" /> : <Circle size={16} color="#9fb8da" />}
                  <span>{item.label}</span>
                </div>
              ))}
            </div>
            <div className="ops-grid" style={{ marginTop: 10 }}>
              <div className="ops-tile"><span>Tier</span><strong>{status?.tier || 'N/A'}</strong></div>
              <div className="ops-tile"><span>Onboarding Status</span><strong>{status?.onboarding_status || 'pending'}</strong></div>
              <div className="ops-tile"><span>Per Txn Limit</span><strong>INR {limits?.per_transaction_limit?.toLocaleString() || 0}</strong></div>
              <div className="ops-tile"><span>Settlement</span><strong>{limits?.settlement_cycle || 'T+2'}</strong></div>
            </div>
          </>
        )}

        <div className="form-actions" style={{ marginTop: 16 }}>
          {step > 1 && (
            <button type="button" className="btn-secondary" onClick={() => setStep(step - 1)} disabled={saving}>
              Back
            </button>
          )}
          {step < 4 && (
            <button type="button" className="btn-primary" onClick={() => void handleNext()} disabled={saving}>
              {saving ? 'Saving...' : 'Continue'}
            </button>
          )}
          {step === 4 && (
            <button type="button" className="btn-primary" onClick={() => void load()} disabled={saving}>
              Refresh Verification Status
            </button>
          )}
        </div>
      </div>

      <style>{`
        .alert.alert-error {
          border: 1px solid rgba(248, 113, 113, 0.4);
          background: rgba(239, 68, 68, 0.15);
          color: #fecaca;
          padding: 10px 12px;
          border-radius: 10px;
          margin-bottom: 12px;
        }
        .onboarding-steps {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }
        .onboarding-step {
          border: 1px solid rgba(102, 163, 252, 0.24);
          background: rgba(13, 33, 60, 0.72);
          color: #cbe1ff;
          border-radius: 999px;
          padding: 8px 12px;
          display: inline-flex;
          align-items: center;
          gap: 6px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
        }
        .onboarding-step.active {
          background: linear-gradient(135deg, #f59e0b, #8b5cf6);
          border-color: rgba(255, 205, 130, 0.4);
          color: #fff;
        }
        .onboarding-step:disabled {
          opacity: 0.55;
          cursor: not-allowed;
        }
        .input-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
          gap: 12px;
        }
        .mini-help {
          margin-top: 10px;
          color: #9db7da;
          font-size: 13px;
        }
        .review-list {
          display: grid;
          gap: 8px;
        }
        .review-row {
          display: flex;
          align-items: center;
          gap: 8px;
          color: #d9eaff;
        }
      `}</style>
    </div>
  );
}
