-- KYC verification records
CREATE TABLE IF NOT EXISTS kyc_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    verification_type VARCHAR(30) NOT NULL,       -- pan/gstin/bank/aadhaar
    document_number VARCHAR(100),
    document_data JSONB,                          -- Encrypted/hashed doc details
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending/verified/rejected
    verified_at TIMESTAMPTZ,
    rejection_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kyc_merchant ON kyc_verifications(merchant_id);

-- Add merchant tier fields
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS 
    onboarding_status VARCHAR(20) NOT NULL DEFAULT 'pending';
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS 
    tier VARCHAR(20) NOT NULL DEFAULT 'starter';
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS 
    daily_limit BIGINT NOT NULL DEFAULT 100000;
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS 
    monthly_limit BIGINT NOT NULL DEFAULT 5000000;
