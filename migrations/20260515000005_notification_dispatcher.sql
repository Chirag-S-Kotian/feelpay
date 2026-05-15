CREATE TABLE IF NOT EXISTS notification_dispatch_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID,
    event_type VARCHAR(120) NOT NULL,
    channel VARCHAR(20) NOT NULL CHECK (channel IN ('email', 'sms')),
    template_id UUID REFERENCES notification_templates(id) ON DELETE SET NULL,
    provider_id UUID REFERENCES integration_providers(id) ON DELETE SET NULL,
    recipient_masked VARCHAR(255) NOT NULL,
    status VARCHAR(30) NOT NULL CHECK (status IN ('sent', 'failed', 'skipped')),
    error_code VARCHAR(120),
    error_message TEXT,
    provider_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    payload_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    retry_payload JSONB,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retryable BOOLEAN NOT NULL DEFAULT true,
    next_retry_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notification_dispatch_logs_created
    ON notification_dispatch_logs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_dispatch_logs_retry
    ON notification_dispatch_logs(status, next_retry_at)
    WHERE status = 'failed' AND retryable = true;

CREATE OR REPLACE FUNCTION update_notification_dispatch_logs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_notification_dispatch_logs_updated_at
    ON notification_dispatch_logs;

CREATE TRIGGER trigger_notification_dispatch_logs_updated_at
    BEFORE UPDATE ON notification_dispatch_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_dispatch_logs_updated_at();
