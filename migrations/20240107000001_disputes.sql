CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    payment_order_id UUID NOT NULL REFERENCES payment_orders(id),
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    status VARCHAR(32) NOT NULL DEFAULT 'needs_response', -- needs_response/under_review/won/lost/accepted
    reason_code VARCHAR(64),
    reason_message TEXT,
    evidence_due_by TIMESTAMPTZ,
    evidence JSONB,
    connector_reference VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_disputes_merchant_created
    ON disputes (merchant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_disputes_status
    ON disputes (status);

CREATE INDEX IF NOT EXISTS idx_disputes_payment_order
    ON disputes (payment_order_id);
