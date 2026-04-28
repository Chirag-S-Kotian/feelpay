-- Phase 11: Email & SMS Logging

CREATE TABLE IF NOT EXISTS email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    recipient VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    template_type VARCHAR(50) NOT NULL,    -- payment_receipt, settlement_alert, kyc_status, dispute_alert
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, sent, failed, bounced
    error_message TEXT,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_logs_merchant ON email_logs(merchant_id);
CREATE INDEX idx_email_logs_status ON email_logs(status);
CREATE INDEX idx_email_logs_template ON email_logs(template_type);
CREATE INDEX idx_email_logs_created ON email_logs(created_at);

CREATE TABLE IF NOT EXISTS sms_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    recipient VARCHAR(20) NOT NULL,       -- E.164 format: +919876543210
    message TEXT NOT NULL,
    template_type VARCHAR(50) NOT NULL,   -- otp, payment_confirmation, settlement_alert
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, sent, failed, bounced
    error_message TEXT,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sms_logs_merchant ON sms_logs(merchant_id);
CREATE INDEX idx_sms_logs_status ON sms_logs(status);
CREATE INDEX idx_sms_logs_template ON sms_logs(template_type);
CREATE INDEX idx_sms_logs_created ON sms_logs(created_at);

-- Customer receipt email preference
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS send_customer_receipt_email BOOLEAN DEFAULT TRUE;
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS send_customer_receipt_sms BOOLEAN DEFAULT FALSE;

-- Email/SMS provider configuration (stored encrypted)
CREATE TABLE IF NOT EXISTS notification_provider_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_type VARCHAR(50) NOT NULL,   -- email_sendgrid, email_ses, sms_twilio, sms_exotel
    config JSONB NOT NULL,                -- encrypted provider credentials
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_type)
);

CREATE INDEX idx_notification_provider_config_active ON notification_provider_config(is_active);
