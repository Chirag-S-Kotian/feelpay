-- Mandates for Recurring Payments (UPI Autopay, e-Mandates)
CREATE TABLE IF NOT EXISTS mandates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE SET NULL,
    mandate_type VARCHAR(20) NOT NULL, -- upi, card, nach
    mandate_reference VARCHAR(100) UNIQUE, -- UMN (Unique Mandate Number) for UPI
    status VARCHAR(20) NOT NULL DEFAULT 'initiated', -- initiated, active, revoked, expired, paused
    amount_limit BIGINT NOT NULL, -- Maximum amount per transaction in paise
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    frequency VARCHAR(20) NOT NULL, -- daily, weekly, monthly, quarterly, yearly, as_presented
    start_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_date TIMESTAMPTZ,
    payer_vpa VARCHAR(255), -- Used for UPI Autopay
    payer_name VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mandates_merchant ON mandates(merchant_id);
CREATE INDEX IF NOT EXISTS idx_mandates_customer ON mandates(customer_id);
CREATE INDEX IF NOT EXISTS idx_mandates_status ON mandates(status);
CREATE INDEX IF NOT EXISTS idx_mandates_reference ON mandates(mandate_reference);

-- Add mandate_id to subscriptions to link them
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS mandate_id UUID REFERENCES mandates(id) ON DELETE SET NULL;
