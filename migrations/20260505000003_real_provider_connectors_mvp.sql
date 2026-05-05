CREATE TABLE IF NOT EXISTS payment_provider_capabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id) ON DELETE CASCADE,
    capability VARCHAR(60) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    status VARCHAR(30) NOT NULL DEFAULT 'unknown',
    last_checked_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, capability)
);

WITH capability_matrix(provider_type, capability) AS (
    VALUES
        ('mock', 'payments'),
        ('mock', 'refunds'),
        ('mock', 'transfers'),
        ('mock', 'reversals'),
        ('mock', 'payouts'),
        ('mock', 'disputes'),
        ('mock', 'settlements'),
        ('mock', 'reconciliation_reports'),
        ('stripe', 'payments'),
        ('stripe', 'refunds'),
        ('stripe', 'transfers'),
        ('stripe', 'reversals'),
        ('stripe', 'payouts'),
        ('stripe', 'disputes'),
        ('stripe', 'settlements'),
        ('stripe', 'reconciliation_reports'),
        ('razorpay', 'payments'),
        ('razorpay', 'refunds'),
        ('razorpay', 'transfers'),
        ('razorpay', 'reversals'),
        ('razorpay', 'payouts'),
        ('razorpay', 'disputes'),
        ('razorpay', 'settlements'),
        ('razorpay', 'reconciliation_reports'),
        ('adyen', 'payments'),
        ('adyen', 'refunds'),
        ('adyen', 'transfers'),
        ('adyen', 'reversals'),
        ('adyen', 'payouts'),
        ('adyen', 'disputes'),
        ('adyen', 'settlements'),
        ('adyen', 'reconciliation_reports')
)
INSERT INTO payment_provider_capabilities (provider_id, capability, enabled, status)
SELECT p.id,
       cm.capability,
       p.provider_type = 'mock',
       CASE WHEN p.provider_type = 'mock' THEN 'ready' ELSE 'credential_required' END
FROM payment_providers p
JOIN capability_matrix cm ON cm.provider_type = p.provider_type
ON CONFLICT (provider_id, capability) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_payment_provider_capabilities_provider
    ON payment_provider_capabilities(provider_id, capability);
