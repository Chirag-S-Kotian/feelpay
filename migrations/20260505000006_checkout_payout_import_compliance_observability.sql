ALTER TABLE payment_links_company
    ADD COLUMN IF NOT EXISTS signed_token TEXT,
    ADD COLUMN IF NOT EXISTS checkout_url TEXT;

ALTER TABLE payment_sessions_company
    ADD COLUMN IF NOT EXISTS provider_session_id TEXT,
    ADD COLUMN IF NOT EXISTS provider_checkout_url TEXT,
    ADD COLUMN IF NOT EXISTS payment_order_id UUID;

ALTER TABLE checkout_receipts_company
    ADD COLUMN IF NOT EXISTS receipt_token TEXT UNIQUE;

CREATE TABLE IF NOT EXISTS payout_batches_company (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    currency VARCHAR(8) NOT NULL,
    total_amount BIGINT NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL DEFAULT 'queued',
    created_by_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS payout_batch_items_company (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES payout_batches_company(id) ON DELETE CASCADE,
    payout_id UUID REFERENCES payouts_company(id) ON DELETE SET NULL,
    business_account_id UUID,
    amount BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'queued',
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS provider_report_imports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID,
    report_type VARCHAR(64) NOT NULL,
    file_name TEXT NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    status VARCHAR(32) NOT NULL DEFAULT 'uploaded',
    row_count INTEGER NOT NULL DEFAULT 0,
    imported_count INTEGER NOT NULL DEFAULT 0,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    uploaded_by_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS provider_report_rows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_id UUID NOT NULL REFERENCES provider_report_imports(id) ON DELETE CASCADE,
    row_number INTEGER NOT NULL,
    external_id TEXT,
    external_type TEXT,
    amount BIGINT,
    currency VARCHAR(8),
    fee_amount BIGINT,
    net_amount BIGINT,
    settlement_reference TEXT,
    payout_reference TEXT,
    raw_row_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,
    match_status VARCHAR(32) NOT NULL DEFAULT 'unmatched',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS report_parser_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID,
    report_type VARCHAR(64) NOT NULL,
    column_mapping_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    date_format VARCHAR(64),
    amount_format VARCHAR(64),
    currency_column VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(128) NOT NULL UNIQUE,
    title TEXT NOT NULL,
    category VARCHAR(64) NOT NULL,
    description TEXT,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    owner_admin_id UUID,
    evidence_required BOOLEAN NOT NULL DEFAULT true,
    last_reviewed_at TIMESTAMPTZ,
    next_review_due_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id UUID NOT NULL REFERENCES compliance_controls(id) ON DELETE CASCADE,
    file_reference TEXT NOT NULL,
    file_name TEXT NOT NULL,
    uploaded_by_admin_id UUID,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE access_reviews
    ADD COLUMN IF NOT EXISTS due_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS scope TEXT,
    ADD COLUMN IF NOT EXISTS reviewer_admin_id UUID;

ALTER TABLE access_review_items
    ADD COLUMN IF NOT EXISTS current_permissions_json JSONB,
    ADD COLUMN IF NOT EXISTS recommended_action TEXT,
    ADD COLUMN IF NOT EXISTS risk_reason TEXT;

CREATE TABLE IF NOT EXISTS compliance_packet_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'queued',
    requested_by_admin_id UUID,
    file_reference TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS metric_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(128) NOT NULL,
    labels_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    value DOUBLE PRECISION NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS slo_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) NOT NULL UNIQUE,
    target_percent DOUBLE PRECISION NOT NULL,
    window_minutes INTEGER NOT NULL,
    metric_query_key VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS slo_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slo_id UUID NOT NULL REFERENCES slo_definitions(id) ON DELETE CASCADE,
    current_percent DOUBLE PRECISION NOT NULL,
    error_budget_remaining DOUBLE PRECISION NOT NULL,
    status VARCHAR(32) NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trace_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trace_id VARCHAR(128) NOT NULL,
    request_id VARCHAR(128),
    span_name VARCHAR(128) NOT NULL,
    duration_ms INTEGER NOT NULL,
    status VARCHAR(32) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO compliance_controls (code, title, category, description, status, evidence_required)
VALUES
    ('no_raw_card_storage', 'No raw card storage', 'PCI', 'Raw card PAN never stored', 'pass', true),
    ('no_cvv_storage', 'No CVV storage', 'PCI', 'CVV/CVC never persisted', 'pass', true),
    ('tokenization_enforced', 'Tokenization enforced', 'PCI', 'Tokenized flows in use', 'pass', true),
    ('secrets_encrypted', 'Secrets encrypted', 'Security', 'Secrets encrypted at rest', 'pass', true),
    ('bank_data_encrypted', 'Bank data encrypted', 'Security', 'Bank account values encrypted/masked', 'pass', true),
    ('webhook_signature_required', 'Webhook signatures required', 'Security', 'Webhook signature verification mandatory', 'pass', true),
    ('rbac_deny_default', 'RBAC deny default', 'Access', 'Unmapped admin routes denied', 'pass', true),
    ('mfa_required', 'MFA required', 'Access', 'MFA enforced for admins', 'warning', true),
    ('audit_logs_enabled', 'Audit logs enabled', 'Audit', 'Sensitive actions audited', 'pass', true),
    ('export_masking_enabled', 'Export masking enabled', 'Audit', 'Exports mask sensitive data', 'pass', true),
    ('incident_workflow_exists', 'Incident workflow exists', 'Ops', 'Incident and SLA operations configured', 'warning', false),
    ('access_review_completed', 'Access review completed', 'Access', 'Periodic access review completed', 'warning', true)
ON CONFLICT (code) DO NOTHING;

INSERT INTO slo_definitions (name, target_percent, window_minutes, metric_query_key, status)
VALUES
    ('API uptime', 99.90, 1440, 'api_uptime', 'active'),
    ('Payment success SLO', 98.50, 1440, 'payment_success_rate', 'active'),
    ('Webhook processing SLO', 99.00, 1440, 'webhook_processing_success', 'active')
ON CONFLICT (name) DO NOTHING;
