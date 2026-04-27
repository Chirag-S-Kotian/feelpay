-- Provider Routing & Payment Method Configuration

-- Payment providers
CREATE TABLE IF NOT EXISTS payment_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE, -- razorpay, payu, cashfree, juspay
    display_name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    
    -- Provider config (encrypted in production)
    api_key_hash VARCHAR(255),
    api_secret_hash VARCHAR(255),
    webhook_secret_hash VARCHAR(255),
    
    -- Health monitoring
    success_rate DECIMAL(5,2) DEFAULT 100.00,
    avg_latency_ms INTEGER DEFAULT 0,
    last_health_check TIMESTAMPTZ,
    
    -- Circuit breaker
    circuit_breaker_state VARCHAR(20) DEFAULT 'closed' CHECK (circuit_breaker_state IN ('closed', 'open', 'half_open')),
    failure_count INTEGER DEFAULT 0,
    last_failure TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Provider routing rules
CREATE TABLE IF NOT EXISTS provider_routing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    merchant_id UUID REFERENCES merchants(id) ON DELETE CASCADE,
    
    -- Rule conditions
    payment_method VARCHAR(20) NOT NULL, -- card, upi, netbanking, wallet
    card_brand VARCHAR(20), -- visa, mastercard, rupay
    amount_min INTEGER,
    amount_max INTEGER,
    
    -- Routing target
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    priority INTEGER DEFAULT 100, -- lower = higher priority
    
    is_active BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Provider health log
CREATE TABLE IF NOT EXISTS provider_health_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES payment_providers(id),
    
    check_type VARCHAR(20) NOT NULL, -- health, transaction
    is_success BOOLEAN NOT NULL,
    latency_ms INTEGER,
    error_code VARCHAR(50),
    error_message TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_provider_routing_merchant ON provider_routing_rules(merchant_id);
CREATE INDEX IF NOT EXISTS idx_provider_routing_method ON provider_routing_rules(payment_method);
CREATE INDEX IF NOT EXISTS idx_provider_health_logs_provider ON provider_health_logs(provider_id);
CREATE INDEX IF NOT EXISTS idx_provider_health_logs_created ON provider_health_logs(created_at);

-- Insert default providers
INSERT INTO payment_providers (name, display_name, is_active, is_default) VALUES
    ('simulator', 'Test Simulator', true, true),
    ('razorpay', 'Razorpay', false, false),
    ('payu', 'PayU', false, false),
    ('cashfree', 'Cashfree', false, false),
    ('juspay', 'Juspay', false, false)
ON CONFLICT (name) DO NOTHING;

-- Trigger for provider updated_at
CREATE OR REPLACE FUNCTION update_payment_providers_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_payment_providers_updated_at
    BEFORE UPDATE ON payment_providers
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_providers_updated_at();

-- Trigger for routing rules updated_at
CREATE OR REPLACE FUNCTION update_provider_routing_rules_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_provider_routing_rules_updated_at
    BEFORE UPDATE ON provider_routing_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_provider_routing_rules_updated_at();