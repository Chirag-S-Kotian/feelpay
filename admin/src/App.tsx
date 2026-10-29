import { BrowserRouter, Routes, Route, NavLink, Navigate, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Settings as SettingsIcon,
  Menu,
  X,
  Bell,
  Search,
  Receipt,
  Landmark,
  Scale,
  ListChecks,
  Cable,
  CreditCard,
  RotateCcw,
  Radar,
  FileSearch,
  Building2,
  Webhook,
  ShieldCheck,
  Route as RouteIcon,
  ClipboardList,
  UserCog,
  HeartPulse,
  WalletCards,
  Code2,
  Boxes,
  MessageSquareText,
  Mail,
  Lock,
  Eye,
  EyeOff,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import { FormEvent, ReactNode, useEffect, useState } from 'react';
import { apiRequest, clearToken, getToken, setToken } from './hooks/useApi';
import AdminIntegrationCenter from './pages/AdminIntegrationCenter';
import AdminNotificationTemplates from './pages/AdminNotificationTemplates';
import AdminDashboard from './pages/AdminDashboard';
import AdminBulkExport from './pages/AdminBulkExport';
import Revenue from './pages/Revenue';
import {
  AdminUsersRolesPage,
  AuditLogsPage,
  BankIntegrationsPage,
  BusinessAccountsPage,
  ComplianceCenterPage,
  ReadinessScorecardPage,
  DeveloperIntegrationPage,
  DisputesPage,
  LedgerPage,
  MoneyMovementPage,
  PaymentOpsPage,
  PlatformSettingsPage,
  ProviderConnectorsPage,
  ReconciliationPage,
  RefundsPage,
  ReportsPage,
  RiskRulesPage,
  RoutingRulesPage,
  SettlementsPage,
  SystemHealthPage,
  TransactionsPage,
  NotificationsPage,
  WebhookConfigPage,
  WhiteLabelCheckoutPage,
} from './pages/CompanyAdminOps';

type AdminPermission =
  | 'payment_ops.view'
  | 'transactions.view'
  | 'transactions.action'
  | 'refunds.request'
  | 'refunds.approve'
  | 'refunds.initiate'
  | 'settlements.manage'
  | 'disputes.manage'
  | 'risk_rules.manage'
  | 'reconciliation.manage'
  | 'providers.manage'
  | 'credentials.rotate'
  | 'bank_integrations.manage'
  | 'routing.manage'
  | 'webhook_config.manage'
  | 'reports.export'
  | 'audit_logs.view'
  | 'admin_users.view'
  | 'admin_users.manage'
  | 'roles.view'
  | 'roles.manage'
  | 'system_health.view'
  | 'system_health.manage'
  | 'notifications.view'
  | 'notifications.manage'
  | 'platform_settings.view'
  | 'platform_settings.manage'
  | 'ledger.view'
  | 'ledger.export'
  | 'developer_tools.view'
  | 'api_keys.manage'
  | 'api_logs.view'
  | 'webhook_replay.manage'
  | 'compliance.view'
  | 'compliance.manage';

type NavItem = {
  path: string;
  label: string;
  icon: typeof LayoutDashboard;
  permission: AdminPermission;
};

type NavSection = {
  title: string;
  items: NavItem[];
};

const NAV_SECTIONS: NavSection[] = [
  {
    title: 'Overview',
    items: [{ path: '/', label: 'Overview', icon: LayoutDashboard, permission: 'payment_ops.view' }],
  },
  {
    title: 'Operations',
    items: [
      { path: '/payment-ops', label: 'Payment Ops', icon: CreditCard, permission: 'payment_ops.view' },
      { path: '/transactions', label: 'Transactions', icon: Receipt, permission: 'transactions.view' },
      { path: '/ledger', label: 'Ledger', icon: WalletCards, permission: 'ledger.view' },
      { path: '/refunds', label: 'Refunds', icon: RotateCcw, permission: 'refunds.request' },
      { path: '/settlements', label: 'Settlements', icon: Landmark, permission: 'settlements.manage' },
      { path: '/disputes', label: 'Payment Issues', icon: Scale, permission: 'disputes.manage' },
      { path: '/risk-rules', label: 'Fraud Protection', icon: Radar, permission: 'risk_rules.manage' },
      { path: '/reconciliation', label: 'Payment Matching', icon: FileSearch, permission: 'reconciliation.manage' },
    ],
  },
  {
    title: 'Accounts',
    items: [{ path: '/business-accounts', label: 'Business Accounts', icon: Building2, permission: 'payment_ops.view' }],
  },
  {
    title: 'Configuration',
    items: [
      { path: '/connectors', label: 'Payment Gateways', icon: Cable, permission: 'providers.manage' },
      { path: '/integration-center', label: 'Email & SMS Services', icon: Boxes, permission: 'providers.manage' },
      { path: '/notification-templates', label: 'Notification Templates', icon: MessageSquareText, permission: 'providers.manage' },
      { path: '/bank-integrations', label: 'Settlement Bank Accounts', icon: Landmark, permission: 'bank_integrations.manage' },
      { path: '/routing-rules', label: 'Smart Payment Routing', icon: RouteIcon, permission: 'routing.manage' },
      { path: '/white-label-checkout', label: 'Platform Checkout', icon: ShieldCheck, permission: 'webhook_config.manage' },
      { path: '/webhook-config', label: 'Webhook Config', icon: Webhook, permission: 'webhook_config.manage' },
      { path: '/developer-tools', label: 'Developer Tools', icon: Code2, permission: 'developer_tools.view' },
    ],
  },
  {
    title: 'Reports',
    items: [
      { path: '/reports', label: 'Reports', icon: ClipboardList, permission: 'reports.export' },
    ],
  },
  {
    title: 'Admin',
    items: [
      { path: '/audit-logs', label: 'Audit Logs', icon: ListChecks, permission: 'audit_logs.view' },
      { path: '/admin-users-roles', label: 'Admin Users & Roles', icon: UserCog, permission: 'admin_users.view' },
      { path: '/system-health', label: 'System Health', icon: HeartPulse, permission: 'system_health.view' },
      { path: '/notifications', label: 'System Alerts', icon: Bell, permission: 'notifications.view' },
      { path: '/compliance-center', label: 'Trust & Security', icon: ShieldCheck, permission: 'compliance.view' },
      { path: '/settings', label: 'Platform Settings', icon: SettingsIcon, permission: 'platform_settings.view' },
    ],
  },
];

function Sidebar({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <img
          src="/feelpay-logo.png"
          onError={(e) => {
            const target = e.currentTarget;
            if (!target.src.endsWith('/feelpay.png')) {
              target.src = '/feelpay.png';
            }
          }}
          alt="FeelPay"
          className="logo-mark"
        />
        <span className="logo-text">FeelPay</span>
      </div>
      <nav className="sidebar-nav">
        {NAV_SECTIONS.map(section => (
          <div key={section.title} className="sidebar-nav-section">
            <div className="sidebar-nav-title">{section.title}</div>
            {section.items.map(n => (
              <NavLink
                key={n.path}
                to={n.path}
                end={n.path === '/'}
                onClick={onNavigate}
                className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}
              >
                <n.icon size={18} />
                <span>{n.label}</span>
              </NavLink>
            ))}
          </div>
        ))}
      </nav>
      <div className="env-panel">
        <div className="env-label">Platform</div>
        <div className="env-badge">
          <span className="env-dot"></span>
          <span>Admin</span>
        </div>
      </div>
    </aside>
  );
}

function Layout() {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [notificationCount] = useState(2); // TODO: fetch from API
  const handleLogout = () => {
    clearToken();
    navigate('/login', { replace: true });
  };
  const handleSearch = (event: FormEvent) => {
    event.preventDefault();
    const query = searchQuery.trim();
    if (query) {
      navigate(`/transactions?q=${encodeURIComponent(query)}`);
    }
  };

  return (
    <div className="app-layout">
      {open && <div className="mobile-overlay" onClick={() => setOpen(false)} />}
      <div className={`mobile-sidebar ${open ? 'open' : ''}`}>
        <Sidebar onNavigate={() => setOpen(false)} />
      </div>
      <div className="desktop-sidebar">
        <Sidebar />
      </div>
      <main className="main-content">
        <header className="top-header">
          <button
            type="button"
            className="mobile-header-menu-btn"
            onClick={() => setOpen(!open)}
            aria-label={open ? 'Close navigation' : 'Open navigation'}
          >
            {open ? <X size={20} /> : <Menu size={20} />}
          </button>
          <form className="global-search" onSubmit={handleSearch}>
            <Search size={16} />
            <input
              type="text"
              placeholder="Search payments, accounts, or providers"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              aria-label="Search payments, accounts, or providers"
            />
          </form>
          <button className="notification-bell" aria-label="Notifications">
            <Bell size={20} />
            {notificationCount > 0 && (
              <span className="notification-badge">{notificationCount}</span>
            )}
          </button>
          <button className="btn-secondary" onClick={handleLogout}>Logout</button>
        </header>
        <Routes>
          <Route path="/" element={<AdminDashboard />} />
          <Route path="/operations" element={<Navigate to="/payment-ops" replace />} />
          <Route path="/payment-ops" element={<RequirePermission permission="payment_ops.view"><PaymentOpsPage /></RequirePermission>} />
          <Route path="/business-accounts" element={<RequirePermission permission="payment_ops.view"><BusinessAccountsPage /></RequirePermission>} />
          <Route path="/merchants" element={<Navigate to="/business-accounts" replace />} />
          <Route path="/transactions" element={<RequirePermission permission="transactions.view"><TransactionsPage /></RequirePermission>} />
          <Route path="/ledger" element={<RequirePermission permission="ledger.view"><LedgerPage /></RequirePermission>} />
          <Route path="/money-movement" element={<RequirePermission permission="settlements.manage"><MoneyMovementPage /></RequirePermission>} />
          <Route path="/refunds" element={<RequirePermission permission="refunds.request"><RefundsPage /></RequirePermission>} />
          <Route path="/settlements" element={<RequirePermission permission="settlements.manage"><SettlementsPage /></RequirePermission>} />
          <Route path="/disputes" element={<RequirePermission permission="disputes.manage"><DisputesPage /></RequirePermission>} />
          <Route path="/risk-rules" element={<RequirePermission permission="risk_rules.manage"><RiskRulesPage /></RequirePermission>} />
          <Route path="/reconciliation" element={<RequirePermission permission="reconciliation.manage"><ReconciliationPage /></RequirePermission>} />
          <Route path="/connectors" element={<RequirePermission permission="providers.manage"><ProviderConnectorsPage /></RequirePermission>} />
          <Route path="/integration-center" element={<RequirePermission permission="providers.manage"><AdminIntegrationCenter /></RequirePermission>} />
          <Route path="/notification-templates" element={<RequirePermission permission="providers.manage"><AdminNotificationTemplates /></RequirePermission>} />
          <Route path="/bank-integrations" element={<RequirePermission permission="bank_integrations.manage"><BankIntegrationsPage /></RequirePermission>} />
          <Route path="/routing-rules" element={<RequirePermission permission="routing.manage"><RoutingRulesPage /></RequirePermission>} />
          <Route path="/white-label-checkout" element={<RequirePermission permission="webhook_config.manage"><WhiteLabelCheckoutPage /></RequirePermission>} />
          <Route path="/webhook-config" element={<RequirePermission permission="webhook_config.manage"><WebhookConfigPage /></RequirePermission>} />
          <Route path="/developer-tools" element={<RequirePermission permission="developer_tools.view"><DeveloperIntegrationPage /></RequirePermission>} />
          <Route path="/developer" element={<Navigate to="/developer-tools" replace />} />
          <Route path="/revenue" element={<RequirePermission permission="reports.export"><Revenue /></RequirePermission>} />
          <Route path="/bulk-export" element={<RequirePermission permission="reports.export"><AdminBulkExport /></RequirePermission>} />
          <Route path="/reports" element={<RequirePermission permission="reports.export"><ReportsPage /></RequirePermission>} />
          <Route path="/audit-logs" element={<RequirePermission permission="audit_logs.view"><AuditLogsPage /></RequirePermission>} />
          <Route path="/admin-users-roles" element={<RequirePermission permission="admin_users.view"><AdminUsersRolesPage /></RequirePermission>} />
          <Route path="/system-health" element={<RequirePermission permission="system_health.view"><SystemHealthPage /></RequirePermission>} />
          <Route path="/notifications" element={<RequirePermission permission="notifications.view"><NotificationsPage /></RequirePermission>} />
          <Route path="/compliance-center" element={<RequirePermission permission="compliance.view"><ComplianceCenterPage /></RequirePermission>} />
          <Route path="/trust-security" element={<RequirePermission permission="compliance.view"><ComplianceCenterPage /></RequirePermission>} />
          <Route path="/readiness-scorecard" element={<RequirePermission permission="compliance.view"><ReadinessScorecardPage /></RequirePermission>} />
          <Route path="/settings" element={<RequirePermission permission="platform_settings.view"><PlatformSettingsPage /></RequirePermission>} />
          <Route path="*" element={<UnknownAdminRoute />} />
        </Routes>
      </main>
    </div>
  );
}

function LoginPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const tokenFromQuery = params.get('token');
    if (!tokenFromQuery) {
      return;
    }
    setToken(tokenFromQuery);
    window.history.replaceState({}, '', '/');
    navigate('/', { replace: true });
  }, [navigate]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const data = await apiRequest<{ token: string }>('POST', '/admin/login', {
        email,
        password,
      });
      setToken(data.token);
      navigate('/', { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-container">
        <div className="login-branding">
          <div className="login-logo-wrap">
            <img
              src="/feelpay-logo.png"
              onError={(e) => {
                const target = e.currentTarget;
                if (!target.src.endsWith('/feelpay.png')) {
                  target.src = '/feelpay.png';
                }
              }}
              alt="FeelPay Logo"
              className="login-logo-img"
            />
          </div>
          <div className="login-env-tag">
            <span className="login-env-dot"></span>
            <span>Platform Administration</span>
          </div>
          <h1 className="login-title">FeelPay Admin Portal</h1>
          <p className="login-subtitle">Secure access for authorized company operations</p>
        </div>

        <div className="login-card">
          {error && (
            <div className="login-alert-error" role="alert">
              <AlertCircle size={18} className="login-alert-icon" />
              <span>{error}</span>
            </div>
          )}

          <form className="login-form" onSubmit={handleSubmit}>
            <div className="login-field-group">
              <label htmlFor="admin-email">Admin Email</label>
              <div className="login-input-wrapper">
                <Mail size={18} className="login-input-icon" />
                <input
                  id="admin-email"
                  type="email"
                  placeholder="admin@feelpay.com"
                  autoComplete="username"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  disabled={loading}
                />
              </div>
            </div>

            <div className="login-field-group">
              <div className="login-label-row">
                <label htmlFor="admin-password">Security Password</label>
              </div>
              <div className="login-input-wrapper">
                <Lock size={18} className="login-input-icon" />
                <input
                  id="admin-password"
                  type={showPassword ? 'text' : 'password'}
                  placeholder="••••••••••••"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  disabled={loading}
                />
                <button
                  type="button"
                  className="login-password-toggle"
                  onClick={() => setShowPassword(!showPassword)}
                  aria-label={showPassword ? 'Hide password' : 'Show password'}
                  tabIndex={-1}
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            <button className="login-submit-btn" type="submit" disabled={loading}>
              {loading ? (
                <>
                  <Loader2 size={18} className="login-spinner" />
                  <span>Authenticating...</span>
                </>
              ) : (
                <span>Sign In to Admin Portal</span>
              )}
            </button>
          </form>
        </div>

        <footer className="login-page-footer">
          <p>Strictly restricted to authorized personnel. All session events are cryptographically audited.</p>
        </footer>
      </div>
    </div>
  );
}

function ProtectedApp() {
  const token = getToken();
  if (!token) {
    return <Navigate to="/login" replace />;
  }
  const claims = getAdminClaims();
  if (!claims) {
    clearToken();
    return <Navigate to="/login" replace />;
  }
  const isSuper = claims.role === 'Super Admin' || claims.role === 'super_admin' || claims.role === 'admin';
  const hasAnyPerm = Boolean(claims.permissions && claims.permissions.length > 0);
  if (!isSuper && !hasAnyPerm) {
    clearToken();
    return <Navigate to="/login" replace />;
  }
  return <Layout />;
}

function RequirePermission({ permission, children }: { permission: AdminPermission; children: ReactNode }) {
  if (!hasPermission(permission)) {
    return (
      <div className="page fade-in">
        <div className="page-header">
          <div>
            <h1>Access denied</h1>
            <p>Company admin permission required: {permission}</p>
          </div>
        </div>
        <div className="card">Route blocked by RBAC deny-by-default policy.</div>
      </div>
    );
  }
  return <>{children}</>;
}

function UnknownAdminRoute() {
  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Access denied</h1>
          <p>Company admin route is not mapped.</p>
        </div>
      </div>
      <div className="card">Route blocked by RBAC deny-by-default policy.</div>
    </div>
  );
}

function hasPermission(permission: AdminPermission) {
  const claims = getAdminClaims();
  if (!claims) return false;
  if (claims.role === 'Super Admin' || claims.role === 'super_admin' || claims.role === 'admin') return true;
  return (claims.permissions || []).includes(permission);
}

function getAdminClaims(): { role?: string; permissions?: string[] } | null {
  const token = getToken();
  if (!token) return null;
  const payload = token.split('.')[1];
  if (!payload) return null;
  try {
    const json = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
    return JSON.parse(json) as { role?: string; permissions?: string[] };
  } catch {
    return null;
  }
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/*" element={<ProtectedApp />} />
      </Routes>
    </BrowserRouter>
  );
}
