-- Subscription Plans
CREATE TABLE IF NOT EXISTS subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL, -- in paise
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    billing_cycle VARCHAR(50) NOT NULL, -- daily, weekly, monthly, quarterly, yearly
    trial_days INTEGER,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_subscription_plans_merchant ON subscription_plans(merchant_id);
CREATE INDEX idx_subscription_plans_active ON subscription_plans(active);

-- Subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE SET NULL,
    plan_id UUID REFERENCES subscription_plans(id) ON DELETE SET NULL,
    plan_name VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL, -- in paise
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    billing_cycle VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active', -- active, paused, cancelled, expired, pending
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    trial_start TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_merchant ON subscriptions(merchant_id);
CREATE INDEX idx_subscriptions_customer ON subscriptions(customer_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_period_end ON subscriptions(current_period_end);

-- Subscription Invoices
CREATE TABLE IF NOT EXISTS subscription_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE SET NULL,
    amount BIGINT NOT NULL, -- in paise
    currency VARCHAR(3) NOT NULL DEFAULT 'INR',
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, paid, failed, void
    payment_order_id UUID REFERENCES payment_orders(id) ON DELETE SET NULL,
    invoice_number VARCHAR(100) NOT NULL UNIQUE,
    due_date TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscription_invoices_subscription ON subscription_invoices(subscription_id);
CREATE INDEX idx_subscription_invoices_merchant ON subscription_invoices(merchant_id);
CREATE INDEX idx_subscription_invoices_status ON subscription_invoices(status);
CREATE INDEX idx_subscription_invoices_due_date ON subscription_invoices(due_date);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_subscriptions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_subscriptions_updated_at();

CREATE TRIGGER subscription_invoices_updated_at
    BEFORE UPDATE ON subscription_invoices
    FOR EACH ROW
    EXECUTE FUNCTION update_subscriptions_updated_at();

CREATE TRIGGER subscription_plans_updated_at
    BEFORE UPDATE ON subscription_plans
    FOR EACH ROW
    EXECUTE FUNCTION update_subscriptions_updated_at();
