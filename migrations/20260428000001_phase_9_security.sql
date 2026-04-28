-- Phase 9: Security Enhancements

-- Blacklist tables for fraud prevention
CREATE TABLE IF NOT EXISTS blacklist_bins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bin VARCHAR(6) NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    added_by UUID NOT NULL REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    CONSTRAINT bin_length CHECK (LENGTH(bin) = 6)
);

CREATE INDEX idx_blacklist_bins_expires ON blacklist_bins(expires_at);

CREATE TABLE IF NOT EXISTS blacklist_ips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address VARCHAR(45) NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    added_by UUID NOT NULL REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_blacklist_ips_expires ON blacklist_ips(expires_at);

CREATE TABLE IF NOT EXISTS blacklist_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain VARCHAR(255) NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    added_by UUID NOT NULL REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_blacklist_domains_expires ON blacklist_domains(expires_at);

-- 3DS2 Challenge tracking
CREATE TABLE IF NOT EXISTS three_ds_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payment_orders(id),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, authenticated, failed
    acs_url TEXT,
    creq TEXT,
    dsTransID VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_three_ds_challenges_payment ON three_ds_challenges(payment_id);
CREATE INDEX idx_three_ds_challenges_merchant ON three_ds_challenges(merchant_id);
CREATE INDEX idx_three_ds_challenges_status ON three_ds_challenges(status);

-- Device fingerprinting
CREATE TABLE IF NOT EXISTS device_fingerprints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    customer_id UUID REFERENCES customers(id),
    device_id VARCHAR(64) NOT NULL UNIQUE,
    user_agent TEXT NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    accept_language VARCHAR(255),
    accept_encoding VARCHAR(255),
    timezone VARCHAR(50),
    screen_resolution VARCHAR(50),
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    transaction_count INT NOT NULL DEFAULT 1,
    is_trusted BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_device_fingerprints_merchant ON device_fingerprints(merchant_id);
CREATE INDEX idx_device_fingerprints_customer ON device_fingerprints(customer_id);
CREATE INDEX idx_device_fingerprints_device_id ON device_fingerprints(device_id);
CREATE INDEX idx_device_fingerprints_is_trusted ON device_fingerprints(is_trusted);

-- Velocity check tracking (stored in Redis, but audit trail in DB)
CREATE TABLE IF NOT EXISTS velocity_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    customer_id UUID REFERENCES customers(id),
    violation_type VARCHAR(50) NOT NULL, -- same_card, same_ip, same_device
    card_last4 VARCHAR(4),
    ip_address VARCHAR(45),
    device_id VARCHAR(64),
    count INT NOT NULL,
    limit_exceeded INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_velocity_audit_merchant ON velocity_audit(merchant_id);
CREATE INDEX idx_velocity_audit_customer ON velocity_audit(customer_id);
CREATE INDEX idx_velocity_audit_created ON velocity_audit(created_at);

-- Replay prevention nonce storage (primarily in Redis, but audit in DB)
CREATE TABLE IF NOT EXISTS replay_nonce_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nonce VARCHAR(255) NOT NULL UNIQUE,
    merchant_id UUID REFERENCES merchants(id),
    request_timestamp BIGINT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_replay_nonce_audit_expires ON replay_nonce_audit(expires_at);
CREATE INDEX idx_replay_nonce_audit_detected ON replay_nonce_audit(detected_at);
