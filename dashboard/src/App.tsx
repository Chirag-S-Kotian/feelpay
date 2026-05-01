import { Suspense, lazy, useEffect, useRef, useState, type ComponentType } from 'react';
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
  Layers3,
  UserCircle2,
  Activity,
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
const Pricing = lazy(() => import('./pages/Pricing'));

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
  '/pricing': () => import('./pages/Pricing'),
  '/developers': () => import('./pages/Developers'),
  '/settings': () => import('./pages/Settings'),
  '/onboarding': () => import('./pages/Onboarding'),
  '/login': () => import('./pages/Login'),
  '/signup': () => import('./pages/Signup'),
  '/forgot-password': () => import('./pages/ForgotPassword'),
};

type NavItem = { path: string; label: string; icon: ComponentType<{ size?: number; 'aria-hidden'?: boolean }> };
type NavSection = { title: string; items: NavItem[] };

const NAV_SECTIONS: NavSection[] = [
  {
    title: 'Core',
    items: [
      { path: '/', label: 'Dashboard', icon: LayoutDashboard },
      { path: '/payments', label: 'Payments', icon: CreditCard },
      { path: '/settlements', label: 'Settlements', icon: Wallet },
      { path: '/exports', label: 'Exports', icon: BarChart3 },
      { path: '/settlement-reports', label: 'Settlement Reports', icon: BarChart3 },
      { path: '/scheduled-reports', label: 'Scheduled Reports', icon: BarChart3 },
      { path: '/analytics', label: 'Analytics', icon: BarChart3 },
      { path: '/disputes', label: 'Disputes', icon: ShieldAlert },
    ],
  },
  {
    title: 'Payment Products',
    items: [
      { path: '/payment-links', label: 'Payment Links', icon: LinkIcon },
      { path: '/payouts', label: 'Payouts', icon: Send },
      { path: '/payment-methods', label: 'Payment Methods', icon: SlidersHorizontal },
      { path: '/checkout-preview', label: 'Checkout Preview', icon: Smartphone },
      { path: '/webhooks', label: 'Webhooks', icon: Code2 },
    ],
  },
  {
    title: 'Customer Products',
    items: [
      { path: '/customers', label: 'Customers', icon: Users },
      { path: '/bank-accounts', label: 'Bank Accounts', icon: Wallet },
      { path: '/developers', label: 'Developers', icon: Code2 },
      { path: '/notifications', label: 'Notifications', icon: Bell },
    ],
  },
  {
    title: 'Account & Settings',
    items: [
      { path: '/pricing', label: 'Pricing', icon: Layers3 },
      { path: '/settings', label: 'Settings', icon: SettingsIcon },
      { path: '/onboarding', label: 'Onboarding', icon: Building2 },
    ],
  },
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
        {NAV_SECTIONS.map((section) => (
          <div key={section.title} className="sidebar-nav-section">
            <div className="sidebar-nav-title">{section.title}</div>
            {section.items.map(n => (
              <NavLink
                key={n.path}
                to={n.path}
                end={n.path === '/'}
                onClick={onNavigate}
                onMouseEnter={() => preloadRoute(n.path)}
                onFocus={() => preloadRoute(n.path)}
                className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}
              >
                <n.icon size={18} aria-hidden={true} />
                {n.label}
              </NavLink>
            ))}
          </div>
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
  const [headerSignals, setHeaderSignals] = useState<{
    announcements: Array<{ id?: string; title?: string; message?: string; severity?: 'info' | 'warning' | 'error' }>;
    ecosystem_health: Array<{ name?: string; success_rate?: number; avg_latency_ms?: number; is_active?: boolean; circuit_breaker_state?: string }>;
  }>({ announcements: [], ecosystem_health: [] });
  const [announcementsOpen, setAnnouncementsOpen] = useState(false);
  const [healthOpen, setHealthOpen] = useState(false);
  const announcementsRef = useRef<HTMLDivElement | null>(null);
  const healthRef = useRef<HTMLDivElement | null>(null);
  const [mode, setMode] = useState<EnvironmentMode>(getEnvironmentMode());
  const [modeWarning, setModeWarning] = useState('');
  const [profileModalOpen, setProfileModalOpen] = useState(false);
  const [profileSaving, setProfileSaving] = useState(false);
  const [profileError, setProfileError] = useState('');
  const [profileForm, setProfileForm] = useState({
    name: '',
    business_name: '',
    business_type: 'individual',
    webhook_url: '',
  });

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

  useEffect(() => {
    const loadSignals = async () => {
      const data = await apiRequest<{
        announcements?: Array<{ id?: string; title?: string; message?: string; severity?: 'info' | 'warning' | 'error' }>;
        ecosystem_health?: Array<{ name?: string; success_rate?: number; avg_latency_ms?: number; is_active?: boolean; circuit_breaker_state?: string }>;
      }>('GET', '/v1/merchants/dashboard-signals', undefined, undefined, { cacheMs: 20_000 }).catch(
        () =>
          ({
            announcements: [],
            ecosystem_health: [],
          }) as {
            announcements: Array<{ id?: string; title?: string; message?: string; severity?: 'info' | 'warning' | 'error' }>;
            ecosystem_health: Array<{ name?: string; success_rate?: number; avg_latency_ms?: number; is_active?: boolean; circuit_breaker_state?: string }>;
          },
      );
      setHeaderSignals({
        announcements: data.announcements || [],
        ecosystem_health: data.ecosystem_health || [],
      });
    };
    void loadSignals();
  }, []);

  useEffect(() => {
    const onDocClick = (event: MouseEvent) => {
      const target = event.target as Node | null;
      if (announcementsRef.current && target && !announcementsRef.current.contains(target)) {
        setAnnouncementsOpen(false);
      }
      if (healthRef.current && target && !healthRef.current.contains(target)) {
        setHealthOpen(false);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, []);

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

  const loadProfileForEdit = async () => {
    setProfileError('');
    try {
      const profile = await apiRequest<{
        name?: string;
        business_name?: string | null;
        business_type?: string | null;
        webhook_url?: string | null;
      }>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 1000 });
      setProfileForm({
        name: profile.name || '',
        business_name: profile.business_name || '',
        business_type: (profile.business_type || 'individual').toLowerCase(),
        webhook_url: profile.webhook_url || '',
      });
    } catch (err) {
      setProfileError(err instanceof Error ? err.message : 'Unable to load profile');
    }
  };

  const openProfileModal = async () => {
    setProfileModalOpen(true);
    await loadProfileForEdit();
  };

  const saveProfileFromHeader = async () => {
    setProfileSaving(true);
    setProfileError('');
    try {
      await apiRequest('PUT', '/v1/merchants/profile', profileForm);
      const raw = localStorage.getItem('feelpay_merchant');
      if (raw) {
        const parsed = JSON.parse(raw) as Record<string, unknown>;
        localStorage.setItem('feelpay_merchant', JSON.stringify({ ...parsed, name: profileForm.name }));
      }
      setProfileModalOpen(false);
    } catch (err) {
      setProfileError(err instanceof Error ? err.message : 'Unable to update profile');
    } finally {
      setProfileSaving(false);
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
            <div className="header-dropdown-wrap" ref={announcementsRef}>
              <button
                className="notification-bell"
                aria-label="Admin announcements"
                onClick={() => {
                  setAnnouncementsOpen((v) => !v);
                  setHealthOpen(false);
                }}
                type="button"
              >
                <Bell size={20} />
                {(headerSignals.announcements.length || notificationCount) > 0 && (
                  <span className="notification-badge">{headerSignals.announcements.length || notificationCount}</span>
                )}
              </button>
              {announcementsOpen && (
                <div className="header-dropdown">
                  <div className="header-dropdown-title">Admin Announcements</div>
                  {headerSignals.announcements.length ? (
                    <div className="header-dropdown-list">
                      {headerSignals.announcements.map((note, idx) => (
                        <div key={note.id || String(idx)} className="header-dropdown-item">
                          <strong>{note.title || 'Update'}</strong>
                          <p>{note.message || 'No message'}</p>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="header-dropdown-empty">No announcements</div>
                  )}
                </div>
              )}
            </div>
            <div className="header-dropdown-wrap" ref={healthRef}>
              <button
                className="notification-bell"
                aria-label="Bank and service health"
                onClick={() => {
                  setHealthOpen((v) => !v);
                  setAnnouncementsOpen(false);
                }}
                type="button"
              >
                <Activity size={20} />
              </button>
              {healthOpen && (
                <div className="header-dropdown">
                  <div className="header-dropdown-title">Bank / Service Health</div>
                  {headerSignals.ecosystem_health.length ? (
                    <div className="header-dropdown-list">
                      {headerSignals.ecosystem_health.map((provider, idx) => (
                        <div key={`${provider.name || 'provider'}-${idx}`} className="header-dropdown-item">
                          <strong>{provider.name || 'Provider'}</strong>
                          <p>
                            {Math.round(provider.success_rate || 0)}% success • {provider.avg_latency_ms || 0} ms • {provider.circuit_breaker_state || 'closed'}
                          </p>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="header-dropdown-empty">No health data</div>
                  )}
                </div>
              )}
            </div>
            <button className="logout-button" onClick={() => void handleLogout()} type="button">
              <LogOut size={18} />
              Logout
            </button>
            <button className="profile-icon-btn" onClick={() => void openProfileModal()} type="button" aria-label="Edit profile">
              <UserCircle2 size={20} />
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
            <Route path="/pricing" element={<Pricing />} />
            <Route path="/developers" element={<Developers />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/onboarding" element={<Onboarding />} />
            <Route path="*" element={<Navigate to="/" />} />
            </Routes>
          </Suspense>
        </ErrorBoundary>
      </main>
      {profileModalOpen && (
        <div className="profile-modal-overlay" onClick={() => setProfileModalOpen(false)}>
          <div className="profile-modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="profile-modal-header">
              <h2>Edit Merchant Profile</h2>
              <button type="button" className="btn-close" onClick={() => setProfileModalOpen(false)}>×</button>
            </div>
            <div className="profile-modal-body" style={{ display: 'grid', gap: 10 }}>
              {profileError && <div className="card" style={{ color: 'var(--fp-danger)' }}>{profileError}</div>}
              <label>
                Name
                <input value={profileForm.name} onChange={(e) => setProfileForm((p) => ({ ...p, name: e.target.value }))} />
              </label>
              <label>
                Business Name
                <input value={profileForm.business_name} onChange={(e) => setProfileForm((p) => ({ ...p, business_name: e.target.value }))} />
              </label>
              <label>
                Business Type
                <select value={profileForm.business_type} onChange={(e) => setProfileForm((p) => ({ ...p, business_type: e.target.value }))}>
                  <option value="individual">Individual</option>
                  <option value="sole_proprietorship">Sole Proprietorship</option>
                  <option value="partnership">Partnership</option>
                  <option value="corporation">Corporation</option>
                  <option value="llc">LLC</option>
                </select>
              </label>
              <label>
                Webhook URL
                <input value={profileForm.webhook_url} onChange={(e) => setProfileForm((p) => ({ ...p, webhook_url: e.target.value }))} />
              </label>
              <button className="btn-primary" type="button" onClick={() => void saveProfileFromHeader()} disabled={profileSaving}>
                {profileSaving ? 'Saving...' : 'Save Profile'}
              </button>
            </div>
          </div>
        </div>
      )}
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
