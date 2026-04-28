-- Extend audit_logs table to support new operation types for export, reporting, and notification features
-- Requirement 3.6, 4.10, 6.10, 7.7, 8.8, 9.6, 10.9, 12.1

-- The audit_logs table already exists with columns: id, entity_type, entity_id, action, old_state, new_state, actor_type, actor_id, ip_address, created_at
-- This migration documents the new action types that will be used for the export/reporting/notification system

-- New action types for export/reporting/notification system:
-- Export operations:
--   - export_job_created: When merchant/admin creates export job
--   - export_job_started: When export job processing begins
--   - export_job_completed: When export job completes successfully
--   - export_job_failed: When export job fails
--   - export_job_downloaded: When merchant downloads export file
--
-- Scheduled report operations:
--   - scheduled_report_created: When merchant creates scheduled report
--   - scheduled_report_updated: When merchant updates scheduled report
--   - scheduled_report_deleted: When merchant deletes scheduled report
--   - scheduled_report_executed: When scheduled report is generated and emailed
--
-- Notification operations:
--   - notification_preference_updated: When merchant updates notification preferences
--   - notification_delivery_logged: When notification is sent/failed
--
-- Bank account operations:
--   - bank_account_added: When merchant adds bank account
--   - bank_account_updated: When merchant updates bank account
--   - bank_account_verified: When bank account penny drop verification succeeds
--   - bank_account_deleted: When merchant deletes bank account
--
-- Webhook operations:
--   - webhook_test_sent: When merchant/admin sends test webhook
--
-- Report operations:
--   - settlement_report_downloaded: When merchant downloads settlement report

-- Add index on action column for faster filtering of new operation types
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);

-- Add index on entity_type for faster filtering by resource type
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity_type ON audit_logs(entity_type);

-- Add composite index for common queries (entity_type + created_at)
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity_created ON audit_logs(entity_type, created_at DESC);
