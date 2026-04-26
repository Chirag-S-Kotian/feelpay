-- Add MFA columns to merchants table
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN DEFAULT false;
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS mfa_secret VARCHAR(255);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;
