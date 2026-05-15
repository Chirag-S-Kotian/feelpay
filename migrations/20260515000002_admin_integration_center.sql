-- Provider-agnostic admin integration center for internal platform operations.

CREATE TABLE IF NOT EXISTS integration_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_kind VARCHAR(40) NOT NULL CHECK (
        provider_kind IN (
            'email',
            'sms',
            'otp',
            'bank_payout',
            'kyc',
            'notification',
            'reconciliation'
        )
    ),
    name VARCHAR(180) NOT NULL,
    code VARCHAR(120) NOT NULL,
    environment VARCHAR(20) NOT NULL CHECK (environment IN ('test', 'live')),
    status VARCHAR(20) NOT NULL DEFAULT 'disabled' CHECK (status IN ('enabled', 'disabled')),
    is_default BOOLEAN NOT NULL DEFAULT false,
    priority INTEGER NOT NULL DEFAULT 100,
    timeout_ms INTEGER NOT NULL DEFAULT 10000,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_backoff_ms INTEGER NOT NULL DEFAULT 1000,
    health_status VARCHAR(20) NOT NULL DEFAULT 'unknown',
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    last_error_code VARCHAR(120),
    last_error_message TEXT,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    live_enabled_approved_at TIMESTAMPTZ,
    live_enabled_approved_by UUID,
    created_by_admin_id UUID,
    updated_by_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_kind, code, environment)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_integration_providers_default
    ON integration_providers(provider_kind, environment)
    WHERE is_default = true;

CREATE TABLE IF NOT EXISTS integration_provider_secrets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES integration_providers(id) ON DELETE CASCADE,
    secret_name VARCHAR(120) NOT NULL,
    encrypted_value TEXT NOT NULL,
    nonce TEXT NOT NULL,
    masked_value VARCHAR(80) NOT NULL,
    rotated_by_admin_id UUID,
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, secret_name)
);

CREATE TABLE IF NOT EXISTS integration_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID REFERENCES integration_providers(id) ON DELETE SET NULL,
    action VARCHAR(120) NOT NULL,
    request_type VARCHAR(120) NOT NULL,
    status VARCHAR(30) NOT NULL,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    error_code VARCHAR(120),
    error_message TEXT,
    actor_admin_id UUID,
    request_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    response_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_integration_logs_provider_created
    ON integration_logs(provider_id, created_at DESC);
