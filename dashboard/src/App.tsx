import { Suspense, lazy, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route, NavLink, Navigate, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  CreditCard,
  Wallet,
  BarChart3,
  Code2,
  ShieldAlert,
  Settings as SettingsIcon,
  Building2,
  Menu,
  X,
  Link as LinkIcon,
  Users,
  Bell,
  Send,
  SlidersHorizontal,
  Smartphone,
  LogOut,
} from 'lucide-react';
import { ErrorBoundary } from './components/ErrorBoundary';
import {
  clearMerchantSession,
  EnvironmentMode,
  getEnvironmentMode,
  isSessionExpired,
  setEnvironmentMode,
} from './hooks/useApi';
import { apiRequest } from './hooks/useApi';

const Dashboard = lazy(() => import('./pages/Dashboard'));
const Payments = lazy(() => import('./pages/Payments'));
const Settlements = lazy(() => import('./pages/Settlements'));
const Analytics = lazy(() => import('./pages/Analytics'));
const Developers = lazy(() => import('./pages/Developers'));
const Disputes = lazy(() => import('./pages/Disputes'));
const Settings = lazy(() => import('./pages/Settings'));
const Onboarding = lazy(() => import('./pages/Onboarding'));
const Login = lazy(() => import('./pages/Login'));
const Signup = lazy(() => import('./pages/Signup'));
const ForgotPassword = lazy(() => import('./pages/ForgotPassword'));
const Landing = lazy(() => import('./pages/Landing'));
const PaymentLinks = lazy(() => import('./pages/PaymentLinks'));
const Payouts = lazy(() => import('./pages/Payouts'));
const PaymentMethods = lazy(() => import('./pages/PaymentMethods'));
const CheckoutPreview = lazy(() => import('./pages/CheckoutPreview'));
const Customers = lazy(() => import('./pages/Customers'));
const Exports = lazy(() => import('./pages/Exports'));
const SettlementReports = lazy(() => import('./pages/SettlementReports'));
const ScheduledReports = lazy(() => import('./pages/ScheduledReports'));
const NotificationPreferences = lazy(() => import('./pages/NotificationPreferences'));
const WebhookTesting = lazy(() => import('./pages/WebhookTesting'));
const BankAccounts = lazy(() => import('./pages/BankAccounts'));

const routePreloaders: Record<string, () => Promise<unknown>> = {
  '/': () => import('./pages/Dashboard'),
  '/payments': () => import('./pages/Payments'),
  '/payment-links': () => import('./pages/PaymentLinks'),
  '/payouts': () => import('./pages/Payouts'),
  '/payment-methods': () => import('./pages/PaymentMethods'),
  '/checkout-preview': () => import('./pages/CheckoutPreview'),
  '/customers': () => import('./pages/Customers'),
  '/settlements': () => import('./pages/Settlements'),
  '/exports': () => import('./pages/Exports'),
  '/settlement-reports': () => import('./pages/SettlementReports'),
  '/scheduled-reports': () => import('./pages/ScheduledReports'),
  '/analytics': () => import('./pages/Analytics'),
  '/disputes': () => import('./pages/Disputes'),
  '/notifications': () => import('./pages/NotificationPreferences'),
  '/webhooks': () => import('./pages/WebhookTesting'),
  '/bank-accounts': () => import('./pages/BankAccounts'),
  '/developers': () => import('./pages/Developers'),
  '/settings': () => import('./pages/Settings'),
  '/onboarding': () => import('./pages/Onboarding'),
  '/login': () => import('./pages/Login'),
  '/signup': () => import('./pages/Signup'),
  '/forgot-password': () => import('./pages/ForgotPassword'),
};

const NAV = [
  { path: '/', label: 'Dashboard', icon: LayoutDashboard },
  { path: '/payments', label: 'Payments', icon: CreditCard },
  { path: '/payment-links', label: 'Payment Links', icon: LinkIcon },
  { path: '/payouts', label: 'Payouts', icon: Send },
  { path: '/payment-methods', label: 'Payment Methods', icon: SlidersHorizontal },
  { path: '/checkout-preview', label: 'Checkout Preview', icon: Smartphone },
  { path: '/customers', label: 'Customers', icon: Users },
  { path: '/settlements', label: 'Settlements', icon: Wallet },
  { path: '/exports', label: 'Exports', icon: BarChart3 },
  { path: '/settlement-reports', label: 'Settlement Reports', icon: BarChart3 },
  { path: '/scheduled-reports', label: 'Scheduled Reports', icon: BarChart3 },
  { path: '/analytics', label: 'Analytics', icon: BarChart3 },
  { path: '/disputes', label: 'Disputes', icon: ShieldAlert },
  { path: '/notifications', label: 'Notifications', icon: Bell },
  { path: '/webhooks', label: 'Webhooks', icon: Code2 },
  { path: '/bank-accounts', label: 'Bank Accounts', icon: Wallet },
  { path: '/developers', label: 'Developers', icon: Code2 },
  { path: '/settings', label: 'Settings', icon: SettingsIcon },
  { path: '/onboarding', label: 'Onboarding', icon: Building2 },
];

function Sidebar({
  onNavigate,
  mode,
  onModeChange,
}: {
  onNavigate?: () => void;
  mode: EnvironmentMode;
  onModeChange: (mode: EnvironmentMode) => Promise<void> | void;
}) {
  const preloadRoute = (path: string) => {
    routePreloaders[path]?.();
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <img src="/feelpay.png" alt="FeelPay" className="logo-mark" />
        <span className="logo-text">FeelPay</span>
      </div>
      <nav className="sidebar-nav">
        {NAV.map(n => (
          <NavLink
            key={n.path}
            to={n.path}
            end={n.path === '/'}
            onClick={onNavigate}
            onMouseEnter={() => preloadRoute(n.path)}
            onFocus={() => preloadRoute(n.path)}
            className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}
          >
            <n.icon size={18} aria-hidden="true" />
            {n.label}
          </NavLink>
        ))}
      </nav>
      <div className="env-panel">
        <div className="env-label">Environment</div>
        <div className="env-badge" style={{ marginBottom: 8 }}>
          <span className="env-dot"></span>
          {mode === 'live' ? 'Live Mode' : 'Test Mode'}
        </div>
        <div className="pill-tabs">
          <button
            className={`pill-tab ${mode === 'test' ? 'active' : ''}`}
            onClick={() => void onModeChange('test')}
            type="button"
          >
            Test
          </button>
          <button
            className={`pill-tab ${mode === 'live' ? 'active' : ''}`}
            onClick={() => void onModeChange('live')}
            type="button"
          >
            Live
          </button>
        </div>
      </div>
    </aside>
  );
}

function Layout() {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [notificationCount] = useState(0);
  const [mode, setMode] = useState<EnvironmentMode>(getEnvironmentMode());
  const [modeWarning, setModeWarning] = useState('');

  const handleLogout = async () => {
    try {
      await apiRequest('POST', '/v1/merchants/logout', {});
    } catch {
      // Local session cleanup still completes if network/API is already gone.
    } finally {
      clearMerchantSession();
      navigate('/login?reason=logout', { replace: true });
    }
  };

  useEffect(() => {
    setEnvironmentMode(mode);
  }, [mode]);

  const handleModeChange = async (nextMode: EnvironmentMode) => {
    if (nextMode === mode) return;
    if (nextMode === 'test') {
      setModeWarning('');
      setMode('test');
      return;
    }
    try {
      const check = await apiRequest<{
        can_go_live: boolean;
        blockers: string[];
      }>('GET', '/v1/onboarding/live-preflight', undefined, undefined, { cacheMs: 5_000 });
      if (!check.can_go_live) {
        setModeWarning(`Live mode blocked: ${check.blockers.join(' | ')}`);
        return;
      }
      setModeWarning('');
      setMode('live');
    } catch (err) {
      setModeWarning(err instanceof Error ? err.message : 'Unable to validate live mode preflight.');
    }
  };

  return (
    <div className="app-layout">
      <button
        className="mobile-nav-toggle"
        aria-label={open ? 'Close navigation' : 'Open navigation'}
        onClick={() => setOpen(!open)}
      >
        {open ? <X size={20} /> : <Menu size={20} />}
      </button>
      {open && <div className="mobile-overlay" onClick={() => setOpen(false)}></div>}
      <div className={`mobile-sidebar ${open ? 'open' : ''}`}>
        <Sidebar onNavigate={() => setOpen(false)} mode={mode} onModeChange={handleModeChange} />
      </div>
      <div className="desktop-sidebar">
        <Sidebar mode={mode} onModeChange={handleModeChange} />
      </div>
      <main className="main-content">
        <ErrorBoundary>
          {modeWarning && (
            <div className="card" style={{ marginBottom: 12, borderColor: 'rgba(245,158,11,0.45)', color: '#f5b96b' }}>
              {modeWarning}
            </div>
          )}
          <header className="top-header">
            <div className="header-spacer"></div>
            <button 
              className="notification-bell" 
              aria-label="Notifications"
              onClick={() => navigate('/notifications')}
            >
              <Bell size={20} />
              {notificationCount > 0 && (
                <span className="notification-badge">{notificationCount}</span>
              )}
            </button>
            <button className="logout-button" onClick={() => void handleLogout()} type="button">
              <LogOut size={18} />
              Logout
            </button>
          </header>
          <Suspense fallback={<RouteLoader />}>
            <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/payments" element={<Payments />} />
            <Route path="/payment-links" element={<PaymentLinks />} />
            <Route path="/payouts" element={<Payouts />} />
            <Route path="/payment-methods" element={<PaymentMethods />} />
            <Route path="/checkout-preview" element={<CheckoutPreview />} />
            <Route path="/customers" element={<Customers />} />
            <Route path="/settlements" element={<Settlements />} />
            <Route path="/exports" element={<Exports />} />
            <Route path="/settlement-reports" element={<SettlementReports />} />
            <Route path="/scheduled-reports" element={<ScheduledReports />} />
            <Route path="/analytics" element={<Analytics />} />
            <Route path="/disputes" element={<Disputes />} />
            <Route path="/notifications" element={<NotificationPreferences />} />
            <Route path="/webhooks" element={<WebhookTesting />} />
            <Route path="/bank-accounts" element={<BankAccounts />} />
            <Route path="/developers" element={<Developers />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/onboarding" element={<Onboarding />} />
            <Route path="*" element={<Navigate to="/" />} />
            </Routes>
          </Suspense>
        </ErrorBoundary>
      </main>
    </div>
  );
}

function RouteLoader() {
  return (
    <div className="card route-loader" style={{ minHeight: 260 }}>
      <span className="fp-loader large"></span>
      <strong>Loading FeelPay workspace</strong>
      <p>Preparing secure merchant controls...</p>
    </div>
  );
}

function ProtectedLayout() {
  const token = localStorage.getItem('feelpay_token');
  if (!token) {
    return <Navigate to="/login" replace />;
  }
  if (isSessionExpired()) {
    clearMerchantSession();
    return <Navigate to="/login?reason=expired" replace />;
  }
  return <Layout />;
}

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<RouteLoader />}>
        <Routes>
          <Route path="/landing" element={<Landing />} />
          <Route path="/login" element={<Login />} />
          <Route path="/signup" element={<Signup />} />
          <Route path="/forgot-password" element={<ForgotPassword />} />
          <Route path="/*" element={<ProtectedLayout />} />
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
