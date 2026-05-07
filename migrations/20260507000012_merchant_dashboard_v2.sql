-- Missing columns from failed migrations
ALTER TABLE api_request_logs ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);
CREATE INDEX IF NOT EXISTS idx_api_request_logs_merchant ON api_request_logs(merchant_id);

ALTER TABLE transaction_events ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE transaction_events ADD COLUMN IF NOT EXISTS ip_address VARCHAR(45);
ALTER TABLE transaction_events ADD COLUMN IF NOT EXISTS user_agent TEXT;

ALTER TABLE merchants ADD COLUMN IF NOT EXISTS enforce_idempotency BOOLEAN DEFAULT FALSE;

ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS verification_details JSONB DEFAULT '{}'::jsonb;
ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS last_payout_at TIMESTAMPTZ;

ALTER TABLE payment_webhook_events ADD COLUMN IF NOT EXISTS request_headers JSONB;
ALTER TABLE payment_webhook_events ADD COLUMN IF NOT EXISTS response_headers JSONB;
ALTER TABLE payment_webhook_events ADD COLUMN IF NOT EXISTS delivery_latency_ms INTEGER;

ALTER TABLE developer_api_keys ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);
CREATE INDEX IF NOT EXISTS idx_developer_api_keys_merchant ON developer_api_keys(merchant_id);
