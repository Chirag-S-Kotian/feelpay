-- Token vault for PCI scope reduction
CREATE TABLE IF NOT EXISTS card_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    token VARCHAR(64) UNIQUE NOT NULL,           -- Non-reversible token
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    last_four VARCHAR(4) NOT NULL,
    card_brand VARCHAR(20) NOT NULL,              -- visa/mastercard/rupay
    expiry_month SMALLINT NOT NULL,
    expiry_year SMALLINT NOT NULL,
    encrypted_pan VARCHAR(255) NOT NULL,                 -- AES-256-GCM encrypted base64
    encryption_key_id VARCHAR(64) NOT NULL,       -- Key rotation support
    nonce VARCHAR(255) NOT NULL,                  -- Initialization vector/nonce
    customer_id UUID,
    holder_name VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_card_tokens_merchant ON card_tokens(merchant_id);
CREATE INDEX IF NOT EXISTS idx_card_tokens_customer ON card_tokens(customer_id);
