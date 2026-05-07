import { Page } from '@playwright/test';
import jwt from 'jsonwebtoken';
import { TEST_ADMIN_ID } from './testData';

export const ADMIN_ROLES = {
  SUPER_ADMIN: 'Super Admin',
  PAYMENT_OPS: 'Payment Ops Admin',
  FINANCE: 'Finance Admin',
  RISK: 'Risk Admin',
  COMPLIANCE: 'Compliance Admin',
  SUPPORT: 'Support Agent',
  AUDITOR: 'Auditor',
} as const;

export type AdminRole = typeof ADMIN_ROLES[keyof typeof ADMIN_ROLES];

export async function loginAsAdmin(page: Page, role: AdminRole = ADMIN_ROLES.SUPER_ADMIN) {
  const secret = process.env.JWT_SECRET || 'feelpay_jwt_secret';
  const adminId = TEST_ADMIN_ID;
  const permissions = getPermissionsForRole(role);

  const payload = {
    sub: adminId,
    merchant_id: adminId,
    role,
    permissions,
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + (60 * 60)
  };
  
  const token = jwt.sign(payload, secret);

  await page.goto('/login');
  await page.evaluate((t) => {
    localStorage.setItem('feelpay_token', t);
  }, token);
  await page.goto('/');
}

export function adminAuthHeaders(role: AdminRole = ADMIN_ROLES.SUPER_ADMIN) {
  const secret = process.env.JWT_SECRET || 'feelpay_jwt_secret';
  const payload = {
    sub: TEST_ADMIN_ID,
    merchant_id: TEST_ADMIN_ID,
    role,
    permissions: getPermissionsForRole(role),
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + (60 * 60),
  };

  return {
    Authorization: `Bearer ${jwt.sign(payload, secret)}`,
    'Content-Type': 'application/json',
  };
}

function getPermissionsForRole(role: AdminRole): string[] {
  const allPermissions = [
    'payment_ops.view', 'transactions.view', 'transactions.action',
    'refunds.request', 'refunds.approve', 'refunds.initiate',
    'settlements.manage', 'disputes.manage', 'risk_rules.manage',
    'reconciliation.manage', 'providers.manage', 'credentials.rotate',
    'bank_integrations.manage', 'routing.manage', 'webhook_config.manage',
    'reports.export', 'audit_logs.view', 'admin_users.view', 'admin_users.manage',
    'roles.view', 'roles.manage', 'system_health.view', 'system_health.manage',
    'notifications.view', 'notifications.manage', 'platform_settings.view',
    'platform_settings.manage', 'ledger.view', 'ledger.export',
    'developer_tools.view', 'api_keys.manage', 'api_logs.view',
    'webhook_replay.manage', 'compliance.view', 'compliance.manage'
  ];

  switch (role) {
    case ADMIN_ROLES.SUPER_ADMIN:
      return allPermissions;
    case ADMIN_ROLES.PAYMENT_OPS:
      return ['payment_ops.view', 'transactions.view', 'transactions.action', 'providers.manage', 'routing.manage'];
    case ADMIN_ROLES.FINANCE:
      return ['ledger.view', 'reports.export', 'settlements.manage', 'refunds.request'];
    case ADMIN_ROLES.RISK:
      return ['risk_rules.manage', 'disputes.manage', 'transactions.view'];
    case ADMIN_ROLES.COMPLIANCE:
      return ['compliance.view', 'audit_logs.view', 'credentials.rotate'];
    case ADMIN_ROLES.SUPPORT:
      return ['transactions.view', 'refunds.request', 'notifications.view'];
    case ADMIN_ROLES.AUDITOR:
      return allPermissions.filter(p => p.endsWith('.view'));
    default:
      return [];
  }
}
