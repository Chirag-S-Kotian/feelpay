-- Add Aadhaar and Phone to merchants
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS phone VARCHAR(15);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS aadhaar_number_masked VARCHAR(12);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS aadhaar_verified BOOLEAN DEFAULT false;

-- Add index on phone for faster lookup
CREATE INDEX IF NOT EXISTS idx_merchants_phone ON merchants(phone);
