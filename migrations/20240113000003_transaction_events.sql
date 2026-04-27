-- Transaction Events - append-only state log for audit trail
CREATE TABLE IF NOT EXISTS transaction_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL,
    merchant_id UUID NOT NULL,
    tenant_id UUID,
    
    -- Event details
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}',
    
    -- Status change tracking
    from_status VARCHAR(30),
    to_status VARCHAR(30) NOT NULL,
    
    -- Actor info (who/what triggered the event)
    actor_type VARCHAR(20), -- 'system', 'merchant', 'customer', 'admin', 'provider'
    actor_id VARCHAR(255),
    
    -- Provider reference (for external events)
    provider_name VARCHAR(50),
    provider_event_id VARCHAR(255),
    
    -- IP address for audit
    ip_address INET,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for transaction_events (partitioned by time, so time-based indexes are critical)
CREATE INDEX IF NOT EXISTS idx_transaction_events_transaction_id ON transaction_events(transaction_id);
CREATE INDEX IF NOT EXISTS idx_transaction_events_merchant_id ON transaction_events(merchant_id);
CREATE INDEX IF NOT EXISTS idx_transaction_events_tenant_id ON transaction_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_transaction_events_event_type ON transaction_events(event_type);
CREATE INDEX IF NOT EXISTS idx_transaction_events_created_at ON transaction_events(created_at);
CREATE INDEX IF NOT EXISTS idx_transaction_events_from_status ON transaction_events(from_status);
CREATE INDEX IF NOT EXISTS idx_transaction_events_to_status ON transaction_events(to_status);

-- Composite index for transaction timeline queries
CREATE INDEX IF NOT EXISTS idx_transaction_events_txn_created 
    ON transaction_events(transaction_id, created_at);

-- Notification Queue table
CREATE TABLE IF NOT EXISTS notification_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Notification type
    notification_type VARCHAR(50) NOT NULL, -- 'email', 'sms', 'whatsapp', 'webhook', 'in_app'
    
    -- Recipient
    recipient_type VARCHAR(20) NOT NULL, -- 'merchant', 'customer', 'admin', 'user'
    recipient_id UUID,
    recipient_email VARCHAR(255),
    recipient_phone VARCHAR(20),
    
    -- Content
    subject VARCHAR(500),
    body TEXT,
    template_id VARCHAR(100),
    template_data JSONB DEFAULT '{}',
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending' 
        CHECK (status IN ('pending', 'sent', 'failed', 'cancelled', 'retrying')),
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    
    -- Scheduling
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    
    -- Error tracking
    error_message TEXT,
    error_code VARCHAR(50),
    
    -- External reference
    external_id VARCHAR(255), -- message ID from provider
    
    -- Related entity (for context)
    entity_type VARCHAR(50),
    entity_id UUID,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for notification_queue
CREATE INDEX IF NOT EXISTS idx_notification_queue_status ON notification_queue(status);
CREATE INDEX IF NOT EXISTS idx_notification_queue_scheduled_at ON notification_queue(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_notification_queue_recipient_id ON notification_queue(recipient_id);
CREATE INDEX IF NOT EXISTS idx_notification_queue_entity ON notification_queue(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_notification_queue_created_at ON notification_queue(created_at);

-- Trigger for notification_queue updated_at
CREATE OR REPLACE FUNCTION update_notification_queue_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_notification_queue_updated_at
    BEFORE UPDATE ON notification_queue
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_queue_updated_at();

-- Report Jobs table for async report generation
CREATE TABLE IF NOT EXISTS report_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID REFERENCES merchants(id) ON DELETE SET NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    
    -- Report type
    report_type VARCHAR(50) NOT NULL, -- 'transactions', 'settlements', 'refunds', 'gst', 'disputes'
    
    -- Parameters
    date_from DATE,
    date_to DATE,
    filters JSONB DEFAULT '{}',
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    
    -- Output
    file_name VARCHAR(255),
    file_url TEXT,
    file_size_bytes BIGINT,
    file_format VARCHAR(10) CHECK (file_format IN ('csv', 'xlsx', 'pdf')),
    
    -- Error tracking
    error_message TEXT,
    
    -- Requester
    requested_by UUID,
    
    -- Scheduled delivery
    email_on_complete BOOLEAN DEFAULT FALSE,
    email_recipients JSONB DEFAULT '[]',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for report_jobs
CREATE INDEX IF NOT EXISTS idx_report_jobs_merchant_id ON report_jobs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_report_jobs_tenant_id ON report_jobs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_report_jobs_status ON report_jobs(status);
CREATE INDEX IF NOT EXISTS idx_report_jobs_report_type ON report_jobs(report_type);
CREATE INDEX IF NOT EXISTS idx_report_jobs_created_at ON report_jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_report_jobs_requested_by ON report_jobs(requested_by);

-- Trigger for report_jobs updated_at
CREATE OR REPLACE FUNCTION update_report_jobs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_report_jobs_updated_at
    BEFORE UPDATE ON report_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_report_jobs_updated_at();