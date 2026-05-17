-- Update password_hash to a valid Argon2id hash for StrongPass123!
UPDATE users 
SET password_hash = '$argon2id$v=19$m=19456,t=2,p=1$402diGTCu8e8q7Y9/e1Ehg$RsVYrVbHeYU8oUSzYo9/KFT0WQfA9J++2jryPBH3oBc'
WHERE email IN ('test@feelpay.com', 'merchant@example.com', 'test@merchant.com', 'testmerchant@example.com');

UPDATE merchants 
SET password_hash = '$argon2id$v=19$m=19456,t=2,p=1$402diGTCu8e8q7Y9/e1Ehg$RsVYrVbHeYU8oUSzYo9/KFT0WQfA9J++2jryPBH3oBc'
WHERE email IN ('test@feelpay.com', 'merchant@example.com', 'test@merchant.com', 'testmerchant@example.com');
