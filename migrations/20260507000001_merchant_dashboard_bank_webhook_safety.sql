-- Merchant dashboard safety hardening for bank account lifecycle.

ALTER TABLE bank_accounts
    ADD COLUMN IF NOT EXISTS account_number_encrypted TEXT NULL,
    ADD COLUMN IF NOT EXISTS account_type TEXT NOT NULL DEFAULT 'savings',
    ADD COLUMN IF NOT EXISTS verification_status TEXT NOT NULL DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS penny_drop_amount_paise INT NULL,
    ADD COLUMN IF NOT EXISTS penny_drop_reference TEXT NULL,
    ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ NULL;

ALTER TABLE bank_accounts
    ALTER COLUMN account_number DROP NOT NULL;

ALTER TABLE bank_accounts
    DROP CONSTRAINT IF EXISTS valid_verification_status;

ALTER TABLE bank_accounts
    ADD CONSTRAINT valid_verification_status
    CHECK (verification_status IN ('pending', 'verified', 'failed', 'deactivated'));

UPDATE bank_accounts
SET verification_status = CASE WHEN COALESCE(is_verified, false) THEN 'verified' ELSE verification_status END,
    account_number = CASE
        WHEN account_number IS NULL OR account_number LIKE 'XXXX...%' THEN account_number
        WHEN length(account_number) >= 4 THEN 'XXXX...' || right(account_number, 4)
        ELSE 'XXXX...XXXX'
    END
WHERE account_number IS NOT NULL;
