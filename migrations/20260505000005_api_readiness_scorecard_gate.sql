CREATE TABLE IF NOT EXISTS api_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version VARCHAR(32) NOT NULL UNIQUE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('current', 'supported', 'deprecated')),
    sunset_at TIMESTAMPTZ,
    changelog_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_idempotency_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(255) NOT NULL,
    api_key_id UUID REFERENCES developer_api_keys(id),
    request_hash TEXT NOT NULL,
    response_hash TEXT,
    response_status INTEGER NOT NULL,
    resource_type VARCHAR(64),
    resource_id UUID,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (idempotency_key, api_key_id)
);

CREATE TABLE IF NOT EXISTS api_error_catalog (
    code VARCHAR(64) PRIMARY KEY,
    message TEXT NOT NULL,
    http_status INTEGER NOT NULL,
    retryable BOOLEAN NOT NULL DEFAULT false,
    docs_url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS api_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID REFERENCES developer_api_keys(id),
    window_seconds INTEGER NOT NULL DEFAULT 60,
    max_requests INTEGER NOT NULL DEFAULT 120,
    current_count INTEGER NOT NULL DEFAULT 0,
    reset_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '60 seconds'
);

CREATE TABLE IF NOT EXISTS readiness_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(64) NOT NULL,
    check_name VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pass', 'warning', 'fail')),
    score INTEGER NOT NULL DEFAULT 0,
    owner TEXT,
    action_link TEXT,
    notes TEXT,
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category, check_name)
);

CREATE INDEX IF NOT EXISTS idx_api_idempotency_lookup
    ON api_idempotency_records(idempotency_key, api_key_id);
CREATE INDEX IF NOT EXISTS idx_api_rate_limits_key_reset
    ON api_rate_limits(api_key_id, reset_at);
CREATE INDEX IF NOT EXISTS idx_readiness_category_status
    ON readiness_checks(category, status);

INSERT INTO api_versions (version, status, changelog_url)
VALUES
    ('2026-05-05', 'current', 'https://internal.feelpay/docs/changelog/2026-05-05'),
    ('2026-04-01', 'supported', 'https://internal.feelpay/docs/changelog/2026-04-01'),
    ('2025-12-01', 'deprecated', 'https://internal.feelpay/docs/changelog/2025-12-01')
ON CONFLICT (version) DO NOTHING;

INSERT INTO api_error_catalog (code, message, http_status, retryable, docs_url)
VALUES
    ('invalid_request', 'Invalid request payload', 400, false, 'https://internal.feelpay/docs/errors#invalid_request'),
    ('idempotency_conflict', 'Idempotency key reused with different payload', 409, false, 'https://internal.feelpay/docs/errors#idempotency_conflict'),
    ('scope_denied', 'API key scope does not allow this operation', 403, false, 'https://internal.feelpay/docs/errors#scope_denied'),
    ('rate_limit_exceeded', 'Request rate limit exceeded', 429, true, 'https://internal.feelpay/docs/errors#rate_limit_exceeded'),
    ('provider_unavailable', 'Provider temporarily unavailable', 503, true, 'https://internal.feelpay/docs/errors#provider_unavailable')
ON CONFLICT (code) DO NOTHING;

INSERT INTO readiness_checks (category, check_name, status, score, owner, action_link, notes)
VALUES
    ('Payments', 'provider configured', 'pass', 100, 'Payment Ops Admin', '/connectors', 'At least one active provider'),
    ('Providers', 'sandbox + production secrets configured', 'warning', 70, 'Payment Ops Admin', '/connectors', 'One provider missing live secret'),
    ('Webhooks', 'signatures required and retries configured', 'pass', 95, 'Platform Engineering', '/webhook-config', 'Dead-letter queue monitored'),
    ('Ledger', 'double-entry balanced', 'pass', 100, 'Finance Admin', '/ledger', 'No orphan entries detected'),
    ('Reconciliation', 'duplicate protection and mismatch review', 'warning', 75, 'Finance Admin', '/reconciliation', 'Some unresolved mismatches'),
    ('Security', 'no raw card/CVV and masking enabled', 'pass', 100, 'Compliance Admin', '/compliance-center', 'Masking checks green'),
    ('RBAC', 'deny default and route checks', 'pass', 100, 'Super Admin', '/admin-users-roles', 'Unmapped routes denied'),
    ('Reports', 'finance export + masking + audit', 'pass', 90, 'Finance Admin', '/reports', 'Scheduled reports configured')
ON CONFLICT (category, check_name) DO NOTHING;
