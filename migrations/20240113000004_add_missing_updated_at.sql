-- Add missing updated_at columns to existing tables

-- Add updated_at to payouts table if not exists
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'payouts') THEN
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'payouts' AND column_name = 'updated_at'
        ) THEN
            ALTER TABLE payouts ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            
            -- Create trigger function if not exists
            CREATE OR REPLACE FUNCTION update_payouts_updated_at()
            RETURNS TRIGGER AS $func$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $func$ LANGUAGE plpgsql;
            
            -- Create trigger
            DROP TRIGGER IF EXISTS trigger_payouts_updated_at ON payouts;
            CREATE TRIGGER trigger_payouts_updated_at
                BEFORE UPDATE ON payouts
                FOR EACH ROW
                EXECUTE FUNCTION update_payouts_updated_at();
        END IF;
    END IF;
END $$;

-- Add updated_at to bank_accounts table if not exists
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'bank_accounts') THEN
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'bank_accounts' AND column_name = 'updated_at'
        ) THEN
            ALTER TABLE bank_accounts ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            
            -- Create trigger function if not exists
            CREATE OR REPLACE FUNCTION update_bank_accounts_updated_at()
            RETURNS TRIGGER AS $func$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $func$ LANGUAGE plpgsql;
            
            -- Create trigger
            DROP TRIGGER IF EXISTS trigger_bank_accounts_updated_at ON bank_accounts;
            CREATE TRIGGER trigger_bank_accounts_updated_at
                BEFORE UPDATE ON bank_accounts
                FOR EACH ROW
                EXECUTE FUNCTION update_bank_accounts_updated_at();
        END IF;
    END IF;
END $$;

-- Add updated_at to vpa_accounts table if not exists
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'vpa_accounts') THEN
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'vpa_accounts' AND column_name = 'updated_at'
        ) THEN
            ALTER TABLE vpa_accounts ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            
            -- Create trigger function if not exists
            CREATE OR REPLACE FUNCTION update_vpa_accounts_updated_at()
            RETURNS TRIGGER AS $func$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $func$ LANGUAGE plpgsql;
            
            -- Create trigger
            DROP TRIGGER IF EXISTS trigger_vpa_accounts_updated_at ON vpa_accounts;
            CREATE TRIGGER trigger_vpa_accounts_updated_at
                BEFORE UPDATE ON vpa_accounts
                FOR EACH ROW
                EXECUTE FUNCTION update_vpa_accounts_updated_at();
        END IF;
    END IF;
END $$;