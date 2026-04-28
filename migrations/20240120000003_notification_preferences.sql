-- Create notification_preferences table for per-event notification channel configuration
-- Requirement 6.2, 6.3: Notification Preferences per Event Type

CREATE TABLE IF NOT EXISTS notification_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE, -- NULL = merchant-level default, NOT NULL = user-level override
    event_type VARCHAR(100) NOT NULL, -- 'payment.success', 'payment.failed', 'refund.initiated', etc.
    email_enabled BOOLEAN DEFAULT true,
    sms_enabled BOOLEAN DEFAULT false,
    in_app_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, user_id, event_type)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_notification_preferences_merchant_id ON notification_preferences(merchant_id);
CREATE INDEX IF NOT EXISTS idx_notification_preferences_user_id ON notification_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_notification_preferences_event_type ON notification_preferences(event_type);
CREATE INDEX IF NOT EXISTS idx_notification_preferences_merchant_event ON notification_preferences(merchant_id, event_type);
