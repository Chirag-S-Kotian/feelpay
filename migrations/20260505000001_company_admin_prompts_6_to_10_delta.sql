-- Final UI/data correction, advanced reconciliation deltas, developer tools, compliance tasks, incidents.

ALTER TABLE reconciliation_files
    ADD COLUMN IF NOT EXISTS processed_at TIMESTAMPTZ;

ALTER TABLE reconciliation_records
    ADD COLUMN IF NOT EXISTS payout_id UUID,
    ADD COLUMN IF NOT EXISTS fee_amount BIGINT,
    ADD COLUMN IF NOT EXISTS expected_fee_amount BIGINT,
    ADD COLUMN IF NOT EXISTS tax_amount BIGINT,
    ADD COLUMN IF NOT EXISTS expected_tax_amount BIGINT;

ALTER TABLE api_request_logs
    ADD COLUMN IF NOT EXISTS error_message TEXT;

CREATE TABLE IF NOT EXISTS incidents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(180) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    source VARCHAR(80),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    owner_admin_id UUID REFERENCES admin_users(id),
    summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS incident_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    event_type VARCHAR(80) NOT NULL,
    message TEXT,
    actor_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE admin_notifications
    ADD COLUMN IF NOT EXISTS source VARCHAR(80),
    ADD COLUMN IF NOT EXISTS dedupe_key VARCHAR(180),
    ADD COLUMN IF NOT EXISTS first_seen_at TIMESTAMPTZ DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMPTZ DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS occurrence_count INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS sla_due_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS incident_id UUID REFERENCES incidents(id);

UPDATE admin_notifications
SET source = COALESCE(source, type),
    first_seen_at = COALESCE(first_seen_at, created_at),
    last_seen_at = COALESCE(last_seen_at, updated_at),
    sla_due_at = COALESCE(
        sla_due_at,
        CASE severity
            WHEN 'critical' THEN created_at + INTERVAL '15 minutes'
            WHEN 'warning' THEN created_at + INTERVAL '1 hour'
            ELSE created_at + INTERVAL '1 day'
        END
    );

CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_notifications_dedupe_open
    ON admin_notifications(dedupe_key)
    WHERE dedupe_key IS NOT NULL AND status NOT IN ('resolved', 'ignored');

CREATE TABLE IF NOT EXISTS compliance_tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(180) NOT NULL,
    category VARCHAR(80) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    owner_admin_id UUID REFERENCES admin_users(id),
    due_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO admin_permissions (code, description) VALUES
    ('developer_tools.view', 'View developer tools'),
    ('api_keys.manage', 'Manage company admin API keys'),
    ('api_logs.view', 'View API request and idempotency logs'),
    ('webhook_replay.manage', 'Manage webhook test and replay')
ON CONFLICT (code) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_reconciliation_records_payout ON reconciliation_records(payout_id);
CREATE INDEX IF NOT EXISTS idx_incidents_status_severity ON incidents(status, severity);
CREATE INDEX IF NOT EXISTS idx_incident_events_incident ON incident_events(incident_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_admin_notifications_sla ON admin_notifications(status, severity, sla_due_at);
CREATE INDEX IF NOT EXISTS idx_compliance_tasks_status ON compliance_tasks(status, due_at);
