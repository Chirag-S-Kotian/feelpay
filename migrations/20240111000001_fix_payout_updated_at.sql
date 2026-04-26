-- Fix Payout and Bank Account schema inconsistencies
-- Adds updated_at column and registers trigger for automated updates

ALTER TABLE payouts ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Register triggers
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_payouts_updated_at') THEN
        CREATE TRIGGER update_payouts_updated_at
            BEFORE UPDATE ON payouts
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_bank_accounts_updated_at') THEN
        CREATE TRIGGER update_bank_accounts_updated_at
            BEFORE UPDATE ON bank_accounts
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;
