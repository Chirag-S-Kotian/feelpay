-- Company admin payment operations module.
-- Internal platform controls only. No merchant/seller/vendor admin access.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

ALTER TABLE payment_providers
    ADD COLUMN IF NOT EXISTS code VARCHAR(64),
    ADD COLUMN IF NOT EXISTS provider_type VARCHAR(40) DEFAULT 'mock',
    ADD COLUMN IF NOT EXISTS environment VARCHAR(20) DEFAULT 'sandbox',
    ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 100,
    ADD COLUMN IF NOT EXISTS failover_enabled BOOLEAN DEFAULT true,
    ADD COLUMN IF NOT EXISTS webhook_url TEXT,
    ADD COLUMN IF NOT EXISTS webhook_secret_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS api_key_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS api_secret_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS credential_nonce TEXT,
    ADD COLUMN IF NOT EXISTS supported_payment_methods JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS supported_currencies JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS supported_countries JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS health_status VARCHAR(20) DEFAULT 'unknown',
    ADD COLUMN IF NOT EXISTS last_health_check_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_successful_webhook_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_failed_webhook_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_failed_payment_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_connection_test_at TIMESTAMPTZ;

UPDATE payment_providers
SET code = COALESCE(code, name),
    provider_type = COALESCE(provider_type, name),
    status = CASE WHEN COALESCE(is_active, true) THEN COALESCE(status, 'active') ELSE 'disabled' END
WHERE code IS NULL OR provider_type IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_payment_providers_code ON payment_providers(code);
CREATE INDEX IF NOT EXISTS idx_payment_providers_company_status ON payment_providers(status, priority);

CREATE TABLE IF NOT EXISTS admin_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(80) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_role_permissions (
    role_id UUID NOT NULL REFERENCES admin_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES admin_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    role_id UUID REFERENCES admin_roles(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_provider_credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id) ON DELETE CASCADE,
    key_name VARCHAR(80) NOT NULL,
    encrypted_value TEXT NOT NULL,
    nonce TEXT NOT NULL,
    masked_value VARCHAR(32) NOT NULL,
    rotated_by UUID,
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, key_name)
);

CREATE TABLE IF NOT EXISTS bank_integration_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    acquiring_bank_name VARCHAR(180) NOT NULL,
    integration_type VARCHAR(30) NOT NULL CHECK (integration_type IN ('api', 'sftp', 'manual_file_upload', 'provider_managed')),
    settlement_account_reference VARCHAR(120),
    encrypted_account_details TEXT,
    account_details_nonce TEXT,
    bank_account_holder_name VARCHAR(180),
    masked_account_number VARCHAR(40),
    routing_code VARCHAR(80),
    settlement_cycle VARCHAR(80),
    supported_currencies JSONB NOT NULL DEFAULT '[]'::jsonb,
    supported_payment_methods JSONB NOT NULL DEFAULT '[]'::jsonb,
    file_format_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    reconciliation_parser_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    contact_reference JSONB NOT NULL DEFAULT '{}'::jsonb,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS white_label_payment_config (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    brand_name VARCHAR(180) NOT NULL DEFAULT 'FeelPay',
    checkout_logo_url TEXT,
    checkout_theme_color VARCHAR(20) NOT NULL DEFAULT '#1DB954',
    support_email VARCHAR(255),
    support_phone VARCHAR(40),
    payment_page_domain VARCHAR(255),
    default_currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    default_country VARCHAR(2) NOT NULL DEFAULT 'IN',
    default_payment_methods JSONB NOT NULL DEFAULT '["upi","card","netbanking"]'::jsonb,
    default_provider_id UUID REFERENCES payment_providers(id),
    fallback_provider_id UUID REFERENCES payment_providers(id),
    invoice_prefix VARCHAR(30) NOT NULL DEFAULT 'INV',
    receipt_prefix VARCHAR(30) NOT NULL DEFAULT 'RCPT',
    statement_descriptor VARCHAR(80),
    terms_url TEXT,
    privacy_url TEXT,
    refund_policy_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_routing_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(180) NOT NULL,
    payment_method VARCHAR(40),
    currency VARCHAR(3),
    country VARCHAR(2),
    amount_min BIGINT,
    amount_max BIGINT,
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    bank_integration_id UUID REFERENCES bank_integration_configs(id),
    priority INTEGER NOT NULL DEFAULT 100,
    risk_level VARCHAR(30),
    customer_region VARCHAR(80),
    business_line VARCHAR(120),
    failover_provider_id UUID REFERENCES payment_providers(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_routing_decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_order_id UUID REFERENCES payment_orders(id),
    routing_rule_id UUID REFERENCES payment_routing_rules(id),
    selected_provider_id UUID REFERENCES payment_providers(id),
    failover_provider_id UUID REFERENCES payment_providers(id),
    decision_reason TEXT NOT NULL,
    provider_failure_history JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payment_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_order_id UUID REFERENCES payment_orders(id),
    provider_id UUID REFERENCES payment_providers(id),
    routing_decision_id UUID REFERENCES payment_routing_decisions(id),
    status VARCHAR(30) NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    payment_method VARCHAR(40),
    provider_order_id VARCHAR(255),
    provider_payment_id VARCHAR(255),
    failure_reason TEXT,
    provider_response_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    idempotency_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(idempotency_key)
);

CREATE TABLE IF NOT EXISTS payment_webhook_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    event_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(120) NOT NULL,
    raw_payload JSONB NOT NULL,
    headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    signature_verification_status VARCHAR(30) NOT NULL,
    processing_status VARCHAR(30) NOT NULL DEFAULT 'queued',
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    related_payment_id UUID REFERENCES payment_orders(id),
    related_refund_id UUID REFERENCES refunds(id),
    related_settlement_id UUID REFERENCES settlements(id),
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    UNIQUE(provider_id, event_id)
);

CREATE TABLE IF NOT EXISTS payment_refunds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_order_id UUID REFERENCES payment_orders(id),
    provider_id UUID REFERENCES payment_providers(id),
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'requested',
    reason TEXT,
    refund_source VARCHAR(40) NOT NULL DEFAULT 'company',
    provider_refund_id VARCHAR(255),
    idempotency_key VARCHAR(255) NOT NULL,
    requested_by UUID,
    approved_by UUID,
    provider_response_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(idempotency_key)
);

CREATE TABLE IF NOT EXISTS payment_transfers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID REFERENCES payment_providers(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    recipient_reference VARCHAR(255),
    provider_transfer_id VARCHAR(255),
    status VARCHAR(30) NOT NULL DEFAULT 'created',
    idempotency_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(idempotency_key)
);

CREATE TABLE IF NOT EXISTS settlement_batches_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID REFERENCES payment_providers(id),
    bank_integration_id UUID REFERENCES bank_integration_configs(id),
    provider_settlement_reference VARCHAR(255),
    bank_reference_number VARCHAR(255),
    gross_payment_amount BIGINT NOT NULL DEFAULT 0,
    provider_fee BIGINT NOT NULL DEFAULT 0,
    platform_fee BIGINT NOT NULL DEFAULT 0,
    tax BIGINT NOT NULL DEFAULT 0,
    refund_adjustments BIGINT NOT NULL DEFAULT 0,
    chargeback_adjustments BIGINT NOT NULL DEFAULT 0,
    net_settlement_amount BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    settlement_date DATE,
    expected_settlement_date DATE,
    actual_settlement_date DATE,
    provider_report_reference VARCHAR(255),
    finance_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS settlement_items_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    settlement_batch_id UUID NOT NULL REFERENCES settlement_batches_company(id) ON DELETE CASCADE,
    payment_order_id UUID REFERENCES payment_orders(id),
    refund_id UUID REFERENCES payment_refunds(id),
    gross_amount BIGINT NOT NULL DEFAULT 0,
    fee_amount BIGINT NOT NULL DEFAULT 0,
    tax_amount BIGINT NOT NULL DEFAULT 0,
    net_amount BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reconciliation_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_type VARCHAR(40) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    source VARCHAR(40) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'uploaded',
    uploaded_by UUID,
    summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reconciliation_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reconciliation_file_id UUID NOT NULL REFERENCES reconciliation_files(id) ON DELETE CASCADE,
    provider_payment_id VARCHAR(255),
    provider_order_id VARCHAR(255),
    internal_order_id UUID,
    internal_payment_id UUID,
    settlement_reference VARCHAR(255),
    bank_utr VARCHAR(255),
    transfer_id UUID,
    refund_id UUID,
    expected_amount BIGINT,
    observed_amount BIGINT,
    status VARCHAR(30) NOT NULL DEFAULT 'unmatched',
    mismatch_reason TEXT,
    resolution_notes TEXT,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE reconciliation_records
    ADD COLUMN IF NOT EXISTS reconciliation_file_id UUID REFERENCES reconciliation_files(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS provider_payment_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS provider_order_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS internal_order_id UUID,
    ADD COLUMN IF NOT EXISTS internal_payment_id UUID,
    ADD COLUMN IF NOT EXISTS settlement_reference VARCHAR(255),
    ADD COLUMN IF NOT EXISTS bank_utr VARCHAR(255),
    ADD COLUMN IF NOT EXISTS transfer_id UUID,
    ADD COLUMN IF NOT EXISTS refund_id UUID,
    ADD COLUMN IF NOT EXISTS expected_amount BIGINT,
    ADD COLUMN IF NOT EXISTS observed_amount BIGINT,
    ADD COLUMN IF NOT EXISTS mismatch_reason TEXT,
    ADD COLUMN IF NOT EXISTS resolution_notes TEXT,
    ADD COLUMN IF NOT EXISTS resolved_by UUID,
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TABLE IF NOT EXISTS disputes_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_dispute_id VARCHAR(255),
    related_payment_id UUID REFERENCES payment_orders(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    reason TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'opened',
    evidence_due_date DATE,
    evidence_files_metadata JSONB NOT NULL DEFAULT '[]'::jsonb,
    provider_response JSONB NOT NULL DEFAULT '{}'::jsonb,
    internal_owner UUID,
    internal_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chargebacks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    dispute_id UUID REFERENCES disputes_company(id),
    settlement_batch_id UUID REFERENCES settlement_batches_company(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    status VARCHAR(30) NOT NULL DEFAULT 'opened',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(180) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,
    condition JSONB NOT NULL,
    action VARCHAR(20) NOT NULL CHECK (action IN ('allow', 'review', 'block')),
    priority INTEGER NOT NULL DEFAULT 100,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_order_id UUID REFERENCES payment_orders(id),
    risk_rule_id UUID REFERENCES risk_rules(id),
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    risk_level VARCHAR(30) NOT NULL DEFAULT 'medium',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_logs_company (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    actor_admin_id UUID,
    actor_role VARCHAR(80),
    action VARCHAR(120) NOT NULL,
    entity_type VARCHAR(80) NOT NULL,
    entity_id UUID,
    old_value_summary JSONB,
    new_value_summary JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payment_routing_rules_lookup ON payment_routing_rules(status, priority, payment_method, currency, country);
CREATE INDEX IF NOT EXISTS idx_payment_attempts_order ON payment_attempts(payment_order_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_webhook_events_status ON payment_webhook_events(processing_status, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_payment_refunds_status ON payment_refunds(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_settlement_batches_company_status ON settlement_batches_company(status, expected_settlement_date);
CREATE INDEX IF NOT EXISTS idx_reconciliation_records_status ON reconciliation_records(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_rules_active ON risk_rules(status, priority);
CREATE INDEX IF NOT EXISTS idx_audit_logs_company_entity ON audit_logs_company(entity_type, entity_id, created_at DESC);

INSERT INTO admin_roles (name, description) VALUES
    ('Super Admin', 'Full internal company admin access'),
    ('Payment Ops Admin', 'Provider, routing, webhook, and transaction operations'),
    ('Finance Admin', 'Settlements, refunds, reconciliation, finance exports'),
    ('Risk Admin', 'Risk rules and review queue'),
    ('Support Admin', 'Support operations and refund requests'),
    ('Support Agent', 'Read transactions and request refunds'),
    ('Compliance Admin', 'Disputes, audit, compliance review'),
    ('Auditor / Read Only', 'Read-only audit and operations access')
ON CONFLICT (name) DO NOTHING;

INSERT INTO admin_permissions (code, description) VALUES
    ('provider.view', 'View payment providers'),
    ('provider.manage', 'Manage payment providers'),
    ('provider.credentials.rotate', 'Rotate provider credentials'),
    ('provider.credentials.view_masked', 'View masked provider credentials'),
    ('bank_config.view', 'View bank configs'),
    ('bank_config.manage', 'Manage bank configs'),
    ('routing.view', 'View routing rules'),
    ('routing.manage', 'Manage routing rules'),
    ('transaction.view', 'View transactions'),
    ('transaction.action', 'Operate payment transactions'),
    ('refund.request', 'Request refunds'),
    ('refund.approve', 'Approve refunds'),
    ('refund.initiate', 'Initiate refunds'),
    ('settlement.view', 'View settlements'),
    ('settlement.manage', 'Manage settlements'),
    ('settlement.export', 'Export settlements'),
    ('reconciliation.view', 'View reconciliation'),
    ('reconciliation.manage', 'Manage reconciliation'),
    ('webhook.view', 'View webhooks'),
    ('webhook.retry', 'Retry webhooks'),
    ('risk.view', 'View risk rules'),
    ('risk.manage', 'Manage risk rules'),
    ('dispute.view', 'View disputes'),
    ('dispute.manage', 'Manage disputes'),
    ('audit.view', 'View audit logs'),
    ('white_label_config.manage', 'Manage white-label payment settings')
ON CONFLICT (code) DO NOTHING;

INSERT INTO payment_providers (name, display_name, code, provider_type, environment, status, priority, is_active, is_default, supported_payment_methods, supported_currencies, supported_countries, health_status)
VALUES
    ('mock', 'Mock Provider', 'mock', 'mock', 'sandbox', 'active', 1, true, false, '["card","upi","netbanking","wallet"]'::jsonb, '["INR","USD"]'::jsonb, '["IN","US"]'::jsonb, 'healthy'),
    ('adyen', 'Adyen', 'adyen', 'adyen', 'sandbox', 'disabled', 80, false, false, '["card","wallet"]'::jsonb, '["INR","USD","EUR"]'::jsonb, '["IN","US","NL"]'::jsonb, 'unknown'),
    ('stripe', 'Stripe', 'stripe', 'stripe', 'sandbox', 'disabled', 70, false, false, '["card","wallet"]'::jsonb, '["INR","USD"]'::jsonb, '["IN","US"]'::jsonb, 'unknown'),
    ('manual_bank', 'Manual Bank / Acquirer', 'manual_bank', 'manual_bank', 'production', 'maintenance', 90, false, false, '["netbanking","bank_transfer"]'::jsonb, '["INR"]'::jsonb, '["IN"]'::jsonb, 'unknown')
ON CONFLICT (name) DO NOTHING;

INSERT INTO white_label_payment_config (brand_name)
SELECT 'FeelPay'
WHERE NOT EXISTS (SELECT 1 FROM white_label_payment_config);
