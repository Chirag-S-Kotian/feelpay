import { Suspense, lazy, useEffect, useRef, useState, type ComponentType } from 'react';
import { BrowserRouter, Routes, Route, NavLink, Navigate, useNavigate, Link } from 'react-router-dom';
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
  Repeat2,
  UserCircle2,
  Activity,
  ClipboardList,
  PanelLeftClose,
  PanelLeftOpen,
} from 'lucide-react';
import { ErrorBoundary } from './components/ErrorBoundary';
import {
  clearMerchantSession,
  EnvironmentMode,
  getEnvironmentMode,
  isSessionExpired,
  MERCHANT_SESSION_INVALID_EVENT,
  setEnvironmentMode,
} from './hooks/useApi';
import { apiRequest } from './hooks/useApi';

const Dashboard = lazy(() => import('./pages/Dashboard'));
const Payments = lazy(() => import('./pages/Payments'));
const Settlements = lazy(() => import('./pages/Settlements'));
const Refunds = lazy(() => import('./pages/Refunds'));
const Analytics = lazy(() => import('./pages/Analytics'));
const Reports = lazy(() => import('./pages/Reports'));
const Developers = lazy(() => import('./pages/Developers'));
const Disputes = lazy(() => import('./pages/Disputes'));
const Settings = lazy(() => import('./pages/Settings'));
const Team = lazy(() => import('./pages/Team'));
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
const AuditLogs = lazy(() => import('./pages/AuditLogs'));
const WebhookTesting = lazy(() => import('./pages/WebhookTesting'));
const BankAccounts = lazy(() => import('./pages/BankAccounts'));
const Pricing = lazy(() => import('./pages/Pricing'));
const Subscriptions = lazy(() => import('./pages/Subscriptions'));
const Mandates = lazy(() => import('./pages/Mandates'));
const VirtualAccounts = lazy(() => import('./pages/VirtualAccounts'));

const routePreloaders: Record<string, () => Promise<unknown>> = {
  '/': () => import('./pages/Dashboard'),
  '/payments': () => import('./pages/Payments'),
  '/payment-links': () => import('./pages/PaymentLinks'),
  '/payouts': () => import('./pages/Payouts'),
  '/payment-methods': () => import('./pages/PaymentMethods'),
  '/checkout-preview': () => import('./pages/CheckoutPreview'),
  '/subscriptions': () => import('./pages/Subscriptions'),
  '/autopay': () => import('./pages/Mandates'),
  '/mandates': () => import('./pages/Mandates'),
  '/virtual-accounts': () => import('./pages/VirtualAccounts'),
  '/customers': () => import('./pages/Customers'),
  '/settlements': () => import('./pages/Settlements'),
  '/refunds': () => import('./pages/Refunds'),
  '/reports': () => import('./pages/Reports'),
  '/exports': () => import('./pages/Exports'),
  '/settlement-reports': () => import('./pages/SettlementReports'),
  '/scheduled-reports': () => import('./pages/ScheduledReports'),
  '/analytics': () => import('./pages/Analytics'),
  '/disputes': () => import('./pages/Disputes'),
  '/notifications': () => import('./pages/NotificationPreferences'),
  '/audit-logs': () => import('./pages/AuditLogs'),
  '/webhooks': () => import('./pages/WebhookTesting'),
  '/bank-accounts': () => import('./pages/BankAccounts'),
  '/pricing': () => import('./pages/Pricing'),
  '/developers': () => import('./pages/Developers'),
  '/team': () => import('./pages/Team'),
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
      { path: '/refunds', label: 'Refunds', icon: Repeat2 },
      { path: '/reports', label: 'Reports', icon: BarChart3 },
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
      { path: '/subscriptions', label: 'Subscriptions', icon: Repeat2 },
      { path: '/autopay', label: 'AutoPay', icon: Activity },
      { path: '/virtual-accounts', label: 'Virtual Accounts', icon: Building2 },
      { path: '/webhooks', label: 'Webhooks', icon: Code2 },
    ],
  },
  {
    title: 'Customer Products',
    items: [
      { path: '/customers', label: 'Customers', icon: Users },
      { path: '/bank-accounts', label: 'Bank Accounts', icon: Wallet },
      { path: '/developers', label: 'Developers', icon: Code2 },
      { path: '/team', label: 'Team', icon: Users },
      { path: '/notifications', label: 'Notifications', icon: Bell },
      { path: '/audit-logs', label: 'Audit Logs', icon: ClipboardList },
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
  const [desktopSidebarCollapsed, setDesktopSidebarCollapsed] = useState(false);
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
  const [profileUploadState, setProfileUploadState] = useState('');
  const [profileForm, setProfileForm] = useState({
    name: '',
    business_name: '',
    business_type: 'individual',
    webhook_url: '',
    phone: '',
    avatar_url: '',
    current_password: '',
    new_password: '',
  });

  const parseBlockers = (warning: string) => {
    if (!warning.startsWith('Live mode blocked: ')) {
      return null;
    }
    const rawList = warning.replace('Live mode blocked: ', '').split(' | ');
    return rawList.map((item) => {
      let cleanText = item;
      let path = '';
      let linkLabel = '';
      
      if (item.toLowerCase().includes('onboarding') || item.toLowerCase().includes('review')) {
        cleanText = item.replace(/`in_progress`/g, 'In Progress')
                        .replace(/`under_review`/g, 'Under Review')
                        .replace(/require `approved`/g, 'requires Admin Approval')
                        .replace(/;/g, ' -');
        path = '/onboarding';
        linkLabel = 'Complete Onboarding';
      } else if (item.toLowerCase().includes('webhook')) {
        cleanText = 'Webhook URL is missing. You need to configure a webhook endpoint to receive real-time event updates.';
        path = '/webhooks';
        linkLabel = 'Configure Webhook';
      } else if (item.toLowerCase().includes('bank account')) {
        cleanText = 'No verified bank account linked. You must add and verify a bank account to receive payouts.';
        path = '/bank-accounts';
        linkLabel = 'Manage Bank Accounts';
      }
      
      return { text: cleanText, path, linkLabel };
    });
  };

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
        phone?: string | null;
        avatar_url?: string | null;
      }>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 1000 });
      setProfileForm({
        name: profile.name || '',
        business_name: profile.business_name || '',
        business_type: (profile.business_type || 'individual').toLowerCase(),
        webhook_url: profile.webhook_url || '',
        phone: profile.phone || '',
        avatar_url: profile.avatar_url || '',
        current_password: '',
        new_password: '',
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
      await apiRequest('PUT', '/v1/merchants/profile', {
        name: profileForm.name.trim() || null,
        business_name: profileForm.business_name.trim() || null,
        business_type: profileForm.business_type,
        webhook_url: profileForm.webhook_url.trim() || null,
        phone: profileForm.phone.trim() || null,
        avatar_url: profileForm.avatar_url.trim() || null,
        current_password: profileForm.current_password || null,
        new_password: profileForm.new_password || null,
      });
      const raw = localStorage.getItem('feelpay_merchant');
      if (raw) {
        const parsed = JSON.parse(raw) as Record<string, unknown>;
        localStorage.setItem('feelpay_merchant', JSON.stringify({ ...parsed, name: profileForm.name.trim() }));
      }
      setProfileModalOpen(false);
    } catch (err) {
      setProfileError(err instanceof Error ? err.message : 'Unable to update profile');
    } finally {
      setProfileSaving(false);
    }
  };

  const uploadAvatarFile = async (file: File) => {
    setProfileError('');
    setProfileUploadState('');
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type)) {
      setProfileError('Only PNG/JPEG/WEBP allowed');
      return;
    }
    if (file.size < 100 * 1024) {
      setProfileError('Avatar must be at least 100KB');
      return;
    }
    if (file.size > 5 * 1024 * 1024) {
      setProfileError('Avatar must be <= 5MB');
      return;
    }
    setProfileUploadState('Uploading...');
    const base64 = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const val = String(reader.result || '');
        const body = val.includes(',') ? (val.split(',')[1] || '') : val;
        resolve(body);
      };
      reader.onerror = () => reject(new Error('Failed to read file'));
      reader.readAsDataURL(file);
    });
    try {
      const res = await apiRequest<{ avatar_url?: string }>('POST', '/v1/merchants/profile/avatar-upload', {
        filename: file.name,
        content_type: file.type,
        data_base64: base64,
      });
      if (res.avatar_url) {
        setProfileForm((p) => ({ ...p, avatar_url: res.avatar_url || p.avatar_url }));
      }
      setProfileUploadState('Upload complete');
    } catch (err) {
      setProfileError(err instanceof Error ? err.message : 'Upload failed');
      setProfileUploadState('');
    }
  };

  return (
    <div className={`app-layout ${desktopSidebarCollapsed ? 'sidebar-collapsed' : ''}`}>
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
            (() => {
              const blockers = parseBlockers(modeWarning);
              if (!blockers) {
                return (
                  <div className="card" style={{ marginBottom: 12, borderColor: 'rgba(245,158,11,0.45)', color: '#f5b96b', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>{modeWarning}</span>
                    <button onClick={() => setModeWarning('')} style={{ background: 'none', border: 'none', color: '#f5b96b', cursor: 'pointer', fontSize: 18 }}>×</button>
                  </div>
                );
              }
              return (
                <div className="card" style={{ 
                  marginBottom: 16, 
                  borderLeft: '4px solid #ef4444', 
                  backgroundColor: '#fef2f2', 
                  color: '#991b1b',
                  padding: '16px 20px',
                  borderRadius: '12px',
                  boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
                }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: 12 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                      <span style={{ color: '#ef4444' }}>⚠️</span>
                      <strong style={{ fontSize: '16px', color: '#7f1d1d' }}>Live Mode Setup Incomplete</strong>
                    </div>
                    <button 
                      onClick={() => setModeWarning('')} 
                      aria-label="Dismiss warning"
                      style={{ 
                        background: 'none', 
                        border: 'none', 
                        color: '#991b1b', 
                        cursor: 'pointer', 
                        fontSize: '20px', 
                        lineHeight: 1,
                        padding: '0 4px'
                      }}
                    >
                      ×
                    </button>
                  </div>
                  <p style={{ margin: '0 0 12px 0', fontSize: '14px', color: '#b91c1c' }}>
                    To enable live transaction processing, please resolve the following requirements:
                  </p>
                  <div style={{ display: 'grid', gap: 10 }}>
                    {blockers.map((blocker, idx) => (
                      <div key={idx} style={{ 
                        display: 'flex', 
                        flexDirection: 'row',
                        justifyContent: 'space-between', 
                        alignItems: 'center', 
                        padding: '10px 14px', 
                        backgroundColor: '#fff', 
                        borderRadius: '8px',
                        border: '1px solid #fee2e2',
                        gap: 12,
                        flexWrap: 'wrap'
                      }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8, flex: 1, minWidth: '240px' }}>
                          <span style={{ color: '#f97316' }}>•</span>
                          <span style={{ fontSize: '13.5px', color: '#374151', lineHeight: 1.4 }}>{blocker.text}</span>
                        </div>
                        {blocker.path && (
                          <Link 
                            to={blocker.path} 
                            style={{ 
                              display: 'inline-flex', 
                              alignItems: 'center', 
                              gap: 4, 
                              fontSize: '13px', 
                              fontWeight: '600',
                              color: '#2563eb', 
                              textDecoration: 'none',
                              padding: '6px 12px',
                              backgroundColor: '#eff6ff',
                              borderRadius: '6px',
                              transition: 'all 0.2s'
                            }}
                            onMouseEnter={(e) => {
                              (e.currentTarget as HTMLElement).style.backgroundColor = '#dbeafe';
                            }}
                            onMouseLeave={(e) => {
                              (e.currentTarget as HTMLElement).style.backgroundColor = '#eff6ff';
                            }}
                          >
                            {blocker.linkLabel} &rarr;
                          </Link>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              );
            })()
          )}
          <header className="top-header">
            <button
              className="sidebar-toggle-btn"
              type="button"
              aria-label={desktopSidebarCollapsed ? 'Show sidebar' : 'Hide sidebar'}
              onClick={() => setDesktopSidebarCollapsed((v) => !v)}
            >
              {desktopSidebarCollapsed ? <PanelLeftOpen size={18} /> : <PanelLeftClose size={18} />}
            </button>
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
                            {Math.round(provider.success_rate || 0)}% success • {provider.avg_latency_ms || 0} ms • {provider.is_active === false ? 'Unavailable' : 'Available'} • {provider.circuit_breaker_state === 'open' ? 'Circuit open' : 'Circuit healthy'}
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
            <Route path="/subscriptions" element={<Subscriptions />} />
            <Route path="/autopay" element={<Mandates />} />
            <Route path="/mandates" element={<Mandates />} />
            <Route path="/virtual-accounts" element={<VirtualAccounts />} />
            <Route path="/customers" element={<Customers />} />
            <Route path="/settlements" element={<Settlements />} />
            <Route path="/refunds" element={<Refunds />} />
            <Route path="/reports" element={<Reports />} />
            <Route path="/exports" element={<Exports />} />
            <Route path="/settlement-reports" element={<SettlementReports />} />
            <Route path="/scheduled-reports" element={<ScheduledReports />} />
            <Route path="/analytics" element={<Analytics />} />
            <Route path="/disputes" element={<Disputes />} />
            <Route path="/notifications" element={<NotificationPreferences />} />
            <Route path="/audit-logs" element={<AuditLogs />} />
            <Route path="/webhooks" element={<WebhookTesting />} />
            <Route path="/bank-accounts" element={<BankAccounts />} />
            <Route path="/pricing" element={<Pricing />} />
            <Route path="/developers" element={<Developers />} />
            <Route path="/team" element={<Team />} />
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
              <button type="button" className="profile-btn-close" onClick={() => setProfileModalOpen(false)}>×</button>
            </div>
            <form
              className="profile-modal-body"
              style={{ display: 'grid', gap: 10 }}
              onSubmit={(e) => {
                e.preventDefault();
                void saveProfileFromHeader();
              }}
            >
              {profileError && <div className="card" style={{ color: 'var(--fp-danger)' }}>{profileError}</div>}
              <label className="profile-field">
                <span>Name</span>
                <input
                  type="text"
                  value={profileForm.name}
                  onChange={(e) => setProfileForm((p) => ({ ...p, name: e.target.value }))}
                  placeholder="Merchant owner name"
                />
              </label>
              <label className="profile-field">
                <span>Business Name</span>
                <input
                  type="text"
                  value={profileForm.business_name}
                  onChange={(e) => setProfileForm((p) => ({ ...p, business_name: e.target.value }))}
                  placeholder="Legal business name"
                />
              </label>
              <label className="profile-field">
                <span>Business Type</span>
                <select value={profileForm.business_type} onChange={(e) => setProfileForm((p) => ({ ...p, business_type: e.target.value }))}>
                  <option value="individual">Individual</option>
                  <option value="sole_proprietorship">Sole Proprietorship</option>
                  <option value="partnership">Partnership</option>
                  <option value="corporation">Corporation</option>
                  <option value="llc">LLC</option>
                </select>
              </label>
              <label className="profile-field">
                <span>Webhook URL</span>
                <input
                  type="url"
                  value={profileForm.webhook_url}
                  onChange={(e) => setProfileForm((p) => ({ ...p, webhook_url: e.target.value }))}
                  placeholder="https://merchant.example/webhooks/feelpay"
                />
              </label>
              <label className="profile-field">
                <span>Phone</span>
                <input
                  type="tel"
                  value={profileForm.phone}
                  onChange={(e) => setProfileForm((p) => ({ ...p, phone: e.target.value }))}
                  placeholder="+919876543210"
                />
              </label>
              <label className="profile-field">
                <span>Avatar URL</span>
                <input
                  type="url"
                  value={profileForm.avatar_url}
                  onChange={(e) => setProfileForm((p) => ({ ...p, avatar_url: e.target.value }))}
                  placeholder="https://cdn.example.com/avatar.png"
                />
              </label>
              <label className="profile-field">
                <span>Avatar Upload (Drag/Drop or Select)</span>
                <div
                  className="profile-dropzone"
                  onDragOver={(e) => e.preventDefault()}
                  onDrop={(e) => {
                    e.preventDefault();
                    const f = e.dataTransfer.files?.[0];
                    if (f) void uploadAvatarFile(f);
                  }}
                >
                  <input
                    type="file"
                    accept="image/png,image/jpeg,image/webp"
                    onChange={(e) => {
                      const f = e.target.files?.[0];
                      if (f) void uploadAvatarFile(f);
                    }}
                  />
                  <span>Drop image here or click to upload (100KB - 5MB)</span>
                </div>
                {profileUploadState && <small>{profileUploadState}</small>}
              </label>
              <label className="profile-field">
                <span>Current Password (optional)</span>
                <input
                  type="password"
                  value={profileForm.current_password}
                  onChange={(e) => setProfileForm((p) => ({ ...p, current_password: e.target.value }))}
                  placeholder="Required only for password change"
                />
              </label>
              <label className="profile-field">
                <span>New Password (optional)</span>
                <input
                  type="password"
                  value={profileForm.new_password}
                  onChange={(e) => setProfileForm((p) => ({ ...p, new_password: e.target.value }))}
                  placeholder="Minimum 8 characters"
                />
              </label>
              <button className="btn-primary" type="submit" disabled={profileSaving}>
                {profileSaving ? 'Saving...' : 'Save Profile'}
              </button>
            </form>
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
  const navigate = useNavigate();
  const [sessionValid, setSessionValid] = useState(() => Boolean(localStorage.getItem('feelpay_token')) && !isSessionExpired());

  useEffect(() => {
    const onInvalidSession = (event: Event) => {
      const reason = event instanceof CustomEvent && event.detail?.reason === 'expired' ? 'expired' : 'invalid';
      setSessionValid(false);
      navigate(`/login?reason=${reason}`, { replace: true });
    };
    const onStorage = (event: StorageEvent) => {
      if (event.key === 'feelpay_token' && !event.newValue) {
        setSessionValid(false);
        navigate('/login?reason=invalid', { replace: true });
      }
    };
    window.addEventListener(MERCHANT_SESSION_INVALID_EVENT, onInvalidSession);
    window.addEventListener('storage', onStorage);
    return () => {
      window.removeEventListener(MERCHANT_SESSION_INVALID_EVENT, onInvalidSession);
      window.removeEventListener('storage', onStorage);
    };
  }, [navigate]);

  const token = localStorage.getItem('feelpay_token');
  if (!token) {
    return <Navigate to="/login?reason=invalid" replace />;
  }
  if (isSessionExpired()) {
    clearMerchantSession();
    return <Navigate to="/login?reason=expired" replace />;
  }
  if (!sessionValid) {
    return <Navigate to="/login?reason=invalid" replace />;
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
