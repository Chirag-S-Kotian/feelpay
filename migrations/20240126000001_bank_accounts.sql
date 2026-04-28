-- Bank Accounts Table
-- Stores encrypted bank account details for merchants with penny drop verification tracking

CREATE TABLE IF NOT EXISTS bank_accounts (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
  account_holder_name VARCHAR(255) NOT NULL,
  account_number_encrypted VARCHAR(255) NOT NULL,
  ifsc_code VARCHAR(11) NOT NULL,
  account_type VARCHAR(20) NOT NULL,
  verification_status VARCHAR(20) NOT NULL,
  penny_drop_amount_paise INT,
  penny_drop_reference VARCHAR(255),
  is_primary BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  verified_at TIMESTAMPTZ
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_bank_accounts_merchant_id ON bank_accounts(merchant_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_is_primary ON bank_accounts(is_primary);
