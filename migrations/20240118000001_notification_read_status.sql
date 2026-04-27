-- Add is_read and read_at to notification_queue
ALTER TABLE notification_queue ADD COLUMN IF NOT EXISTS is_read BOOLEAN DEFAULT FALSE;
ALTER TABLE notification_queue ADD COLUMN IF NOT EXISTS read_at TIMESTAMPTZ;

-- Add index for unread notifications performance
CREATE INDEX IF NOT EXISTS idx_notification_queue_unread ON notification_queue(recipient_id, is_read) WHERE is_read = FALSE;
