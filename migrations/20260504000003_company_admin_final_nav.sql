-- Final company-admin navigation support.
-- Internal company admins only; no external self-service roles.

ALTER TABLE admin_roles
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE admin_permissions
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE admin_users
    ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255) NOT NULL DEFAULT 'invite_pending',
    ADD COLUMN IF NOT EXISTS role VARCHAR(50),
    ADD COLUMN IF NOT EXISTS role_id UUID REFERENCES admin_roles(id),
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;

UPDATE admin_users
SET status = CASE WHEN COALESCE(is_active, TRUE) THEN 'active' ELSE 'inactive' END
WHERE status IS NULL;

CREATE TABLE IF NOT EXISTS admin_user_roles (
    admin_user_id UUID NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES admin_roles(id) ON DELETE CASCADE,
    assigned_by_admin_id UUID,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (admin_user_id, role_id)
);

CREATE TABLE IF NOT EXISTS system_health_checks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    component VARCHAR(120) NOT NULL UNIQUE,
    status VARCHAR(30) NOT NULL DEFAULT 'unknown',
    latency_ms INTEGER,
    error_message TEXT,
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS background_job_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_name VARCHAR(140) NOT NULL UNIQUE,
    status VARCHAR(30) NOT NULL DEFAULT 'idle',
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    failure_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    type VARCHAR(80) NOT NULL,
    title VARCHAR(180) NOT NULL,
    message TEXT,
    related_entity_type VARCHAR(80),
    related_entity_id UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    assigned_to_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS platform_settings (
    key VARCHAR(120) PRIMARY KEY,
    value TEXT NOT NULL,
    value_type VARCHAR(30) NOT NULL DEFAULT 'string',
    updated_by_admin_id UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO admin_permissions (code, description) VALUES
    ('admin_users.view', 'View company admin users'),
    ('admin_users.manage', 'Manage company admin users'),
    ('roles.view', 'View company admin roles'),
    ('roles.manage', 'Manage company admin roles'),
    ('system_health.view', 'View system health'),
    ('system_health.manage', 'Manage system health incidents and jobs'),
    ('notifications.view', 'View company admin notifications'),
    ('notifications.manage', 'Manage company admin notifications'),
    ('platform_settings.view', 'View platform settings'),
    ('platform_settings.manage', 'Manage platform settings')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_roles (name, description) VALUES
    ('Auditor', 'Read-only audit and operations access')
ON CONFLICT (name) DO NOTHING;

INSERT INTO system_health_checks (component, status)
VALUES
    ('API status', 'healthy'),
    ('Database status', 'healthy'),
    ('Queue status', 'healthy'),
    ('Webhook worker status', 'healthy'),
    ('Settlement sync status', 'healthy'),
    ('Reconciliation job status', 'healthy'),
    ('Provider health', 'unknown'),
    ('Failed background jobs', 'healthy')
ON CONFLICT (component) DO NOTHING;

INSERT INTO background_job_health (job_name, status)
VALUES
    ('webhook_retry_worker', 'idle'),
    ('settlement_sync', 'idle'),
    ('reconciliation_auto_match', 'idle'),
    ('credential_rotation_reminder', 'idle')
ON CONFLICT (job_name) DO NOTHING;

INSERT INTO platform_settings (key, value, value_type) VALUES
    ('platform_name', 'FeelPay', 'string'),
    ('support_email', 'support@feelpay.local', 'string'),
    ('support_phone', '', 'string'),
    ('default_country', 'IN', 'string'),
    ('default_currency', 'INR', 'string'),
    ('maintenance_mode', 'false', 'boolean'),
    ('session_timeout', '86400', 'number'),
    ('mfa_required', 'true', 'boolean'),
    ('audit_retention_days', '365', 'number'),
    ('export_retention_days', '30', 'number'),
    ('notification_email_sender', 'alerts@feelpay.local', 'string'),
    ('timezone', 'Asia/Kolkata', 'string'),
    ('date_format', 'DD MMM YYYY', 'string')
ON CONFLICT (key) DO NOTHING;
