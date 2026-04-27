-- KYC Status Machine & Multi-step Registration

-- Add registration progress to merchants
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS registration_step INTEGER DEFAULT 1;
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS registration_data JSONB DEFAULT '{}';

-- Create merchant_kyc table if it doesn't exist
CREATE TABLE IF NOT EXISTS merchant_kyc (
    merchant_id UUID PRIMARY KEY REFERENCES merchants(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- KYC Status enum: pending, under_review, approved, rejected, resubmit_required
ALTER TABLE merchant_kyc ADD COLUMN IF NOT EXISTS kyc_status VARCHAR(20) DEFAULT 'pending' 
    CHECK (kyc_status IN ('pending', 'under_review', 'approved', 'rejected', 'resubmit_required'));
ALTER TABLE merchant_kyc ADD COLUMN IF NOT EXISTS rejection_reason TEXT;
ALTER TABLE merchant_kyc ADD COLUMN IF NOT EXISTS resubmit_fields JSONB DEFAULT '[]';
ALTER TABLE merchant_kyc ADD COLUMN IF NOT EXISTS reviewed_by UUID REFERENCES users(id);
ALTER TABLE merchant_kyc ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;

-- Add trigger for updated_at
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_merchant_kyc_updated_at') THEN
        CREATE TRIGGER update_merchant_kyc_updated_at 
            BEFORE UPDATE ON merchant_kyc 
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;

-- Add business type to merchants
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS business_type VARCHAR(50) 
    CHECK (business_type IN ('pvt_ltd', 'llp', 'sole_proprietor', 'partnership', 'trust', 'ngo'));

-- Add MCC code
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS mcc_code VARCHAR(4);

-- Transaction limits
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS per_transaction_limit INTEGER DEFAULT 1000000; -- 10 lakhs
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS daily_limit INTEGER DEFAULT 5000000; -- 50 lakhs
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS monthly_limit INTEGER DEFAULT 50000000; -- 5 crores

-- Settlement cycle
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS settlement_cycle VARCHAR(20) DEFAULT 't+1'
    CHECK (settlement_cycle IN ('t+1', 't+2', 'instant'));

-- Allowed payment methods
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS allowed_payment_methods JSONB DEFAULT 
    '["card", "upi", "netbanking", "wallet"]';

-- Custom branding
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS branding_logo VARCHAR(500);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS branding_color VARCHAR(7);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_merchants_registration_step ON merchants(registration_step);
CREATE INDEX IF NOT EXISTS idx_merchant_kyc_status ON merchant_kyc(kyc_status);
CREATE INDEX IF NOT EXISTS idx_merchants_business_type ON merchants(business_type);