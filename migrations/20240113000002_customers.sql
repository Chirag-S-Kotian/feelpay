-- Customers table - tokenized customer data
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    
    -- Customer identifier (can be email, phone, or internal ID)
    customer_reference VARCHAR(255) NOT NULL,
    customer_reference_type VARCHAR(20) NOT NULL CHECK (customer_reference_type IN ('email', 'phone', 'external_id')),
    
    -- Tokenized contact info (for communications)
    email VARCHAR(255),
    phone VARCHAR(20),
    name VARCHAR(255),
    
    -- Consent tracking for DPDP Act compliance
    consent_given BOOLEAN DEFAULT FALSE,
    consent_timestamp TIMESTAMPTZ,
    consent_purpose TEXT,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    UNIQUE(merchant_id, customer_reference, customer_reference_type)
);

-- Indexes for customers
CREATE INDEX IF NOT EXISTS idx_customers_merchant_id ON customers(merchant_id);
CREATE INDEX IF NOT EXISTS idx_customers_tenant_id ON customers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_customers_customer_reference ON customers(customer_reference);
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers(email) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_customers_phone ON customers(phone) WHERE phone IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_customers_created_at ON customers(created_at);
CREATE INDEX IF NOT EXISTS idx_customers_deleted_at ON customers(deleted_at) WHERE deleted_at IS NULL;

-- Soft delete trigger
CREATE OR REPLACE FUNCTION update_customers_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_customers_updated_at
    BEFORE UPDATE ON customers
    FOR EACH ROW
    EXECUTE FUNCTION update_customers_updated_at();

-- Saved Payment Methods table
CREATE TABLE IF NOT EXISTS saved_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    
    -- Payment method type
    method_type VARCHAR(20) NOT NULL CHECK (method_type IN ('card', 'upi', 'bank_account', 'wallet')),
    
    -- Tokenized payment details (vault reference)
    vault_token VARCHAR(255) NOT NULL,
    
    -- Card specific (if applicable)
    card_brand VARCHAR(20),
    card_last4 VARCHAR(4),
    card_expiry_month INTEGER,
    card_expiry_year INTEGER,
    card_type VARCHAR(20), -- debit, credit
    card_network VARCHAR(20), -- visa, mastercard, rupay
    
    -- UPI specific (if applicable)
    vpa VARCHAR(255),
    
    -- Bank account specific (if applicable)
    bank_name VARCHAR(100),
    bank_account_last4 VARCHAR(4),
    ifsc_code VARCHAR(11),
    
    -- Status
    is_default BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes for saved_payment_methods
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_customer_id ON saved_payment_methods(customer_id);
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_merchant_id ON saved_payment_methods(merchant_id);
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_tenant_id ON saved_payment_methods(tenant_id);
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_vault_token ON saved_payment_methods(vault_token);
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_is_default ON saved_payment_methods(is_default) WHERE is_default = TRUE;
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_is_active ON saved_payment_methods(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_created_at ON saved_payment_methods(created_at);
CREATE INDEX IF NOT EXISTS idx_saved_payment_methods_deleted_at ON saved_payment_methods(deleted_at) WHERE deleted_at IS NULL;

-- Soft delete trigger
CREATE OR REPLACE FUNCTION update_saved_payment_methods_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_saved_payment_methods_updated_at
    BEFORE UPDATE ON saved_payment_methods
    FOR EACH ROW
    EXECUTE FUNCTION update_saved_payment_methods_updated_at();