-- Add retry settings to subscriptions and plans
ALTER TABLE subscription_plans ADD COLUMN IF NOT EXISTS retry_settings JSONB;
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS retry_settings JSONB;

-- Add failed_attempts to invoices to track retries
ALTER TABLE subscription_invoices ADD COLUMN IF NOT EXISTS failed_attempts INTEGER DEFAULT 0;
ALTER TABLE subscription_invoices ADD COLUMN IF NOT EXISTS next_retry_at TIMESTAMPTZ;
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS failed_attempts INTEGER DEFAULT 0;
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS next_retry_at TIMESTAMPTZ;