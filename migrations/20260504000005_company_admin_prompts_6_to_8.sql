-- Advanced reconciliation, developer experience, compliance center.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE reconciliation_files
    ADD COLUMN IF NOT EXISTS provider_id UUID REFERENCES payment_providers(id),
    ADD COLUMN IF NOT EXISTS file_hash VARCHAR(128),
    ADD COLUMN IF NOT EXISTS uploaded_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS row_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS matched_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS mismatch_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS duplicate_count INTEGER NOT NULL DEFAULT 0;

UPDATE reconciliation_files
SET uploaded_by_admin_id = COALESCE(uploaded_by_admin_id, uploaded_by)
WHERE uploaded_by_admin_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_reconciliation_files_hash
    ON reconciliation_files(file_hash)
    WHERE file_hash IS NOT NULL;

ALTER TABLE reconciliation_records
    ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES reconciliation_files(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS row_number INTEGER,
    ADD COLUMN IF NOT EXISTS external_reference VARCHAR(255),
    ADD COLUMN IF NOT EXISTS internal_reference VARCHAR(255),
    ADD COLUMN IF NOT EXISTS amount BIGINT,
    ADD COLUMN IF NOT EXISTS currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    ADD COLUMN IF NOT EXISTS record_type VARCHAR(60),
    ADD COLUMN IF NOT EXISTS match_status VARCHAR(40) NOT NULL DEFAULT 'unmatched',
    ADD COLUMN IF NOT EXISTS matched_entity_type VARCHAR(80),
    ADD COLUMN IF NOT EXISTS matched_entity_id UUID,
    ADD COLUMN IF NOT EXISTS resolution_status VARCHAR(40) NOT NULL DEFAULT 'open',
    ADD COLUMN IF NOT EXISTS resolution_note TEXT,
    ADD COLUMN IF NOT EXISTS resolved_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS provider_refund_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS provider_dispute_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS ledger_transaction_group_id UUID,
    ADD COLUMN IF NOT EXISTS confidence_score INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS suggested_match JSONB NOT NULL DEFAULT '{}'::jsonb;

UPDATE reconciliation_records
SET file_id = COALESCE(file_id, reconciliation_file_id),
    external_reference = COALESCE(external_reference, provider_payment_id, provider_order_id, settlement_reference, bank_utr),
    internal_reference = COALESCE(internal_reference, internal_payment_id::text, internal_order_id::text, refund_id::text, transfer_id::text),
    amount = COALESCE(amount, observed_amount, expected_amount),
    match_status = COALESCE(NULLIF(match_status, ''), status),
    resolution_status = CASE WHEN resolved_at IS NULL THEN resolution_status ELSE 'resolved' END,
    resolution_note = COALESCE(resolution_note, resolution_notes),
    resolved_by_admin_id = COALESCE(resolved_by_admin_id, resolved_by)
WHERE file_id IS NULL OR external_reference IS NULL OR internal_reference IS NULL;

CREATE INDEX IF NOT EXISTS idx_reconciliation_records_file ON reconciliation_records(file_id);
CREATE INDEX IF NOT EXISTS idx_reconciliation_records_match_status ON reconciliation_records(match_status, resolution_status);
CREATE INDEX IF NOT EXISTS idx_reconciliation_records_external_reference ON reconciliation_records(external_reference);
CREATE INDEX IF NOT EXISTS idx_reconciliation_records_ledger_group ON reconciliation_records(ledger_transaction_group_id);

CREATE TABLE IF NOT EXISTS developer_api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(120) NOT NULL,
    environment VARCHAR(20) NOT NULL,
    key_prefix VARCHAR(24) NOT NULL,
    encrypted_key_hash TEXT NOT NULL,
    masked_key VARCHAR(80) NOT NULL,
    scopes JSONB NOT NULL DEFAULT '[]'::jsonb,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_by_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_request_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    environment VARCHAR(20) NOT NULL,
    api_key_id UUID REFERENCES developer_api_keys(id),
    method VARCHAR(12) NOT NULL,
    path TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    idempotency_key VARCHAR(255),
    request_id VARCHAR(255),
    error_code VARCHAR(120),
    request_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    response_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS webhook_replays (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    webhook_event_id UUID NOT NULL REFERENCES payment_webhook_events(id) ON DELETE CASCADE,
    replay_status VARCHAR(30) NOT NULL DEFAULT 'queued',
    replayed_by_admin_id UUID,
    replayed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS security_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(30) NOT NULL,
    actor_admin_id UUID,
    entity_type VARCHAR(80),
    entity_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT,
    summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS access_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    review_period VARCHAR(80) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    started_by_admin_id UUID,
    completed_by_admin_id UUID,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS access_review_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    review_id UUID NOT NULL REFERENCES access_reviews(id) ON DELETE CASCADE,
    admin_user_id UUID NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    role VARCHAR(80) NOT NULL,
    decision VARCHAR(30) NOT NULL DEFAULT 'keep',
    note TEXT,
    reviewed_by_admin_id UUID,
    reviewed_at TIMESTAMPTZ
);

INSERT INTO admin_permissions (code, description) VALUES
    ('developer.view', 'View developer integration tools'),
    ('developer.manage', 'Manage API keys, webhook testing, and replay'),
    ('compliance.view', 'View compliance center'),
    ('compliance.manage', 'Manage compliance reviews and exports'),
    ('security_events.view', 'View security events')
ON CONFLICT (code) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_developer_api_keys_env_status ON developer_api_keys(environment, status);
CREATE INDEX IF NOT EXISTS idx_api_request_logs_request_id ON api_request_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_api_request_logs_idempotency ON api_request_logs(idempotency_key);
CREATE INDEX IF NOT EXISTS idx_webhook_replays_event ON webhook_replays(webhook_event_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_created ON security_events(severity, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_access_reviews_status ON access_reviews(status, started_at DESC);
