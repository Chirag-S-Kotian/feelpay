-- Store generated company-admin report files and terminal failure reasons.

ALTER TABLE admin_report_exports
    ADD COLUMN IF NOT EXISTS file_reference TEXT,
    ADD COLUMN IF NOT EXISTS error_message TEXT;

CREATE INDEX IF NOT EXISTS idx_admin_report_exports_status_generated
    ON admin_report_exports(status, generated_at DESC);
