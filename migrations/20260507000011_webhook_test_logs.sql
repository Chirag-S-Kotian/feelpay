-- Add migration for webhook test logs
CREATE TABLE IF NOT EXISTS webhook_test_logs (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    endpoint_id UUID,
    event_type TEXT NOT NULL,
    webhook_url TEXT NOT NULL,
    status TEXT NOT NULL,
    http_status INTEGER,
    request_headers JSONB NOT NULL DEFAULT '{}',
    request_payload JSONB NOT NULL DEFAULT '{}',
    response_body TEXT,
    error_message TEXT,
    signature_header TEXT,
    response_time_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_test_logs_merchant_id ON webhook_test_logs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_webhook_test_logs_created_at ON webhook_test_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_webhook_test_logs_status ON webhook_test_logs(status);
CREATE INDEX IF NOT EXISTS idx_webhook_test_logs_endpoint_id ON webhook_test_logs(endpoint_id) WHERE endpoint_id IS NOT NULL;
