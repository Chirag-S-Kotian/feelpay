-- Create export_jobs table for async export job tracking
-- Requirement 1.1: Transaction Export (CSV/Excel)

CREATE TABLE IF NOT EXISTS export_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL, -- 'transaction', 'settlement', 'refund', 'dispute', 'bulk'
    status VARCHAR(20) NOT NULL DEFAULT 'queued', -- 'queued', 'processing', 'completed', 'failed'
    filters JSONB, -- date_range, status, payment_method, amount_range, etc.
    file_url TEXT, -- signed S3 URL
    file_size_bytes BIGINT,
    record_count INT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ, -- 7 days after completion
    idempotency_key VARCHAR(255) UNIQUE, -- for deduplication
    created_by_user_id UUID REFERENCES users(id),
    CONSTRAINT valid_status CHECK (status IN ('queued', 'processing', 'completed', 'failed')),
    CONSTRAINT valid_job_type CHECK (job_type IN ('transaction', 'settlement', 'refund', 'dispute', 'bulk'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_export_jobs_merchant_id ON export_jobs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_export_jobs_status ON export_jobs(status);
CREATE INDEX IF NOT EXISTS idx_export_jobs_created_at ON export_jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_export_jobs_idempotency_key ON export_jobs(idempotency_key);
