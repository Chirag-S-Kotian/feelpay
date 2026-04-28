-- Phase 10: Settlements & Finance
-- Phase 11: Notifications & Communications
-- Phase 12: Analytics & Reports

-- Phase 10: Instant Settlement
CREATE TABLE IF NOT EXISTS instant_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    amount BIGINT NOT NULL,             -- In paise
    fee BIGINT NOT NULL,                -- Platform fee
    tds_amount BIGINT NOT NULL,         -- TDS deducted
    net_amount BIGINT NOT NULL,         -- After fee and TDS
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    utr VARCHAR(255),
    CONSTRAINT amount_positive CHECK (amount > 0)
);

CREATE INDEX idx_instant_settlements_merchant ON instant_settlements(merchant_id);
CREATE INDEX idx_instant_settlements_status ON instant_settlements(status);
CREATE INDEX idx_instant_settlements_requested ON instant_settlements(requested_at);

-- Phase 10: TDS Calculations
CREATE TABLE IF NOT EXISTS tds_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID NOT NULL REFERENCES settlements(id),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    gross_amount BIGINT NOT NULL,
    tds_rate NUMERIC(5, 2) NOT NULL,    -- e.g., 1.00 for 1%
    tds_amount BIGINT NOT NULL,
    net_amount BIGINT NOT NULL,
    financial_year VARCHAR(20) NOT NULL, -- FY 2025-26
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tds_calculations_merchant ON tds_calculations(merchant_id);
CREATE INDEX idx_tds_calculations_financial_year ON tds_calculations(financial_year);

-- Phase 10: Reconciliation Records
CREATE TABLE IF NOT EXISTS reconciliation_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    settlement_id UUID NOT NULL REFERENCES settlements(id),
    feelpay_amount BIGINT NOT NULL,     -- Amount in FeelPay records
    provider_amount BIGINT NOT NULL,    -- Amount from provider report
    bank_amount BIGINT NOT NULL,        -- Amount from bank statement
    discrepancy BIGINT NOT NULL,        -- Difference
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, matched, discrepancy
    reconciled_at TIMESTAMPTZ
);

CREATE INDEX idx_reconciliation_records_merchant ON reconciliation_records(merchant_id);
CREATE INDEX idx_reconciliation_records_status ON reconciliation_records(status);

-- Phase 10: Payout Retry History
ALTER TABLE settlements ADD COLUMN IF NOT EXISTS retry_count INT DEFAULT 0;
ALTER TABLE settlements ADD COLUMN IF NOT EXISTS last_retry_at TIMESTAMPTZ;

-- Phase 11: Notification Templates
CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,    -- payment_receipt, payment_failed, etc.
    channel VARCHAR(20) NOT NULL,       -- email, sms, whatsapp
    subject VARCHAR(255),               -- For email
    body TEXT NOT NULL,                 -- Template with {{variables}}
    variables TEXT[] NOT NULL,          -- Array of required variables
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_type, channel)
);

CREATE INDEX idx_notification_templates_event ON notification_templates(event_type);
CREATE INDEX idx_notification_templates_active ON notification_templates(is_active);

-- Phase 11: Rendered Notifications (audit trail)
CREATE TABLE IF NOT EXISTS rendered_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES notification_templates(id),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    recipient VARCHAR(255) NOT NULL,   -- Email, phone, or WhatsApp ID
    channel VARCHAR(20) NOT NULL,
    subject VARCHAR(255),
    body TEXT NOT NULL,
    variables_used JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rendered_notifications_merchant ON rendered_notifications(merchant_id);
CREATE INDEX idx_rendered_notifications_channel ON rendered_notifications(channel);
CREATE INDEX idx_rendered_notifications_created ON rendered_notifications(created_at);

-- Phase 12: Generated Reports
CREATE TABLE IF NOT EXISTS generated_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    report_type VARCHAR(50) NOT NULL,   -- gst, settlement, refunds, transactions, disputes, merchant_fee
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, generating, completed, failed
    file_url TEXT,                      -- S3 URL
    file_format VARCHAR(10) NOT NULL,   -- csv, excel, pdf
    row_count INT,
    generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_generated_reports_merchant ON generated_reports(merchant_id);
CREATE INDEX idx_generated_reports_status ON generated_reports(status);
CREATE INDEX idx_generated_reports_created ON generated_reports(created_at);

-- Phase 12: Scheduled Report Delivery (already exists, ensure columns)
ALTER TABLE scheduled_reports ADD COLUMN IF NOT EXISTS last_sent_at TIMESTAMPTZ;
ALTER TABLE scheduled_reports ADD COLUMN IF NOT EXISTS next_run_at TIMESTAMPTZ;
ALTER TABLE scheduled_reports ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT TRUE;
