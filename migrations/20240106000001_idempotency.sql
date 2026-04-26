CREATE TABLE IF NOT EXISTS idempotency_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    idempotency_key VARCHAR(255) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    resource_type VARCHAR(32) NOT NULL, -- payment/refund/capture
    response_code INT,
    response_body JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (merchant_id, idempotency_key, resource_type)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_lookup
    ON idempotency_keys (merchant_id, idempotency_key, resource_type);
