-- Merchant dashboard RBAC roles required by docs/goal.md.
INSERT INTO roles (name, description, is_system)
VALUES
    ('merchant_owner', 'Owner access to all merchant features', true),
    ('merchant_admin', 'Admin access to merchant operations and settings', true),
    ('merchant_developer', 'Developer access to API keys, webhooks, logs, and docs', true),
    ('merchant_finance', 'Finance access to transactions, settlements, refunds, and exports', true),
    ('merchant_support', 'Support access to customers, transactions, and disputes', true),
    ('merchant_read_only', 'Read-only access to merchant dashboard data', true)
ON CONFLICT DO NOTHING;
