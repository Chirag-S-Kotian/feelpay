-- Settlement tracking
CREATE TABLE IF NOT EXISTS settlements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    settlement_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending/processing/completed/failed
    total_amount BIGINT NOT NULL,                 -- Gross amount
    fee_amount BIGINT NOT NULL DEFAULT 0,        -- Platform fee
    tax_amount BIGINT NOT NULL DEFAULT 0,         -- GST on fee
    net_amount BIGINT NOT NULL,                  -- Net to merchant
    utr VARCHAR(50),                             -- Bank reference
    bank_reference VARCHAR(100),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_settlements_merchant ON settlements(merchant_id);
CREATE INDEX IF NOT EXISTS idx_settlements_date ON settlements(settlement_date);
CREATE INDEX IF NOT EXISTS idx_settlements_status ON settlements(status);

-- Settlement line items
CREATE TABLE IF NOT EXISTS settlement_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    settlement_id UUID NOT NULL REFERENCES settlements(id),
    transaction_id UUID NOT NULL REFERENCES payment_orders(id),
    amount BIGINT NOT NULL,
    fee BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_settlement_items_settlement ON settlement_items(settlement_id);
