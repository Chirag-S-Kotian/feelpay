import { BrowserRouter, Routes, Route, NavLink, Navigate, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Settings as SettingsIcon,
  Menu,
  X,
  Bell,
  Search,
  Download,
  Receipt,
  Landmark,
  Scale,
  IndianRupee,
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
} from 'lucide-react';
import { FormEvent, ReactNode, useEffect, useState } from 'react';
import { apiRequest, clearToken, getToken, setToken } from './hooks/useApi';
import AdminDashboard from './pages/AdminDashboard';
import AdminBulkExport from './pages/AdminBulkExport';
import Revenue from './pages/Revenue';
import {
  AdminUsersRolesPage,
  AuditLogsPage,
  BankIntegrationsPage,
  BusinessAccountsPage,
  DisputesPage,
  PaymentOpsPage,
  PlatformSettingsPage,
  ProviderConnectorsPage,
  ReconciliationPage,
  RefundsPage,
  ReportsPage,
  RiskRulesPage,
  RoutingRulesPage,
  SettlementsPage,
  TransactionsPage,
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
  | 'admin_users.manage'
  | 'platform_settings.manage';

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
      { path: '/refunds', label: 'Refunds', icon: RotateCcw, permission: 'refunds.request' },
      { path: '/settlements', label: 'Settlements', icon: Landmark, permission: 'settlements.manage' },
      { path: '/disputes', label: 'Disputes', icon: Scale, permission: 'disputes.manage' },
      { path: '/risk-rules', label: 'Risk Rules', icon: Radar, permission: 'risk_rules.manage' },
      { path: '/reconciliation', label: 'Reconciliation', icon: FileSearch, permission: 'reconciliation.manage' },
    ],
  },
  {
    title: 'Accounts',
    items: [{ path: '/business-accounts', label: 'Business Accounts', icon: Building2, permission: 'payment_ops.view' }],
  },
  {
    title: 'Configuration',
    items: [
      { path: '/connectors', label: 'Provider Connectors', icon: Cable, permission: 'providers.manage' },
      { path: '/bank-integrations', label: 'Bank Integrations', icon: Landmark, permission: 'bank_integrations.manage' },
      { path: '/routing-rules', label: 'Routing Rules', icon: RouteIcon, permission: 'routing.manage' },
      { path: '/white-label-checkout', label: 'White-label Checkout', icon: ShieldCheck, permission: 'webhook_config.manage' },
      { path: '/webhook-config', label: 'Webhook Config', icon: Webhook, permission: 'webhook_config.manage' },
    ],
  },
  {
    title: 'Reports',
    items: [
      { path: '/revenue', label: 'Revenue', icon: IndianRupee, permission: 'reports.export' },
      { path: '/bulk-export', label: 'Bulk Export', icon: Download, permission: 'reports.export' },
      { path: '/reports', label: 'Reports', icon: ClipboardList, permission: 'reports.export' },
    ],
  },
  {
    title: 'Admin',
    items: [
      { path: '/audit-logs', label: 'Audit Logs', icon: ListChecks, permission: 'audit_logs.view' },
      { path: '/admin-users-roles', label: 'Admin Users & Roles', icon: UserCog, permission: 'admin_users.manage' },
      { path: '/settings', label: 'Platform Settings', icon: SettingsIcon, permission: 'platform_settings.manage' },
    ],
  },
];

function Sidebar({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <div className="logo-icon">FP</div>
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
      <button className="mobile-nav-toggle" onClick={() => setOpen(!open)}>
        {open ? <X size={20} /> : <Menu size={20} />}
      </button>
      <div className={`mobile-sidebar ${open ? 'open' : ''}`}>
        <Sidebar onNavigate={() => setOpen(false)} />
      </div>
      <div className="desktop-sidebar">
        <Sidebar />
      </div>
      <main className="main-content">
        <header className="top-header">
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
          <Route path="/refunds" element={<RequirePermission permission="refunds.request"><RefundsPage /></RequirePermission>} />
          <Route path="/settlements" element={<RequirePermission permission="settlements.manage"><SettlementsPage /></RequirePermission>} />
          <Route path="/disputes" element={<RequirePermission permission="disputes.manage"><DisputesPage /></RequirePermission>} />
          <Route path="/risk-rules" element={<RequirePermission permission="risk_rules.manage"><RiskRulesPage /></RequirePermission>} />
          <Route path="/reconciliation" element={<RequirePermission permission="reconciliation.manage"><ReconciliationPage /></RequirePermission>} />
          <Route path="/connectors" element={<RequirePermission permission="providers.manage"><ProviderConnectorsPage /></RequirePermission>} />
          <Route path="/bank-integrations" element={<RequirePermission permission="bank_integrations.manage"><BankIntegrationsPage /></RequirePermission>} />
          <Route path="/routing-rules" element={<RequirePermission permission="routing.manage"><RoutingRulesPage /></RequirePermission>} />
          <Route path="/white-label-checkout" element={<RequirePermission permission="webhook_config.manage"><WhiteLabelCheckoutPage /></RequirePermission>} />
          <Route path="/webhook-config" element={<RequirePermission permission="webhook_config.manage"><WebhookConfigPage /></RequirePermission>} />
          <Route path="/revenue" element={<Revenue />} />
          <Route path="/bulk-export" element={<AdminBulkExport />} />
          <Route path="/reports" element={<RequirePermission permission="reports.export"><ReportsPage /></RequirePermission>} />
          <Route path="/audit-logs" element={<RequirePermission permission="audit_logs.view"><AuditLogsPage /></RequirePermission>} />
          <Route path="/admin-users-roles" element={<RequirePermission permission="admin_users.manage"><AdminUsersRolesPage /></RequirePermission>} />
          <Route path="/settings" element={<RequirePermission permission="platform_settings.manage"><PlatformSettingsPage /></RequirePermission>} />
          <Route path="*" element={<Navigate to="/" />} />
        </Routes>
      </main>
    </div>
  );
}

function LoginPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
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
      <form className="login-card" onSubmit={handleSubmit}>
        <div className="login-header">
          <h2>FeelPay Admin Login</h2>
          <p>Use internal company admin email + password</p>
        </div>
        {error && <div className="alert alert-error">{error}</div>}
        <input type="email" placeholder="Email" autoComplete="username" value={email} onChange={(e) => setEmail(e.target.value)} required />
        <input type="password" placeholder="Password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} required />
        <button className="btn-primary" type="submit" disabled={loading}>
          {loading ? 'Signing in...' : 'Sign In'}
        </button>
      </form>
    </div>
  );
}

function ProtectedApp() {
  if (!getToken()) {
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

function hasPermission(permission: AdminPermission) {
  const claims = getAdminClaims();
  if (!claims) return false;
  if (claims.role === 'Super Admin' || claims.role === 'super_admin' || claims.role === 'admin' || claims.role === 'merchant') return true;
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
