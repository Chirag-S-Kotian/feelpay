const jwt = require('jsonwebtoken');

const secret = 'your_secure_secret_256_bits_minimum_change_in_production';
const merchant_id = '00000000-0000-0000-0000-000000000000';
const payload = {
  sub: merchant_id,
  merchant_id: merchant_id,
  role: 'Super Admin',
  permissions: [
    'payment_ops.view',
    'transactions.view',
    'transactions.action',
    'refunds.request',
    'refunds.approve',
    'refunds.initiate',
    'settlements.manage',
    'disputes.manage',
    'risk_rules.manage',
    'reconciliation.manage',
    'providers.manage',
    'credentials.rotate',
    'bank_integrations.manage',
    'routing.manage',
    'webhook_config.manage',
    'reports.export',
    'audit_logs.view',
    'admin_users.view',
    'admin_users.manage',
    'roles.view',
    'roles.manage',
    'system_health.view',
    'system_health.manage',
    'notifications.view',
    'notifications.manage',
    'platform_settings.view',
    'platform_settings.manage',
    'ledger.view',
    'ledger.export',
    'developer_tools.view',
    'api_keys.manage',
    'api_logs.view',
    'webhook_replay.manage',
    'compliance.view',
    'compliance.manage'
  ],
  iat: Math.floor(Date.now() / 1000),
  exp: Math.floor(Date.now() / 1000) + (60 * 60 * 24 * 365) // 1 year
};

const token = jwt.sign(payload, secret);
console.log(token);
