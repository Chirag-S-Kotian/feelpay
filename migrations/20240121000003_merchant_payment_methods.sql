-- Merchant-specific payment method configuration
CREATE TABLE IF NOT EXISTS merchant_payment_methods (
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    method_type VARCHAR(20) NOT NULL, -- card, upi, netbanking, wallet, emi, bnpl
    enabled BOOLEAN NOT NULL DEFAULT true,
    config JSONB, -- Optional configuration like gateway priority, surcharges, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (merchant_id, method_type)
);

-- Seed defaults for existing merchants
INSERT INTO merchant_payment_methods (merchant_id, method_type, enabled)
SELECT id, m, true
FROM merchants, (VALUES ('card'), ('upi'), ('netbanking'), ('wallet'), ('emi'), ('bnpl')) AS t(m)
ON CONFLICT DO NOTHING;
