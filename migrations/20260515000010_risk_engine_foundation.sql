-- Safe internal risk and fraud rules engine foundation.

ALTER TABLE risk_rules
    ADD COLUMN IF NOT EXISTS description TEXT,
    ADD COLUMN IF NOT EXISTS scope VARCHAR(24) NOT NULL DEFAULT 'payment',
    ADD COLUMN IF NOT EXISTS environment VARCHAR(16) NOT NULL DEFAULT 'test',
    ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS severity VARCHAR(16) NOT NULL DEFAULT 'medium',
    ADD COLUMN IF NOT EXISTS condition_config JSONB NOT NULL DEFAULT '{}'::jsonb;

UPDATE risk_rules
SET enabled = status <> 'disabled',
    condition_config = COALESCE(condition_config, condition, '{}'::jsonb);

ALTER TABLE risk_rules
    DROP CONSTRAINT IF EXISTS risk_rules_scope_check,
    ADD CONSTRAINT risk_rules_scope_check
    CHECK (scope IN ('merchant','payment','refund','dispute','payout','settlement'));

ALTER TABLE risk_rules
    DROP CONSTRAINT IF EXISTS risk_rules_environment_check,
    ADD CONSTRAINT risk_rules_environment_check
    CHECK (environment IN ('test','live'));

ALTER TABLE risk_rules
    DROP CONSTRAINT IF EXISTS risk_rules_severity_check,
    ADD CONSTRAINT risk_rules_severity_check
    CHECK (severity IN ('low','medium','high','critical'));

ALTER TABLE risk_rules
    DROP CONSTRAINT IF EXISTS risk_rules_action_check,
    ADD CONSTRAINT risk_rules_action_check
    CHECK (action IN ('allow','review','block','hold_payout','restrict_merchant'));

CREATE TABLE IF NOT EXISTS risk_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID REFERENCES risk_rules(id) ON DELETE SET NULL,
    merchant_id UUID REFERENCES merchants(id),
    scope VARCHAR(24) NOT NULL,
    environment VARCHAR(16) NOT NULL,
    entity_type VARCHAR(32) NOT NULL,
    entity_id UUID,
    matched BOOLEAN NOT NULL,
    action VARCHAR(24) NOT NULL,
    severity VARCHAR(16) NOT NULL,
    reason TEXT NOT NULL,
    input_summary_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID REFERENCES merchants(id),
    primary_evaluation_id UUID REFERENCES risk_evaluations(id) ON DELETE SET NULL,
    scope VARCHAR(24) NOT NULL,
    entity_type VARCHAR(32) NOT NULL,
    entity_id UUID,
    status VARCHAR(24) NOT NULL DEFAULT 'open',
    severity VARCHAR(16) NOT NULL,
    action VARCHAR(24) NOT NULL,
    reason TEXT NOT NULL,
    assigned_to_admin_id UUID REFERENCES admin_users(id),
    reviewed_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_case_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES risk_cases(id) ON DELETE CASCADE,
    note TEXT NOT NULL,
    created_by_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS risk_action_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID REFERENCES risk_cases(id) ON DELETE CASCADE,
    evaluation_id UUID REFERENCES risk_evaluations(id) ON DELETE SET NULL,
    action VARCHAR(24) NOT NULL,
    from_status VARCHAR(24),
    to_status VARCHAR(24),
    note TEXT,
    actor_admin_id UUID REFERENCES admin_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE risk_signals
    ADD COLUMN IF NOT EXISTS evaluation_id UUID REFERENCES risk_evaluations(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS rule_id UUID REFERENCES risk_rules(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS scope VARCHAR(24),
    ADD COLUMN IF NOT EXISTS entity_type VARCHAR(32),
    ADD COLUMN IF NOT EXISTS entity_id UUID,
    ADD COLUMN IF NOT EXISTS signal_payload_redacted JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_risk_rules_scope_env_enabled
    ON risk_rules(scope, environment, enabled, priority);
CREATE INDEX IF NOT EXISTS idx_risk_evaluations_entity
    ON risk_evaluations(scope, entity_type, entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_cases_status
    ON risk_cases(status, severity, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_case_notes_case
    ON risk_case_notes(case_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_action_history_case
    ON risk_action_history(case_id, created_at DESC);
