-- System ledger accounts required by LedgerService::record_transaction.
-- Without them every capture and refund fails with "Platform account not configured".
INSERT INTO ledger_accounts (name, account_type, currency)
SELECT 'FeelPay Platform Fees', 'platform', 'INR'
WHERE NOT EXISTS (SELECT 1 FROM ledger_accounts WHERE account_type = 'platform');

INSERT INTO ledger_accounts (name, account_type, currency)
SELECT 'FeelPay Escrow', 'escrow', 'INR'
WHERE NOT EXISTS (SELECT 1 FROM ledger_accounts WHERE account_type = 'escrow');
