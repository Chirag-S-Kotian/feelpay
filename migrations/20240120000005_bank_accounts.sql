-- Create bank_accounts table for merchant bank account management with penny drop verification
-- Requirement 4.1, 4.8: Bank Account Management with Re-verification

CREATE TABLE IF NOT EXISTS bank_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    account_holder_name VARCHAR(255) NOT NULL,
    account_number_encrypted VARCHAR(255) NOT NULL, -- AES-256-GCM encrypted
    ifsc_code VARCHAR(11) NOT NULL,
    account_type VARCHAR(20) NOT NULL, -- 'savings', 'current'
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'verified', 'failed'
    penny_drop_amount_paise INT, -- amount of micro-deposit for verification
    penny_drop_reference VARCHAR(255), -- reference code for verification
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ,
    CONSTRAINT valid_account_type CHECK (account_type IN ('savings', 'current')),
    CONSTRAINT valid_verification_status CHECK (verification_status IN ('pending', 'verified', 'failed'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_bank_accounts_merchant_id ON bank_accounts(merchant_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_is_primary ON bank_accounts(is_primary);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_verification_status ON bank_accounts(verification_status);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_merchant_primary ON bank_accounts(merchant_id, is_primary);
