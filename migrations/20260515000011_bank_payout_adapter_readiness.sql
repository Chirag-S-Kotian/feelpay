-- Provider-neutral bank/payout adapter readiness without enabling live rails.

ALTER TABLE integration_providers
    ADD COLUMN IF NOT EXISTS approved_for_live BOOLEAN NOT NULL DEFAULT false;

UPDATE integration_providers
SET approved_for_live = live_enabled_approved_at IS NOT NULL
WHERE provider_kind = 'bank_payout';

ALTER TABLE payouts_company
    ADD COLUMN IF NOT EXISTS bank_payout_provider_id UUID REFERENCES integration_providers(id),
    ADD COLUMN IF NOT EXISTS provider_reference TEXT,
    ADD COLUMN IF NOT EXISTS provider_response_summary JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE UNIQUE INDEX IF NOT EXISTS idx_payouts_company_idempotency
    ON payouts_company(idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS bank_payout_provider_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payout_id UUID NOT NULL REFERENCES payouts_company(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES integration_providers(id),
    idempotency_key TEXT NOT NULL,
    operation VARCHAR(40) NOT NULL,
    status VARCHAR(24) NOT NULL,
    provider_reference TEXT,
    response_summary_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_code VARCHAR(120),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, idempotency_key, operation)
);

CREATE TABLE IF NOT EXISTS bank_payout_provider_callbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES integration_providers(id),
    payout_id UUID REFERENCES payouts_company(id) ON DELETE SET NULL,
    provider_event_id TEXT NOT NULL,
    provider_reference TEXT,
    mapped_status VARCHAR(24) NOT NULL,
    payload_summary_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,
    processing_status VARCHAR(24) NOT NULL DEFAULT 'processed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, provider_event_id)
);

CREATE INDEX IF NOT EXISTS idx_bank_payout_provider_attempts_payout
    ON bank_payout_provider_attempts(payout_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bank_payout_provider_callbacks_payout
    ON bank_payout_provider_callbacks(payout_id, created_at DESC);
