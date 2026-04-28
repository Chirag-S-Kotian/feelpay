-- Notification Delivery Logs Table
-- Logs all notification delivery attempts for audit and troubleshooting

CREATE TABLE IF NOT EXISTS notification_delivery_logs (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  event_type VARCHAR(100) NOT NULL,
  delivery_channel VARCHAR(20) NOT NULL,
  recipient VARCHAR(255) NOT NULL,
  payload JSONB NOT NULL,
  status VARCHAR(20) NOT NULL,
  error_message TEXT,
  retry_count INT DEFAULT 0,
  sent_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_merchant_id ON notification_delivery_logs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_event_type ON notification_delivery_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_status ON notification_delivery_logs(status);
CREATE INDEX IF NOT EXISTS idx_notification_delivery_logs_created_at ON notification_delivery_logs(created_at);
