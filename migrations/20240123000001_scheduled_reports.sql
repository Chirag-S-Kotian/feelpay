-- Scheduled Reports Table
-- Tracks recurring report schedules for merchants with email delivery

CREATE TABLE IF NOT EXISTS scheduled_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    report_type VARCHAR(50) NOT NULL,
    frequency VARCHAR(20) NOT NULL,
    day_of_week INT,
    day_of_month INT,
    time_of_day TIME NOT NULL,
    recipient_emails TEXT[] NOT NULL,
    format VARCHAR(10) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_user_id UUID REFERENCES users(id),
    FOREIGN KEY (merchant_id) REFERENCES merchants(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_merchant_id ON scheduled_reports(merchant_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_next_run_at ON scheduled_reports(next_run_at);
