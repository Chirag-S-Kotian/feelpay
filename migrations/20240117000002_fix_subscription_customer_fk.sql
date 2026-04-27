-- Bring subscriptions tables in line with the current service model

ALTER TABLE subscriptions
    ADD COLUMN IF NOT EXISTS plan_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS amount BIGINT,
    ADD COLUMN IF NOT EXISTS currency VARCHAR(3),
    ADD COLUMN IF NOT EXISTS billing_cycle VARCHAR(50),
    ADD COLUMN IF NOT EXISTS trial_start TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS trial_end TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMPTZ;

UPDATE subscriptions
SET
    plan_name = COALESCE(plan_name, 'Custom Plan'),
    amount = COALESCE(amount, 0),
    currency = COALESCE(currency, 'INR'),
    billing_cycle = COALESCE(billing_cycle, 'monthly')
WHERE
    plan_name IS NULL
    OR amount IS NULL
    OR currency IS NULL
    OR billing_cycle IS NULL;

ALTER TABLE subscriptions
    ALTER COLUMN plan_name SET NOT NULL,
    ALTER COLUMN amount SET NOT NULL,
    ALTER COLUMN currency SET NOT NULL,
    ALTER COLUMN billing_cycle SET NOT NULL;

ALTER TABLE subscription_invoices
    ADD COLUMN IF NOT EXISTS customer_id UUID,
    ADD COLUMN IF NOT EXISTS currency VARCHAR(3),
    ADD COLUMN IF NOT EXISTS invoice_number VARCHAR(100),
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;

UPDATE subscription_invoices si
SET customer_id = s.customer_id
FROM subscriptions s
WHERE si.subscription_id = s.id
  AND si.customer_id IS NULL;

UPDATE subscription_invoices
SET
    currency = COALESCE(currency, 'INR'),
    invoice_number = COALESCE(
        invoice_number,
        'INV-' || to_char(COALESCE(created_at, NOW()), 'YYYYMMDDHH24MISS') || '-' || substr(id::text, 1, 8)
    ),
    updated_at = COALESCE(updated_at, NOW())
WHERE
    currency IS NULL
    OR invoice_number IS NULL
    OR updated_at IS NULL;

ALTER TABLE subscription_invoices
    ALTER COLUMN customer_id SET NOT NULL,
    ALTER COLUMN currency SET NOT NULL,
    ALTER COLUMN invoice_number SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'subscription_invoices_invoice_number_key'
    ) THEN
        ALTER TABLE subscription_invoices
            ADD CONSTRAINT subscription_invoices_invoice_number_key UNIQUE (invoice_number);
    END IF;
END $$;

ALTER TABLE subscriptions
    DROP CONSTRAINT IF EXISTS subscriptions_customer_id_fkey,
    ADD CONSTRAINT subscriptions_customer_id_fkey
        FOREIGN KEY (customer_id)
        REFERENCES customers(id)
        ON DELETE RESTRICT;

ALTER TABLE subscription_invoices
    DROP CONSTRAINT IF EXISTS subscription_invoices_customer_id_fkey,
    ADD CONSTRAINT subscription_invoices_customer_id_fkey
        FOREIGN KEY (customer_id)
        REFERENCES customers(id)
        ON DELETE RESTRICT;
