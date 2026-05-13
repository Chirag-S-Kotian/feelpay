-- Merchant audit log scoping for dashboard views.
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS target TEXT;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS user_agent TEXT;

UPDATE audit_logs
SET merchant_id = entity_id
WHERE merchant_id IS NULL
  AND entity_type IN ('merchant', 'merchant_checkout_settings');

CREATE INDEX IF NOT EXISTS idx_audit_logs_merchant_created
    ON audit_logs(merchant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_created
    ON audit_logs(tenant_id, created_at DESC)
    WHERE tenant_id IS NOT NULL;
