CREATE TABLE IF NOT EXISTS kyc_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    action VARCHAR(50) NOT NULL,
    actor_id UUID NOT NULL,
    previous_state JSONB,
    new_state JSONB,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_kyc_audit_log_merchant_id ON kyc_audit_log(merchant_id);
CREATE INDEX IF NOT EXISTS idx_kyc_audit_log_action ON kyc_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_kyc_audit_log_created ON kyc_audit_log(created_at DESC);

ALTER TABLE payment_orders
    ADD COLUMN IF NOT EXISTS is_frozen BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS frozen_reason TEXT,
    ADD COLUMN IF NOT EXISTS is_flagged BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS flagged_reason TEXT,
    ADD COLUMN IF NOT EXISTS manually_confirmed BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX IF NOT EXISTS idx_payment_orders_customer_email ON payment_orders(customer_email);
CREATE INDEX IF NOT EXISTS idx_payment_orders_customer_phone ON payment_orders(customer_phone);
CREATE INDEX IF NOT EXISTS idx_payment_orders_admin_flags ON payment_orders(is_frozen, is_flagged);

ALTER TABLE settlements
    ADD COLUMN IF NOT EXISTS hold_reason TEXT,
    ADD COLUMN IF NOT EXISTS processed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS utr_number VARCHAR(50);
UPDATE settlements SET utr_number = utr WHERE utr_number IS NULL AND utr IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_settlements_utr_number_unique
    ON settlements(utr_number) WHERE utr_number IS NOT NULL;

CREATE TABLE IF NOT EXISTS provider_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    encrypted_value TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_validated TIMESTAMPTZ,
    is_valid BOOLEAN DEFAULT TRUE,
    UNIQUE(provider, key_name)
);

CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    tenant_id UUID REFERENCES tenants(id),
    is_active BOOLEAN DEFAULT TRUE,
    totp_secret VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS system_config (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES admin_users(id)
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_admin ON audit_logs(admin_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
