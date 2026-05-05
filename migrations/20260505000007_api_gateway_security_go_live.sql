ALTER TABLE developer_api_keys
    ADD COLUMN IF NOT EXISTS merchant_id UUID REFERENCES merchants(id);

ALTER TABLE api_idempotency_records
    ADD COLUMN IF NOT EXISTS response_body JSONB;

INSERT INTO api_error_catalog (code, message, http_status, retryable, docs_url)
VALUES
    ('invalid_api_key', 'Invalid API key', 401, false, 'https://internal.feelpay/docs/errors#invalid_api_key'),
    ('idempotency_key_required', 'Idempotency-Key header required', 400, false, 'https://internal.feelpay/docs/errors#idempotency_key_required'),
    ('unsupported_api_version', 'Unsupported API version', 400, false, 'https://internal.feelpay/docs/errors#unsupported_api_version')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE IF NOT EXISTS security_review_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(80) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    title VARCHAR(180) NOT NULL,
    description TEXT NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'assigned', 'resolved')),
    owner_admin_id UUID REFERENCES admin_users(id),
    due_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS step_up_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_user_id UUID REFERENCES admin_users(id),
    action VARCHAR(100) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'verified', 'expired', 'cancelled')),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '10 minutes',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS step_up_actions (
    action VARCHAR(100) PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sensitive_action_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(100) NOT NULL,
    actor_admin_id UUID REFERENCES admin_users(id),
    entity_type VARCHAR(80),
    entity_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT,
    step_up_challenge_id UUID REFERENCES step_up_challenges(id),
    status VARCHAR(30) NOT NULL,
    summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS go_live_checklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_key VARCHAR(100) NOT NULL UNIQUE,
    title VARCHAR(180) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'assigned', 'complete', 'blocked')),
    critical BOOLEAN NOT NULL DEFAULT true,
    owner_admin_id UUID REFERENCES admin_users(id),
    evidence_url TEXT,
    completed_by_admin_id UUID REFERENCES admin_users(id),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS production_cutovers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'started', 'paused', 'completed', 'rolled_back', 'failed')),
    started_by_admin_id UUID REFERENCES admin_users(id),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    rollback_plan TEXT NOT NULL,
    notes TEXT,
    super_admin_approved BOOLEAN NOT NULL DEFAULT false,
    compliance_admin_approved BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_review_findings_status
    ON security_review_findings(status, severity, due_at);
CREATE INDEX IF NOT EXISTS idx_step_up_challenges_admin_action
    ON step_up_challenges(admin_user_id, action, status, expires_at);
CREATE INDEX IF NOT EXISTS idx_sensitive_action_history_created
    ON sensitive_action_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_go_live_checklist_status
    ON go_live_checklist(status, critical);
CREATE INDEX IF NOT EXISTS idx_production_cutovers_status
    ON production_cutovers(status, environment);

INSERT INTO step_up_actions (action) VALUES
    ('credential_rotation'),
    ('provider_disable'),
    ('bank_account_change'),
    ('payout_batch_approval'),
    ('role_change'),
    ('compliance_export')
ON CONFLICT (action) DO NOTHING;

INSERT INTO security_review_findings (category, severity, title, description, status)
VALUES
    ('secrets_leakage', 'critical', 'Secret redaction enforced', 'Verify logs, exports, and provider configs never expose credentials.', 'open'),
    ('webhook_spoofing', 'high', 'Webhook signatures required', 'Provider webhook endpoints must reject unsigned payloads.', 'open'),
    ('idempotency_replay_abuse', 'high', 'Idempotency replay controls', 'Write APIs must bind idempotency keys to body hash.', 'open'),
    ('payout_fraud', 'critical', 'Payout eligibility checks', 'Payout approval requires verified bank and eligible business account.', 'open'),
    ('admin_account_takeover', 'critical', 'Admin MFA coverage', 'All company admins must enable MFA before go-live.', 'open')
ON CONFLICT DO NOTHING;

INSERT INTO go_live_checklist (item_key, title, critical)
VALUES
    ('provider_sandbox_tested', 'Provider sandbox tested', true),
    ('provider_live_credentials_configured', 'Provider live credentials configured', true),
    ('webhook_live_endpoints_verified', 'Webhook live endpoints verified', true),
    ('payout_bank_verified', 'Payout bank verified', true),
    ('business_account_kyc_flow_tested', 'Business account KYC flow tested', true),
    ('payment_link_test_successful', 'Payment link test successful', true),
    ('refund_test_successful', 'Refund test successful', true),
    ('dispute_test_simulated', 'Dispute test simulated', false),
    ('payout_test_successful', 'Payout test successful', true),
    ('reconciliation_import_tested', 'Reconciliation import tested', true),
    ('ledger_balanced', 'Ledger balanced', true),
    ('reports_export_tested', 'Reports export tested', false),
    ('rbac_review_completed', 'RBAC review completed', true),
    ('compliance_controls_reviewed', 'Compliance controls reviewed', true),
    ('incident_contacts_configured', 'Incident contacts configured', true),
    ('quality_gate_passing', 'Quality gate passing', true)
ON CONFLICT (item_key) DO NOTHING;

INSERT INTO admin_permissions (code, description) VALUES
    ('security_review.view', 'View security review'),
    ('security_review.manage', 'Manage security findings and step-up actions'),
    ('go_live.view', 'View go-live readiness'),
    ('go_live.manage', 'Manage production cutover')
ON CONFLICT (code) DO NOTHING;
