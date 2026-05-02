-- Extend audit_logs table to support new operation types for export, reporting, and notification features

-- Add CHECK constraint to document supported action types
-- This ensures only valid operations are logged
ALTER TABLE audit_logs
ADD CONSTRAINT check_valid_action CHECK (
    action IN (
        -- Existing operations (from previous migrations)
        'create', 'update', 'delete', 'read',
        
        -- Export operations
        'export_job_created',
        'export_job_completed',
        'export_job_failed',
        
        -- Scheduled report operations
        'scheduled_report_created',
        'scheduled_report_updated',
        'scheduled_report_deleted',
        
        -- Notification preference operations
        'notification_preference_updated',
        
        -- Bank account operations
        'bank_account_added',
        'bank_account_updated',
        'bank_account_verified',
        
        -- Webhook operations
        'webhook_test_sent',
        
        -- Settlement report operations
        'settlement_report_downloaded',
        
        -- Notification delivery operations
        'notification_delivery_logged'
    )
);

-- Add index on action column for faster filtering of specific operation types
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);

-- Add composite index for common query patterns: entity_type + action + created_at
CREATE INDEX IF NOT EXISTS idx_audit_entity_action_time ON audit_logs(entity_type, action, created_at DESC);

-- Add index on actor_id for tracking operations by specific users/admins
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_id, created_at DESC);
