-- Add environment column to settlements table
ALTER TABLE settlements ADD COLUMN IF NOT EXISTS environment VARCHAR(20);

-- Update existing settlements based on the environment of their associated transactions
UPDATE settlements s
SET environment = COALESCE(
    (SELECT po.metadata->>'environment'
     FROM settlement_items si
     JOIN payment_orders po ON po.id = si.transaction_id
     WHERE si.settlement_id = s.id
     LIMIT 1),
    'test'
);

-- Make column non-nullable and set default
ALTER TABLE settlements ALTER COLUMN environment SET NOT NULL;
ALTER TABLE settlements ALTER COLUMN environment SET DEFAULT 'test';

-- Index for querying settlements by environment
CREATE INDEX IF NOT EXISTS idx_settlements_environment ON settlements(merchant_id, environment);
