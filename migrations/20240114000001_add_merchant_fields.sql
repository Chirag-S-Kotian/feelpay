-- Add missing merchant fields for onboarding
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS onboarding_status VARCHAR(20) DEFAULT 'pending';
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS tier VARCHAR(20) DEFAULT 'basic';
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS pan VARCHAR(10);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS gstin VARCHAR(15);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS address JSONB;

-- Update limit columns to BIGINT to match Rust i64
ALTER TABLE merchants ALTER COLUMN per_transaction_limit TYPE BIGINT;
ALTER TABLE merchants ALTER COLUMN daily_limit TYPE BIGINT;
ALTER TABLE merchants ALTER COLUMN monthly_limit TYPE BIGINT;
