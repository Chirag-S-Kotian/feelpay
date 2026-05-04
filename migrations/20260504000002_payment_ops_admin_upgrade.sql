-- Upgrade company-only Payment Operations admin module.

ALTER TABLE payment_provider_credentials
    ADD COLUMN IF NOT EXISTS environment VARCHAR(20) NOT NULL DEFAULT 'sandbox',
    ADD COLUMN IF NOT EXISTS secret_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS last_validated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_rotated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rotation_due_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS description TEXT,
    ADD COLUMN IF NOT EXISTS created_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS updated_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

UPDATE payment_provider_credentials
SET secret_type = COALESCE(secret_type, key_name),
    last_rotated_at = COALESCE(last_rotated_at, rotated_at)
WHERE secret_type IS NULL OR last_rotated_at IS NULL;

CREATE TABLE IF NOT EXISTS webhook_endpoint_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id) ON DELETE CASCADE,
    environment VARCHAR(20) NOT NULL DEFAULT 'sandbox',
    webhook_url TEXT NOT NULL,
    encrypted_signing_secret TEXT,
    signing_secret_nonce TEXT,
    masked_signing_secret VARCHAR(32) NOT NULL DEFAULT 'Missing',
    signature_verification_enabled BOOLEAN NOT NULL DEFAULT true,
    last_event_received_at TIMESTAMPTZ,
    failed_event_count INTEGER NOT NULL DEFAULT 0,
    retry_queue_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, environment)
);

ALTER TABLE bank_integration_configs
    ADD COLUMN IF NOT EXISTS account_scope VARCHAR(40) NOT NULL DEFAULT 'company_settlement',
    ADD COLUMN IF NOT EXISTS encrypted_account_number TEXT,
    ADD COLUMN IF NOT EXISTS account_number_nonce TEXT,
    ADD COLUMN IF NOT EXISTS ifsc_or_swift_or_routing_code VARCHAR(80),
    ADD COLUMN IF NOT EXISTS currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    ADD COLUMN IF NOT EXISTS verification_status VARCHAR(30) NOT NULL DEFAULT 'unverified',
    ADD COLUMN IF NOT EXISTS last_verified_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS created_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS updated_by_admin_id UUID;

UPDATE bank_integration_configs
SET ifsc_or_swift_or_routing_code = COALESCE(ifsc_or_swift_or_routing_code, routing_code)
WHERE ifsc_or_swift_or_routing_code IS NULL;

ALTER TABLE payment_routing_rules
    ADD COLUMN IF NOT EXISTS fallback_provider_id UUID REFERENCES payment_providers(id),
    ADD COLUMN IF NOT EXISTS last_matched_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS created_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS updated_by_admin_id UUID;

UPDATE payment_routing_rules
SET fallback_provider_id = COALESCE(fallback_provider_id, failover_provider_id)
WHERE fallback_provider_id IS NULL;

ALTER TABLE payment_webhook_events
    ADD COLUMN IF NOT EXISTS provider_event_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

UPDATE payment_webhook_events
SET provider_event_id = COALESCE(provider_event_id, event_id)
WHERE provider_event_id IS NULL;

ALTER TABLE payment_refunds
    ADD COLUMN IF NOT EXISTS payment_id UUID,
    ADD COLUMN IF NOT EXISTS refund_type VARCHAR(30) NOT NULL DEFAULT 'full',
    ADD COLUMN IF NOT EXISTS requested_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS approved_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS initiated_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS provider_response JSONB NOT NULL DEFAULT '{}'::jsonb;

UPDATE payment_refunds
SET payment_id = COALESCE(payment_id, payment_order_id),
    requested_by_admin_id = COALESCE(requested_by_admin_id, requested_by),
    approved_by_admin_id = COALESCE(approved_by_admin_id, approved_by),
    provider_response = COALESCE(provider_response, provider_response_summary, '{}'::jsonb)
WHERE payment_id IS NULL OR provider_response = '{}'::jsonb;

ALTER TABLE settlement_batches_company
    ADD COLUMN IF NOT EXISTS provider_fee_amount BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS company_fee_amount BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS refund_adjustment_amount BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS chargeback_adjustment_amount BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    ADD COLUMN IF NOT EXISTS reconciliation_status VARCHAR(30) NOT NULL DEFAULT 'unmatched';

UPDATE settlement_batches_company
SET provider_fee_amount = COALESCE(NULLIF(provider_fee_amount, 0), provider_fee),
    company_fee_amount = COALESCE(NULLIF(company_fee_amount, 0), platform_fee),
    refund_adjustment_amount = COALESCE(NULLIF(refund_adjustment_amount, 0), refund_adjustments),
    chargeback_adjustment_amount = COALESCE(NULLIF(chargeback_adjustment_amount, 0), chargeback_adjustments);

ALTER TABLE risk_rules
    ADD COLUMN IF NOT EXISTS trigger_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_triggered_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS created_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS updated_by_admin_id UUID;

CREATE INDEX IF NOT EXISTS idx_payment_provider_credentials_provider ON payment_provider_credentials(provider_id, environment, secret_type);
CREATE INDEX IF NOT EXISTS idx_webhook_endpoint_configs_provider ON webhook_endpoint_configs(provider_id, environment);
CREATE INDEX IF NOT EXISTS idx_payment_webhook_events_provider_event ON payment_webhook_events(provider_id, provider_event_id);

INSERT INTO admin_permissions (code, description) VALUES
    ('payment_ops.view', 'View company payment operations'),
    ('provider.test', 'Test provider health'),
    ('provider.maintenance', 'Set provider maintenance mode'),
    ('credentials.create', 'Create provider secrets'),
    ('credentials.rotate', 'Rotate provider secrets'),
    ('credentials.disable', 'Disable provider secrets'),
    ('webhook_config.view', 'View webhook endpoint config'),
    ('webhook_config.manage', 'Manage webhook endpoint config'),
    ('webhook_events.view', 'View webhook events'),
    ('webhook_events.retry', 'Retry webhook events'),
    ('bank_integrations.view', 'View bank integrations'),
    ('bank_integrations.manage', 'Manage bank integrations'),
    ('refunds.view', 'View refunds'),
    ('refunds.request', 'Request refunds'),
    ('refunds.approve', 'Approve refunds'),
    ('refunds.initiate', 'Initiate refunds'),
    ('settlements.export', 'Export settlements'),
    ('risk_rules.view', 'View risk rules'),
    ('risk_rules.manage', 'Manage risk rules'),
    ('audit_logs.view', 'View audit logs'),
    ('white_label_config.view', 'View white-label payment config')
ON CONFLICT (code) DO NOTHING;
