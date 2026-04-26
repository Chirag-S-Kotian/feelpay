-- Enable Row Level Security for Tenant Isolation

-- Helper function to get current tenant from session
CREATE OR REPLACE FUNCTION get_current_tenant_id() RETURNS UUID AS $$
    SELECT NULLIF(current_setting('app.current_tenant_id', TRUE), '')::UUID;
$$ LANGUAGE sql STABLE;

-- Helper function to check if user is platform admin
CREATE OR REPLACE FUNCTION is_platform_admin() RETURNS BOOLEAN AS $$
    -- This assumes platform admins have a specific role or are in a specific tenant (e.g., 'platform')
    -- For now, let's assume if app.is_platform_admin is set to 'true'
    SELECT current_setting('app.is_platform_admin', TRUE) = 'true';
$$ LANGUAGE sql STABLE;

-- Enable RLS on tables
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE merchants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE payouts ENABLE ROW LEVEL SECURITY;
ALTER TABLE bank_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE subscription_invoices ENABLE ROW LEVEL SECURITY;
ALTER TABLE fee_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment_orders ENABLE ROW LEVEL SECURITY;

-- Tenants Policy (Only platform admins can see all, others see only their own)
CREATE POLICY tenant_isolation_policy ON tenants
    USING (id = get_current_tenant_id() OR is_platform_admin());

-- Merchants Policy
CREATE POLICY merchant_isolation_policy ON merchants
    USING (tenant_id = get_current_tenant_id() OR is_platform_admin());

-- Users Policy
CREATE POLICY user_isolation_policy ON users
    USING (tenant_id = get_current_tenant_id() OR is_platform_admin());

-- For tables that link via merchant_id, we might need to join or add tenant_id
-- Let's add tenant_id to these tables for better performance with RLS

ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE payouts ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE subscription_invoices ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE fee_rules ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE payment_orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Backfill tenant_id from merchants table
UPDATE api_keys k SET tenant_id = m.tenant_id FROM merchants m WHERE k.merchant_id = m.id AND k.tenant_id IS NULL;
UPDATE payouts p SET tenant_id = m.tenant_id FROM merchants m WHERE p.merchant_id = m.id AND p.tenant_id IS NULL;
UPDATE bank_accounts b SET tenant_id = m.tenant_id FROM merchants m WHERE b.merchant_id = m.id AND b.tenant_id IS NULL;
UPDATE subscriptions s SET tenant_id = m.tenant_id FROM merchants m WHERE s.merchant_id = m.id AND s.tenant_id IS NULL;
UPDATE subscription_invoices i SET tenant_id = m.tenant_id FROM merchants m WHERE i.merchant_id = m.id AND i.tenant_id IS NULL;
UPDATE fee_rules f SET tenant_id = m.tenant_id FROM merchants m WHERE f.merchant_id = m.id AND f.tenant_id IS NULL;
UPDATE payment_orders o SET tenant_id = m.tenant_id FROM merchants m WHERE o.merchant_id = m.id AND o.tenant_id IS NULL;

-- Policies for tables with tenant_id
CREATE POLICY api_key_isolation_policy ON api_keys USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY payout_isolation_policy ON payouts USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY bank_account_isolation_policy ON bank_accounts USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY subscription_isolation_policy ON subscriptions USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY subscription_invoice_isolation_policy ON subscription_invoices USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY fee_rule_isolation_policy ON fee_rules USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
CREATE POLICY payment_order_isolation_policy ON payment_orders USING (tenant_id = get_current_tenant_id() OR is_platform_admin());
