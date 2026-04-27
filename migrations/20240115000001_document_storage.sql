-- Document Storage for KYC and other uploads
-- Documents are stored with metadata in DB, actual files in object storage

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    document_type VARCHAR(50) NOT NULL, -- 'pan_card', 'gst_certificate', 'bank_statement', 'aadhaar', 'cancelled_cheque', 'other'
    document_purpose VARCHAR(50) NOT NULL, -- 'kyc', 'dispute_evidence', 'settlement_proof', 'other'
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(100) NOT NULL, -- MIME type
    file_size BIGINT NOT NULL, -- in bytes
    storage_key VARCHAR(500) NOT NULL, -- Object storage key/path
    storage_bucket VARCHAR(100), -- Bucket name if different from default
    checksum VARCHAR(128), -- SHA-256 hash for integrity
    status VARCHAR(50) DEFAULT 'pending_review', -- 'pending_review', 'approved', 'rejected', 'expired'
    rejection_reason TEXT,
    verified_at TIMESTAMPTZ,
    verified_by UUID REFERENCES users(id) ON DELETE SET NULL,
    expires_at TIMESTAMPTZ, -- For documents that expire (e.g., GST certificate)
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_documents_merchant_id ON documents(merchant_id);
CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_purpose ON documents(document_purpose);

-- Document access log for audit
CREATE TABLE IF NOT EXISTS document_access_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    accessed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    access_type VARCHAR(50) NOT NULL, -- 'view', 'download', 'delete'
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_document_access_document_id ON document_access_log(document_id);
CREATE INDEX idx_document_access_accessed_by ON document_access_log(accessed_by);

-- OTP verifications table
CREATE TABLE IF NOT EXISTS otp_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    merchant_id UUID REFERENCES merchants(id) ON DELETE CASCADE,
    email VARCHAR(255),
    phone VARCHAR(20),
    otp_hash VARCHAR(255) NOT NULL,
    otp_type VARCHAR(50) NOT NULL, -- 'email_verification', 'phone_verification', 'login', 'password_reset', 'transaction'
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'verified', 'expired', 'used'
    attempts INT DEFAULT 0,
    max_attempts INT DEFAULT 5,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_otp_verifications_user_id ON otp_verifications(user_id);
CREATE INDEX idx_otp_verifications_merchant_id ON otp_verifications(merchant_id);
CREATE INDEX idx_otp_verifications_email ON otp_verifications(email);
CREATE INDEX idx_otp_verifications_phone ON otp_verifications(phone);
CREATE INDEX idx_otp_verifications_expires_at ON otp_verifications(expires_at) WHERE status = 'pending';

-- Maker-checker approval workflow
CREATE TABLE IF NOT EXISTS approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL, -- 'merchant_kyc', 'payout', 'settlement', 'fee_change'
    entity_id UUID NOT NULL,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    request_type VARCHAR(50) NOT NULL, -- 'approve', 'reject', 'modify'
    request_data JSONB,
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'cancelled'
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_approval_requests_entity ON approval_requests(entity_type, entity_id);
CREATE INDEX idx_approval_requests_status ON approval_requests(status);
CREATE INDEX idx_approval_requests_requested_by ON approval_requests(requested_by);

-- Audit trail for approval workflow
CREATE TABLE IF NOT EXISTS approval_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    approval_request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL, -- 'created', 'approved', 'rejected', 'cancelled', 'escalated'
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_approval_audit_request_id ON approval_audit_log(approval_request_id);
