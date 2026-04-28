-- Export Jobs Table
-- Tracks async export jobs for transactions, settlements, refunds, disputes

CREATE TABLE IF NOT EXISTS export_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    job_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'queued',
    filters JSONB,
    file_url TEXT,
    file_size_bytes BIGINT,
    record_count INT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    idempotency_key VARCHAR(255) UNIQUE,
    created_by_user_id UUID
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_export_jobs_merchant_id ON export_jobs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_export_jobs_status ON export_jobs(status);
CREATE INDEX IF NOT EXISTS idx_export_jobs_created_at ON export_jobs(created_at);
