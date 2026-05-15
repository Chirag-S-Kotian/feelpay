CREATE TABLE IF NOT EXISTS reconciliation_batches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_type VARCHAR(40) NOT NULL CHECK (source_type IN (
        'internal_ledger', 'settlement_report', 'payout_report',
        'bank_statement', 'provider_report', 'manual_csv'
    )),
    environment VARCHAR(20) NOT NULL CHECK (environment IN ('test', 'live')),
    status VARCHAR(30) NOT NULL DEFAULT 'uploaded' CHECK (status IN (
        'uploaded', 'parsing', 'parsed', 'matching', 'matched',
        'mismatch_found', 'failed', 'closed'
    )),
    uploaded_by_admin_id UUID REFERENCES admin_users(id),
    file_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    date_from DATE,
    date_to DATE,
    total_items INTEGER NOT NULL DEFAULT 0,
    matched_items INTEGER NOT NULL DEFAULT 0,
    mismatched_items INTEGER NOT NULL DEFAULT 0,
    total_amount_paise BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS reconciliation_source_imports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES reconciliation_batches(id) ON DELETE CASCADE,
    file_name TEXT,
    file_hash TEXT,
    content_type VARCHAR(120),
    row_count INTEGER NOT NULL DEFAULT 0,
    parser VARCHAR(80) NOT NULL DEFAULT 'safe_csv_v1',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reconciliation_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES reconciliation_batches(id) ON DELETE CASCADE,
    source_import_id UUID REFERENCES reconciliation_source_imports(id) ON DELETE SET NULL,
    transaction_id UUID,
    refund_id UUID,
    settlement_id UUID,
    payout_id UUID,
    amount_paise BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    utr_reference VARCHAR(255),
    provider_reference VARCHAR(255),
    order_reference VARCHAR(255),
    external_status VARCHAR(80),
    matched BOOLEAN NOT NULL DEFAULT false,
    mismatch_reason VARCHAR(80),
    raw_summary_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    reviewed_by_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reconciliation_mismatches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES reconciliation_batches(id) ON DELETE CASCADE,
    item_id UUID REFERENCES reconciliation_items(id) ON DELETE CASCADE,
    mismatch_type VARCHAR(80) NOT NULL,
    detail_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    status VARCHAR(30) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'reviewed', 'resolved', 'ignored')),
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS reconciliation_adjustments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES reconciliation_batches(id) ON DELETE CASCADE,
    item_id UUID REFERENCES reconciliation_items(id) ON DELETE SET NULL,
    amount_paise BIGINT NOT NULL,
    reason TEXT NOT NULL,
    created_by_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reconciliation_batches_created
    ON reconciliation_batches(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_reconciliation_items_batch_matched
    ON reconciliation_items(batch_id, matched, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_reconciliation_mismatches_batch_status
    ON reconciliation_mismatches(batch_id, status, created_at DESC);
