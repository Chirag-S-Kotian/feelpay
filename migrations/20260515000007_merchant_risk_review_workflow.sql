ALTER TABLE merchants
    ADD COLUMN IF NOT EXISTS review_status VARCHAR(40) NOT NULL DEFAULT 'draft',
    ADD COLUMN IF NOT EXISTS review_assigned_admin_id UUID REFERENCES admin_users(id),
    ADD COLUMN IF NOT EXISTS review_last_decided_at TIMESTAMPTZ;

UPDATE merchants
SET review_status = CASE
        WHEN onboarding_status = 'approved' THEN 'approved'
        WHEN onboarding_status = 'rejected' THEN 'rejected'
        WHEN onboarding_status = 'suspended' THEN 'suspended'
        WHEN onboarding_status IN ('pending', 'submitted') THEN 'submitted'
        WHEN onboarding_status IN ('in_progress', 'under_review') THEN 'under_review'
        ELSE 'draft'
    END
WHERE review_status = 'draft';

CREATE TABLE IF NOT EXISTS merchant_review_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    actor_admin_id UUID REFERENCES admin_users(id),
    action VARCHAR(60) NOT NULL,
    from_status VARCHAR(40),
    to_status VARCHAR(40),
    risk_level VARCHAR(40),
    reason_code VARCHAR(60),
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_merchant_review_events_merchant_created
    ON merchant_review_events(merchant_id, created_at DESC);
