-- Create notification_delivery_logs table for tracking notification delivery attempts
-- Requirement 12.1, 12.2: Notification Delivery Logging

CREATE TABLE IF NOT EXISTS notification_delivery_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    delivery_channel VARCHAR(20) NOT NULL, -- 'email', 'sms', 'in_app'
    recipient VARCHAR(255) NOT NULL, -- email, phone, or user_id
    payload JSONB NOT NULL, -- full notification payload
    status VARCHAR(20) NOT NULL, -- 'sent', 'failed', 'bounced'
    error_message TEXT,
    retry_count INT DEFAULT 0,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_delivery_channel CHECK (delivery_channel IN ('email', 'sms', 'in_app')),
    CONSTRAINT valid_status CHECK (status IN ('sent', 'failed', 'bounced'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_merchant_id ON notification_delivery_logs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_event_type ON notification_delivery_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_status ON notification_delivery_logs(status);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_created_at ON notification_delivery_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_delivery_channel ON notification_delivery_logs(delivery_channel);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_merchant_event ON notification_delivery_logs(merchant_id, event_type);
