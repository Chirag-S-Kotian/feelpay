-- Finance-grade company admin reports and final hardening indexes.

CREATE TABLE IF NOT EXISTS admin_scheduled_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_type VARCHAR(80) NOT NULL,
    filters_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    frequency VARCHAR(20) NOT NULL CHECK (frequency IN ('daily', 'weekly', 'monthly')),
    recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    format VARCHAR(12) NOT NULL CHECK (format IN ('CSV', 'PDF', 'XLSX')),
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_by_admin_id UUID REFERENCES admin_users(id),
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_report_exports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_type VARCHAR(80) NOT NULL,
    file_name TEXT NOT NULL,
    format VARCHAR(12) NOT NULL CHECK (format IN ('CSV', 'PDF', 'XLSX')),
    status VARCHAR(30) NOT NULL DEFAULT 'queued',
    row_count BIGINT NOT NULL DEFAULT 0,
    filters_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    generated_by_admin_id UUID REFERENCES admin_users(id),
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days'
);

CREATE INDEX IF NOT EXISTS idx_admin_scheduled_reports_next_run
    ON admin_scheduled_reports(status, next_run_at);
CREATE INDEX IF NOT EXISTS idx_admin_report_exports_type_generated
    ON admin_report_exports(report_type, generated_at DESC);

ALTER TABLE payment_orders
    ADD COLUMN IF NOT EXISTS provider_id UUID REFERENCES payment_providers(id);

CREATE INDEX IF NOT EXISTS idx_payment_orders_created_status_provider
    ON payment_orders(created_at DESC, status, provider_id);
CREATE INDEX IF NOT EXISTS idx_payment_refunds_created_status_provider
    ON payment_refunds(created_at DESC, status, provider_id);
CREATE INDEX IF NOT EXISTS idx_settlement_batches_company_status_expected_provider
    ON settlement_batches_company(status, expected_settlement_date, provider_id);
CREATE INDEX IF NOT EXISTS idx_payment_webhook_events_provider_event_id
    ON payment_webhook_events(provider_event_id);
CREATE INDEX IF NOT EXISTS idx_payment_webhook_events_processing_status
    ON payment_webhook_events(processing_status);
CREATE INDEX IF NOT EXISTS idx_audit_logs_company_entity
    ON audit_logs_company(created_at DESC, entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_transaction_group
    ON ledger_entries(transaction_group_id);
CREATE INDEX IF NOT EXISTS idx_reconciliation_records_match_status_simple
    ON reconciliation_records(match_status);
CREATE INDEX IF NOT EXISTS idx_admin_notifications_status_severity
    ON admin_notifications(status, severity);
CREATE INDEX IF NOT EXISTS idx_api_request_logs_request_id_hardening
    ON api_request_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_api_request_logs_idempotency_hardening
    ON api_request_logs(idempotency_key);
