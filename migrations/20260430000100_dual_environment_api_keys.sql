-- Dual environment API keys (test/live)
ALTER TABLE merchants
    ADD COLUMN IF NOT EXISTS api_key_test VARCHAR(255),
    ADD COLUMN IF NOT EXISTS api_key_live VARCHAR(255);

-- Backfill existing merchants for non-breaking rollout.
UPDATE merchants
SET api_key_test = COALESCE(api_key_test, api_key),
    api_key_live = COALESCE(api_key_live, api_key)
WHERE api_key IS NOT NULL;

-- Keep uniqueness per environment key when present.
CREATE UNIQUE INDEX IF NOT EXISTS idx_merchants_api_key_test_unique
    ON merchants (api_key_test)
    WHERE api_key_test IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_merchants_api_key_live_unique
    ON merchants (api_key_live)
    WHERE api_key_live IS NOT NULL;
