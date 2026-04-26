-- UPI transactions for collect/intent/qr flows
CREATE TABLE IF NOT EXISTS upi_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_order_id UUID NOT NULL REFERENCES payment_orders(id),
    upi_txn_id VARCHAR(64) UNIQUE,
    payer_vpa VARCHAR(255) NOT NULL,
    payee_vpa VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL,
    flow_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'initiated',
    rrn VARCHAR(20),
    approval_number VARCHAR(10),
    npci_response_code VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_upi_txn_payment ON upi_transactions(payment_order_id);
CREATE INDEX IF NOT EXISTS idx_upi_txn_id ON upi_transactions(upi_txn_id);
CREATE INDEX IF NOT EXISTS idx_upi_txn_payer ON upi_transactions(payer_vpa);
