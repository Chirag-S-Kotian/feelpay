-- Create scheduled_reports table for recurring report scheduling
-- Requirement 7.1, 7.3: Scheduled Email Delivery of Reports

CREATE TABLE IF NOT EXISTS scheduled_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    report_type VARCHAR(50) NOT NULL, -- 'transaction_summary', 'settlement_reconciliation', 'refund_report', 'dispute_report', 'gst_report', 'custom'
    frequency VARCHAR(20) NOT NULL, -- 'daily', 'weekly', 'monthly'
    day_of_week INT, -- 0-6 (Monday-Sunday), NULL if daily/monthly
    day_of_month INT, -- 1-31, NULL if daily/weekly
    time_of_day TIME NOT NULL, -- HH:MM in merchant's timezone
    recipient_emails TEXT[] NOT NULL, -- array of email addresses
    format VARCHAR(10) NOT NULL, -- 'csv', 'pdf', 'excel'
    is_active BOOLEAN DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_user_id UUID REFERENCES users(id),
    CONSTRAINT valid_frequency CHECK (frequency IN ('daily', 'weekly', 'monthly')),
    CONSTRAINT valid_report_type CHECK (report_type IN ('transaction_summary', 'settlement_reconciliation', 'refund_report', 'dispute_report', 'gst_report', 'custom')),
    CONSTRAINT valid_format CHECK (format IN ('csv', 'pdf', 'excel')),
    CONSTRAINT valid_day_of_week CHECK (day_of_week IS NULL OR (day_of_week >= 0 AND day_of_week <= 6)),
    CONSTRAINT valid_day_of_month CHECK (day_of_month IS NULL OR (day_of_month >= 1 AND day_of_month <= 31))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_merchant_id ON scheduled_reports(merchant_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_next_run_at ON scheduled_reports(next_run_at);
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_is_active ON scheduled_reports(is_active);
