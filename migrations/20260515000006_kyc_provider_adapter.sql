CREATE TABLE IF NOT EXISTS kyc_provider_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID,
    provider_id UUID REFERENCES integration_providers(id) ON DELETE SET NULL,
    verification_type VARCHAR(40) NOT NULL,
    environment VARCHAR(20) NOT NULL CHECK (environment IN ('test', 'live')),
    identifier_masked VARCHAR(120) NOT NULL,
    result_status VARCHAR(40) NOT NULL CHECK (
        result_status IN (
            'verified',
            'failed',
            'pending',
            'provider_error',
            'manual_review_required'
        )
    ),
    provider_reference VARCHAR(180),
    response_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    error_code VARCHAR(120),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kyc_provider_attempts_merchant_created
    ON kyc_provider_attempts(merchant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_kyc_provider_attempts_provider_created
    ON kyc_provider_attempts(provider_id, created_at DESC);
