-- Move merchant API-key lookup to hashed developer_api_keys rows and
-- prepare encrypted webhook-secret storage.

ALTER TABLE merchants
    ALTER COLUMN webhook_secret DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS webhook_secret_ciphertext TEXT,
    ADD COLUMN IF NOT EXISTS webhook_secret_nonce TEXT,
    ADD COLUMN IF NOT EXISTS webhook_secret_hint VARCHAR(32);

ALTER TABLE merchants
    ALTER COLUMN api_key DROP NOT NULL;

ALTER TABLE merchants
    ADD COLUMN IF NOT EXISTS pan_ciphertext TEXT,
    ADD COLUMN IF NOT EXISTS pan_nonce TEXT,
    ADD COLUMN IF NOT EXISTS gstin_ciphertext TEXT,
    ADD COLUMN IF NOT EXISTS gstin_nonce TEXT;

ALTER TABLE kyc_verifications
    ADD COLUMN IF NOT EXISTS document_number_ciphertext TEXT,
    ADD COLUMN IF NOT EXISTS document_number_nonce TEXT;

-- Legacy plaintext API-key columns are retained only for rollout/backfill
-- compatibility. Runtime auth no longer reads them after the app change.
CREATE INDEX IF NOT EXISTS idx_developer_api_keys_merchant_env_status
    ON developer_api_keys (merchant_id, environment, status, created_at DESC)
    WHERE merchant_id IS NOT NULL;
