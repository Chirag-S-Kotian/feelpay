CREATE TABLE IF NOT EXISTS payment_links_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID REFERENCES merchants(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(8) NOT NULL DEFAULT 'INR',
    description TEXT,
    customer_email TEXT,
    customer_phone TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    expires_at TIMESTAMPTZ,
    signed_token_hash TEXT NOT NULL,
    created_by_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_sessions_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_link_id UUID REFERENCES payment_links_company(id) ON DELETE SET NULL,
    payment_order_id UUID REFERENCES payment_orders(id) ON DELETE SET NULL,
    provider_id UUID REFERENCES payment_providers(id) ON DELETE SET NULL,
    selected_payment_method VARCHAR(40),
    status VARCHAR(30) NOT NULL DEFAULT 'created',
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS checkout_theme_snapshots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_session_id UUID NOT NULL REFERENCES payment_sessions_company(id) ON DELETE CASCADE,
    brand_name TEXT NOT NULL,
    logo_url TEXT,
    theme_color TEXT NOT NULL DEFAULT '#16A34A',
    support_email TEXT,
    terms_url TEXT,
    privacy_url TEXT,
    refund_policy_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS checkout_receipts_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_session_id UUID REFERENCES payment_sessions_company(id) ON DELETE SET NULL,
    payment_order_id UUID REFERENCES payment_orders(id) ON DELETE SET NULL,
    receipt_number VARCHAR(120) NOT NULL,
    recipient_email TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'generated',
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS split_rules_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    business_account_id UUID REFERENCES merchants(id),
    rule_type VARCHAR(20) NOT NULL DEFAULT 'percent',
    company_fee_percent NUMERIC(8,4) NOT NULL DEFAULT 0,
    company_fee_fixed BIGINT NOT NULL DEFAULT 0,
    tax_percent NUMERIC(8,4) NOT NULL DEFAULT 0,
    reserve_percent NUMERIC(8,4) NOT NULL DEFAULT 0,
    settlement_delay_days INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    priority INTEGER NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_splits_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id UUID NOT NULL REFERENCES payment_orders(id) ON DELETE CASCADE,
    business_account_id UUID REFERENCES merchants(id),
    gross_amount BIGINT NOT NULL,
    company_fee_amount BIGINT NOT NULL DEFAULT 0,
    provider_fee_amount BIGINT NOT NULL DEFAULT 0,
    tax_amount BIGINT NOT NULL DEFAULT 0,
    reserve_amount BIGINT NOT NULL DEFAULT 0,
    payable_amount BIGINT NOT NULL DEFAULT 0,
    currency VARCHAR(8) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'created',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transfers_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_split_id UUID REFERENCES payment_splits_company(id) ON DELETE SET NULL,
    provider_id UUID REFERENCES payment_providers(id),
    provider_transfer_id TEXT,
    business_account_id UUID REFERENCES merchants(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(8) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    failure_reason TEXT,
    ledger_transaction_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transfer_reversals_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transfer_id UUID REFERENCES transfers_company(id) ON DELETE CASCADE,
    provider_reversal_id TEXT,
    reason TEXT,
    amount BIGINT NOT NULL,
    currency VARCHAR(8) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    ledger_transaction_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payouts_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID REFERENCES merchants(id),
    provider_id UUID REFERENCES payment_providers(id),
    bank_account_id UUID,
    amount BIGINT NOT NULL,
    currency VARCHAR(8) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'scheduled',
    scheduled_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    provider_payout_id TEXT,
    failure_reason TEXT,
    ledger_transaction_id UUID,
    idempotency_key TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_signals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id UUID REFERENCES payment_orders(id),
    business_account_id UUID REFERENCES merchants(id),
    signal_type VARCHAR(80) NOT NULL,
    signal_value TEXT,
    severity VARCHAR(20) NOT NULL DEFAULT 'info',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id UUID REFERENCES payment_orders(id),
    business_account_id UUID REFERENCES merchants(id),
    decision VARCHAR(20) NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    matched_rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_lists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    list_type VARCHAR(20) NOT NULL,
    entity_type VARCHAR(40) NOT NULL,
    value_hash TEXT NOT NULL,
    display_value_masked TEXT NOT NULL,
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE risk_reviews
    ADD COLUMN IF NOT EXISTS assigned_to_admin_id UUID REFERENCES admin_users(id),
    ADD COLUMN IF NOT EXISTS decision_note TEXT,
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ;

ALTER TABLE payment_webhook_events
    ADD COLUMN IF NOT EXISTS event_version VARCHAR(20),
    ADD COLUMN IF NOT EXISTS payload_hash TEXT,
    ADD COLUMN IF NOT EXISTS dedupe_key TEXT,
    ADD COLUMN IF NOT EXISTS idempotency_key TEXT,
    ADD COLUMN IF NOT EXISTS processing_started_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS processing_finished_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS next_retry_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS dead_letter_reason TEXT;

CREATE TABLE IF NOT EXISTS webhook_processing_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    webhook_event_id UUID NOT NULL REFERENCES payment_webhook_events(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    status VARCHAR(30) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    error_message TEXT,
    worker_id TEXT
);

CREATE TABLE IF NOT EXISTS webhook_retry_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID REFERENCES payment_providers(id),
    max_attempts INTEGER NOT NULL DEFAULT 6,
    backoff_strategy VARCHAR(20) NOT NULL DEFAULT 'exponential',
    initial_delay_seconds INTEGER NOT NULL DEFAULT 60,
    max_delay_seconds INTEGER NOT NULL DEFAULT 3600,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id)
);

CREATE INDEX IF NOT EXISTS idx_payment_links_company_status ON payment_links_company(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_sessions_company_status ON payment_sessions_company(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transfers_company_status ON transfers_company(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payouts_company_status ON payouts_company(status, scheduled_at);
CREATE INDEX IF NOT EXISTS idx_risk_lists_lookup ON risk_lists(list_type, entity_type, status);
CREATE INDEX IF NOT EXISTS idx_webhook_attempts_event ON webhook_processing_attempts(webhook_event_id, attempt_number DESC);
