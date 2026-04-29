import { useEffect, useState } from 'react';
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
import Dashboard from './pages/Dashboard';
import Payments from './pages/Payments';
import Settlements from './pages/Settlements';
import Analytics from './pages/Analytics';
import Developers from './pages/Developers';
import Disputes from './pages/Disputes';
import Settings from './pages/Settings';
import Onboarding from './pages/Onboarding';
import Login from './pages/Login';
import Signup from './pages/Signup';
import ForgotPassword from './pages/ForgotPassword';
import PaymentLinks from './pages/PaymentLinks';
import Customers from './pages/Customers';
import Exports from './pages/Exports';
import SettlementReports from './pages/SettlementReports';
import ScheduledReports from './pages/ScheduledReports';
import NotificationPreferences from './pages/NotificationPreferences';
import WebhookTesting from './pages/WebhookTesting';
import BankAccounts from './pages/BankAccounts';
import { ErrorBoundary } from './components/ErrorBoundary';
import { EnvironmentMode, getEnvironmentMode, setEnvironmentMode } from './hooks/useApi';

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
        </ErrorBoundary>
      </main>
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
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/signup" element={<Signup />} />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/*" element={<ProtectedLayout />} />
      </Routes>
    </BrowserRouter>
  );
}
