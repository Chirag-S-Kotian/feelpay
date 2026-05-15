-- Harden internal settlement and payout lifecycle without implying live rail execution.

ALTER TABLE settlement_batches_company
    ADD COLUMN IF NOT EXISTS failure_reason TEXT,
    ADD COLUMN IF NOT EXISTS reference_kind VARCHAR(16),
    ADD COLUMN IF NOT EXISTS reconciliation_batch_id UUID REFERENCES reconciliation_batches(id),
    ADD COLUMN IF NOT EXISTS reconciliation_item_id UUID REFERENCES reconciliation_items(id);

ALTER TABLE payouts_company
    ADD COLUMN IF NOT EXISTS reference_kind VARCHAR(16),
    ADD COLUMN IF NOT EXISTS manual_reference TEXT,
    ADD COLUMN IF NOT EXISTS approved_by_admin_id UUID,
    ADD COLUMN IF NOT EXISTS approved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS reconciliation_batch_id UUID REFERENCES reconciliation_batches(id),
    ADD COLUMN IF NOT EXISTS reconciliation_item_id UUID REFERENCES reconciliation_items(id);

ALTER TABLE payout_batches_company
    ADD COLUMN IF NOT EXISTS failure_reason TEXT,
    ADD COLUMN IF NOT EXISTS reconciliation_batch_id UUID REFERENCES reconciliation_batches(id);

ALTER TABLE payout_batch_items_company
    ADD COLUMN IF NOT EXISTS failure_reason TEXT,
    ADD COLUMN IF NOT EXISTS reconciliation_item_id UUID REFERENCES reconciliation_items(id);

CREATE TABLE IF NOT EXISTS payout_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payout_id UUID NOT NULL REFERENCES payouts_company(id) ON DELETE CASCADE,
    from_status VARCHAR(32),
    to_status VARCHAR(32) NOT NULL,
    reason TEXT,
    reference_kind VARCHAR(16),
    reference_value TEXT,
    actor_admin_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payout_status_history_payout
    ON payout_status_history(payout_id, created_at DESC);

UPDATE settlement_batches_company
SET status = CASE status
    WHEN 'pending' THEN 'scheduled'
    WHEN 'settled' THEN 'paid'
    WHEN 'reconciled' THEN 'paid'
    ELSE status
END
WHERE status IN ('pending', 'settled', 'reconciled');

UPDATE payouts_company
SET status = CASE status
    WHEN 'scheduled' THEN 'queued'
    WHEN 'retried' THEN 'queued'
    ELSE status
END
WHERE status IN ('scheduled', 'retried');

ALTER TABLE settlement_batches_company
    DROP CONSTRAINT IF EXISTS settlement_batches_company_status_check,
    ADD CONSTRAINT settlement_batches_company_status_check
    CHECK (status IN ('scheduled', 'processing', 'paid', 'failed', 'reversed', 'on_hold'));

ALTER TABLE payouts_company
    DROP CONSTRAINT IF EXISTS payouts_company_status_check,
    ADD CONSTRAINT payouts_company_status_check
    CHECK (status IN ('draft', 'queued', 'approved', 'processing', 'paid', 'failed', 'reversed', 'cancelled', 'on_hold'));

ALTER TABLE payout_batches_company
    DROP CONSTRAINT IF EXISTS payout_batches_company_status_check,
    ADD CONSTRAINT payout_batches_company_status_check
    CHECK (status IN ('draft', 'queued', 'approved', 'processing', 'paid', 'failed', 'reversed', 'cancelled', 'on_hold', 'partial_failed'));

ALTER TABLE payout_batch_items_company
    DROP CONSTRAINT IF EXISTS payout_batch_items_company_status_check,
    ADD CONSTRAINT payout_batch_items_company_status_check
    CHECK (status IN ('draft', 'queued', 'approved', 'processing', 'paid', 'failed', 'reversed', 'cancelled', 'on_hold'));

ALTER TABLE settlement_batches_company
    DROP CONSTRAINT IF EXISTS settlement_batches_company_reference_kind_check,
    ADD CONSTRAINT settlement_batches_company_reference_kind_check
    CHECK (reference_kind IS NULL OR reference_kind IN ('manual', 'test', 'provider'));

ALTER TABLE payouts_company
    DROP CONSTRAINT IF EXISTS payouts_company_reference_kind_check,
    ADD CONSTRAINT payouts_company_reference_kind_check
    CHECK (reference_kind IS NULL OR reference_kind IN ('manual', 'test', 'provider'));
