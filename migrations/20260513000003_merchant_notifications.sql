-- Merchant-scoped notification inbox.
ALTER TABLE notification_queue ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE notification_queue ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);

UPDATE notification_queue nq
SET merchant_id = nq.recipient_id
WHERE nq.merchant_id IS NULL
  AND nq.recipient_type = 'merchant'
  AND EXISTS (SELECT 1 FROM merchants m WHERE m.id = nq.recipient_id);

UPDATE notification_queue nq
SET tenant_id = m.tenant_id
FROM merchants m
WHERE nq.tenant_id IS NULL
  AND nq.merchant_id = m.id;

CREATE INDEX IF NOT EXISTS idx_notification_queue_merchant_created
    ON notification_queue(merchant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notification_queue_merchant_unread
    ON notification_queue(merchant_id, is_read)
    WHERE is_read = false;

CREATE INDEX IF NOT EXISTS idx_notification_queue_tenant_created
    ON notification_queue(tenant_id, created_at DESC)
    WHERE tenant_id IS NOT NULL;
