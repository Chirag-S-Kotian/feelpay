-- Preserve request correlation for company-admin audit entries.

ALTER TABLE audit_logs_company
    ADD COLUMN IF NOT EXISTS request_id TEXT;

CREATE INDEX IF NOT EXISTS idx_audit_logs_company_request_id
    ON audit_logs_company(request_id)
    WHERE request_id IS NOT NULL;
