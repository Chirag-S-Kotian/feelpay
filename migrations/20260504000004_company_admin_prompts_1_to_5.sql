-- Company admin hardening for UX, ledger, routing, business accounts, refunds, disputes.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE ledger_accounts
    ADD COLUMN IF NOT EXISTS owner_type VARCHAR(40) NOT NULL DEFAULT 'company',
    ADD COLUMN IF NOT EXISTS status VARCHAR(30) NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE ledger_entries
    ADD COLUMN IF NOT EXISTS transaction_group_id UUID,
    ADD COLUMN IF NOT EXISTS source_type VARCHAR(60),
    ADD COLUMN IF NOT EXISTS source_id UUID,
    ADD COLUMN IF NOT EXISTS direction VARCHAR(10),
    ADD COLUMN IF NOT EXISTS balance_after BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS status VARCHAR(30) NOT NULL DEFAULT 'posted';

UPDATE ledger_entries
SET transaction_group_id = COALESCE(transaction_group_id, transaction_id),
    direction = COALESCE(direction, entry_type)
WHERE transaction_group_id IS NULL OR direction IS NULL;

CREATE TABLE IF NOT EXISTS ledger_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_group_id UUID NOT NULL,
    type VARCHAR(60) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'posted',
    source_type VARCHAR(60),
    source_id UUID,
    provider_id UUID REFERENCES payment_providers(id),
    business_account_id UUID REFERENCES merchants(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    idempotency_key VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reserve_balances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID NOT NULL REFERENCES merchants(id),
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    reserve_type VARCHAR(30) NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0,
    release_at TIMESTAMPTZ,
    status VARCHAR(30) NOT NULL DEFAULT 'held',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE payment_routing_rules
    ADD COLUMN IF NOT EXISTS rule_group VARCHAR(120),
    ADD COLUMN IF NOT EXISTS provider_weight INTEGER NOT NULL DEFAULT 100,
    ADD COLUMN IF NOT EXISTS payment_method_priority INTEGER NOT NULL DEFAULT 100,
    ADD COLUMN IF NOT EXISTS min_success_rate DECIMAL(5,2),
    ADD COLUMN IF NOT EXISTS max_provider_fee_percent DECIMAL(6,3),
    ADD COLUMN IF NOT EXISTS max_latency_ms INTEGER,
    ADD COLUMN IF NOT EXISTS health_required BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS failover_strategy VARCHAR(40) NOT NULL DEFAULT 'fallback_only',
    ADD COLUMN IF NOT EXISTS retry_strategy VARCHAR(40) NOT NULL DEFAULT 'none',
    ADD COLUMN IF NOT EXISTS cooldown_minutes INTEGER NOT NULL DEFAULT 15,
    ADD COLUMN IF NOT EXISTS effective_from TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS effective_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS strategy VARCHAR(40) NOT NULL DEFAULT 'priority';

CREATE TABLE IF NOT EXISTS provider_performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    payment_method VARCHAR(40),
    currency VARCHAR(3),
    country VARCHAR(2),
    success_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    failure_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    avg_latency_ms INTEGER NOT NULL DEFAULT 0,
    p95_latency_ms INTEGER NOT NULL DEFAULT 0,
    provider_fee_percent DECIMAL(6,3) NOT NULL DEFAULT 0,
    attempt_count BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, payment_method, currency, country)
);

ALTER TABLE payment_routing_decisions
    ADD COLUMN IF NOT EXISTS payment_id UUID,
    ADD COLUMN IF NOT EXISTS selected_provider_id UUID REFERENCES payment_providers(id),
    ADD COLUMN IF NOT EXISTS fallback_provider_id UUID REFERENCES payment_providers(id),
    ADD COLUMN IF NOT EXISTS rule_id UUID REFERENCES payment_routing_rules(id),
    ADD COLUMN IF NOT EXISTS provider_scores_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS risk_result VARCHAR(40),
    ADD COLUMN IF NOT EXISTS failover_applied BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(255);

ALTER TABLE merchants
    ADD COLUMN IF NOT EXISTS display_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS legal_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS country VARCHAR(2) NOT NULL DEFAULT 'IN',
    ADD COLUMN IF NOT EXISTS kyc_status VARCHAR(40) NOT NULL DEFAULT 'not_started',
    ADD COLUMN IF NOT EXISTS risk_status VARCHAR(40) NOT NULL DEFAULT 'low',
    ADD COLUMN IF NOT EXISTS payout_status VARCHAR(40) NOT NULL DEFAULT 'not_configured',
    ADD COLUMN IF NOT EXISTS default_currency VARCHAR(3) NOT NULL DEFAULT 'INR';

UPDATE merchants
SET display_name = COALESCE(display_name, business_name, name),
    legal_name = COALESCE(legal_name, business_name, name),
    kyc_status = COALESCE(NULLIF(kyc_status, ''), 'not_started'),
    risk_status = COALESCE(NULLIF(risk_status, ''), 'low'),
    payout_status = COALESCE(NULLIF(payout_status, ''), 'not_configured')
WHERE display_name IS NULL OR legal_name IS NULL;

CREATE TABLE IF NOT EXISTS business_account_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    document_type VARCHAR(80) NOT NULL,
    file_reference TEXT NOT NULL,
    status VARCHAR(40) NOT NULL DEFAULT 'pending',
    rejection_reason TEXT,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_by_admin_id UUID,
    reviewed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS business_account_bank_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    account_holder_name VARCHAR(180),
    bank_name VARCHAR(180),
    encrypted_account_number TEXT,
    masked_account_number VARCHAR(40) NOT NULL,
    ifsc_or_swift_or_routing_code VARCHAR(80),
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    verification_status VARCHAR(40) NOT NULL DEFAULT 'pending_verification',
    status VARCHAR(40) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS business_account_provider_links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    provider_account_id VARCHAR(255),
    onboarding_status VARCHAR(40) NOT NULL DEFAULT 'pending',
    capabilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    payouts_enabled BOOLEAN NOT NULL DEFAULT false,
    charges_enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS business_account_risk_notes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_account_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    note TEXT NOT NULL,
    risk_status VARCHAR(40),
    created_by_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE payment_refunds
    ADD COLUMN IF NOT EXISTS refund_type VARCHAR(30) NOT NULL DEFAULT 'full',
    ADD COLUMN IF NOT EXISTS approval_status VARCHAR(30) NOT NULL DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS failure_reason TEXT,
    ADD COLUMN IF NOT EXISTS reversal_status VARCHAR(30) NOT NULL DEFAULT 'none',
    ADD COLUMN IF NOT EXISTS settlement_adjustment_id UUID,
    ADD COLUMN IF NOT EXISTS ledger_transaction_id UUID REFERENCES ledger_transactions(id),
    ADD COLUMN IF NOT EXISTS business_account_id UUID REFERENCES merchants(id);

ALTER TABLE disputes_company
    ADD COLUMN IF NOT EXISTS business_account_id UUID REFERENCES merchants(id),
    ADD COLUMN IF NOT EXISTS payment_id UUID,
    ADD COLUMN IF NOT EXISTS evidence_status VARCHAR(40) NOT NULL DEFAULT 'required',
    ADD COLUMN IF NOT EXISTS outcome VARCHAR(40),
    ADD COLUMN IF NOT EXISTS chargeback_fee BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS ledger_transaction_id UUID REFERENCES ledger_transactions(id),
    ADD COLUMN IF NOT EXISTS settlement_adjustment_id UUID,
    ADD COLUMN IF NOT EXISTS timeline JSONB NOT NULL DEFAULT '[]'::jsonb;

INSERT INTO admin_permissions (code, description) VALUES
    ('ledger.view', 'View company ledger'),
    ('ledger.export', 'Export company ledger'),
    ('business_accounts.manage', 'Manage business accounts'),
    ('business_accounts.kyc_review', 'Review business account KYC'),
    ('business_accounts.risk_review', 'Review business account risk'),
    ('business_accounts.payouts_manage', 'Manage business account payout readiness')
ON CONFLICT (code) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_ledger_entries_group ON ledger_entries(transaction_group_id);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_source ON ledger_entries(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_ledger_transactions_source ON ledger_transactions(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_provider_performance_lookup ON provider_performance_metrics(provider_id, payment_method, currency, country);
CREATE INDEX IF NOT EXISTS idx_payment_routing_decisions_payment ON payment_routing_decisions(payment_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_business_account_documents_status ON business_account_documents(status, uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_refunds_workflow ON payment_refunds(status, approval_status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_disputes_company_workflow ON disputes_company(status, evidence_status, created_at DESC);
