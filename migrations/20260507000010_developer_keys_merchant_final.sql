-- Add merchant_id to developer_api_keys for multi-tenant isolation
ALTER TABLE developer_api_keys ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);
CREATE INDEX IF NOT EXISTS idx_developer_api_keys_merchant ON developer_api_keys(merchant_id);
