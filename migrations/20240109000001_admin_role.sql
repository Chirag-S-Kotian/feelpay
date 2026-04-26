-- Add admin role to merchants
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT false;

-- Create a default super admin if none exists (for development)
-- password is 'admin123' hashed
-- UPDATE merchants SET is_admin = true WHERE email = 'test@feelpay.com';
