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
} from 'lucide-react';
import { ErrorBoundary } from './components/ErrorBoundary';
import { EnvironmentMode, getEnvironmentMode, setEnvironmentMode } from './hooks/useApi';

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
const PaymentLinks = lazy(() => import('./pages/PaymentLinks'));
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
  onModeChange: (mode: EnvironmentMode) => void;
}) {
  const preloadRoute = (path: string) => {
    routePreloaders[path]?.();
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <div className="logo-icon">F</div>
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
            onClick={() => onModeChange('test')}
            type="button"
          >
            Test
          </button>
          <button
            className={`pill-tab ${mode === 'live' ? 'active' : ''}`}
            onClick={() => onModeChange('live')}
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

  useEffect(() => {
    setEnvironmentMode(mode);
  }, [mode]);

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
        <Sidebar onNavigate={() => setOpen(false)} mode={mode} onModeChange={setMode} />
      </div>
      <div className="desktop-sidebar">
        <Sidebar mode={mode} onModeChange={setMode} />
      </div>
      <main className="main-content">
        <ErrorBoundary>
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
          </header>
          <Suspense fallback={<RouteLoader />}>
            <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/payments" element={<Payments />} />
            <Route path="/payment-links" element={<PaymentLinks />} />
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
    <div className="card" style={{ minHeight: 260 }}>
      <div className="skeleton" style={{ height: 24, width: 220, marginBottom: 16 }}></div>
      <div className="skeleton" style={{ height: 46, width: '100%', marginBottom: 12 }}></div>
      <div className="skeleton" style={{ height: 46, width: '100%', marginBottom: 12 }}></div>
      <div className="skeleton" style={{ height: 180, width: '100%' }}></div>
    </div>
  );
}

function ProtectedLayout() {
  const token = localStorage.getItem('feelpay_token');
  if (!token) {
    return <Navigate to="/login" replace />;
  }
  return <Layout />;
}

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<RouteLoader />}>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/signup" element={<Signup />} />
          <Route path="/forgot-password" element={<ForgotPassword />} />
          <Route path="/*" element={<ProtectedLayout />} />
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
