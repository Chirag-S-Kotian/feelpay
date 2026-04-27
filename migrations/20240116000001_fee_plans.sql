-- Fee Plans and Merchant Fee Assignment

-- Fee plans table
CREATE TABLE IF NOT EXISTS fee_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- MDR rates (in basis points, 100 bps = 1%)
    card_debit_bps INT DEFAULT 200, -- 2%
    card_credit_bps INT DEFAULT 250, -- 2.5%
    upi_bps INT DEFAULT 0, -- 0% for UPI
    netbanking_bps INT DEFAULT 200,
    wallet_bps INT DEFAULT 200,
    emi_bps INT DEFAULT 300,
    
    -- Flat fees (in paise)
    card_flat_fee INT DEFAULT 0,
    upi_flat_fee INT DEFAULT 0,
    netbanking_flat_fee INT DEFAULT 200,
    wallet_flat_fee INT DEFAULT 0,
    
    -- GST on fees
    gst_percentage DECIMAL(5,2) DEFAULT 18.00,
    gst_inclusive BOOLEAN DEFAULT FALSE,
    
    -- Limits
    min_transaction_amount INT DEFAULT 100, -- 1 INR in paise
    max_transaction_amount INT DEFAULT 5000000, -- 50,000 INR
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,
    
    -- Versioning
    version INT DEFAULT 1,
    parent_plan_id UUID REFERENCES fee_plans(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_fee_plans_tenant_id ON fee_plans(tenant_id);
CREATE INDEX idx_fee_plans_is_active ON fee_plans(is_active) WHERE deleted_at IS NULL;

-- Merchant fee plan assignment
CREATE TABLE IF NOT EXISTS merchant_fee_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    fee_plan_id UUID NOT NULL REFERENCES fee_plans(id),
    
    -- Custom overrides (null = use plan defaults)
    custom_card_debit_bps INT,
    custom_card_credit_bps INT,
    custom_upi_bps INT,
    custom_netbanking_bps INT,
    custom_wallet_bps INT,
    
    -- Effective dates
    effective_from TIMESTAMPTZ DEFAULT NOW(),
    effective_until TIMESTAMPTZ,
    
    -- Audit
    assigned_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(merchant_id, effective_from)
);

CREATE INDEX idx_merchant_fee_assignments_merchant_id ON merchant_fee_assignments(merchant_id);
CREATE INDEX idx_merchant_fee_assignments_effective ON merchant_fee_assignments(effective_from, effective_until);

-- Auto-verification providers config
CREATE TABLE IF NOT EXISTS verification_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_type VARCHAR(50) NOT NULL, -- 'pan', 'gstin', 'aadhaar', 'bank_penny_drop'
    provider_name VARCHAR(100) NOT NULL, -- 'nsdl', 'utiitsel', 'cashfree', 'razorpay'
    
    api_endpoint VARCHAR(500),
    api_key_ref VARCHAR(255), -- Reference to secrets manager
    api_secret_ref VARCHAR(255),
    
    is_active BOOLEAN DEFAULT TRUE,
    priority INT DEFAULT 1, -- Lower = higher priority
    
    rate_limit_per_minute INT DEFAULT 100,
    timeout_ms INT DEFAULT 30000,
    
    config JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_verification_providers_type ON verification_providers(provider_type);

-- Auto-verification logs
CREATE TABLE IF NOT EXISTS verification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID REFERENCES merchants(id) ON DELETE CASCADE,
    verification_type VARCHAR(50) NOT NULL,
    provider VARCHAR(100),
    
    request_payload JSONB,
    response_payload JSONB,
    
    status VARCHAR(50) NOT NULL, -- 'success', 'failed', 'pending', 'manual_review'
    error_message TEXT,
    
    verification_score INT, -- 0-100 confidence score
    verified_fields JSONB,
    
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_verification_logs_merchant_id ON verification_logs(merchant_id);
CREATE INDEX idx_verification_logs_type ON verification_logs(verification_type, created_at);

-- Settlement batches
CREATE TABLE IF NOT EXISTS settlement_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_date DATE NOT NULL,
    batch_type VARCHAR(50) DEFAULT 'daily', -- 'daily', 'instant', 'manual'
    
    total_merchants INT DEFAULT 0,
    total_amount BIGINT DEFAULT 0, -- in paise
    total_transactions INT DEFAULT 0,
    
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    
    processed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(batch_date, batch_type)
);

CREATE INDEX idx_settlement_batches_date ON settlement_batches(batch_date);
CREATE INDEX idx_settlement_batches_status ON settlement_batches(status);

-- Batch settlements (link settlements to batches)
ALTER TABLE settlements ADD COLUMN IF NOT EXISTS batch_id UUID REFERENCES settlement_batches(id);
