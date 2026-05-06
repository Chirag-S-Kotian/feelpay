const jwt = require('jsonwebtoken');

const secret = 'your_secure_secret_256_bits_minimum_change_in_production';
const payload = {
  sub: '00000000-0000-0000-0000-000000000000',
  role: 'admin',
  permissions: [
    'payment_ops.view',
    'transactions.view',
    'ledger.view',
    'providers.manage',
    'bank_integrations.manage',
    'webhook_config.manage',
    'developer_tools.view',
    'audit_logs.view',
    'readiness_scorecard.view',
    'compliance.view',
    'system_health.view'
  ],
  iat: Math.floor(Date.now() / 1000),
  exp: Math.floor(Date.now() / 1000) + (60 * 60 * 24 * 365) // 1 year
};

const token = jwt.sign(payload, secret);
console.log(token);
