import { BrowserRouter, Routes, Route, NavLink, Navigate, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Users,
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
} from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';
import { apiRequest, clearToken, getToken, setToken } from './hooks/useApi';
import AdminDashboard from './pages/AdminDashboard';
import Merchants from './pages/Merchants';
import AdminBulkExport from './pages/AdminBulkExport';
import Transactions from './pages/Transactions';
import Settlements from './pages/Settlements';
import Disputes from './pages/Disputes';
import Revenue from './pages/Revenue';
import AdminOperations from './pages/AdminOperations';

const NAV = [
  { path: '/', label: 'Overview', icon: LayoutDashboard },
  { path: '/operations', label: 'Operations', icon: ListChecks },
  { path: '/merchants', label: 'Merchants', icon: Users },
  { path: '/transactions', label: 'Transactions', icon: Receipt },
  { path: '/settlements', label: 'Settlements', icon: Landmark },
  { path: '/disputes', label: 'Disputes', icon: Scale },
  { path: '/revenue', label: 'Revenue', icon: IndianRupee },
  { path: '/bulk-export', label: 'Bulk Export', icon: Download },
  { path: '/settings', label: 'Platform Settings', icon: SettingsIcon },
];

function Sidebar({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <div className="logo-icon">FP</div>
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
            <n.icon size={18} />
            <span>{n.label}</span>
          </NavLink>
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
              placeholder="Search payments or merchants"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              aria-label="Search transactions, merchants"
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
          <Route path="/operations" element={<AdminOperations />} />
          <Route path="/merchants" element={<Merchants />} />
          <Route path="/transactions" element={<Transactions />} />
          <Route path="/settlements" element={<Settlements />} />
          <Route path="/disputes" element={<Disputes />} />
          <Route path="/revenue" element={<Revenue />} />
          <Route path="/bulk-export" element={<AdminBulkExport />} />
          <Route path="/settings" element={<SettingsPage />} />
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
          <p>Use admin merchant email + password</p>
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

function SettingsPage() {
  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Platform Settings</h1>
          <p>Only verified modules enabled in this release train.</p>
        </div>
      </div>
      <div className="card">
        <h3 style={{ marginBottom: 10 }}>Production Notes</h3>
        <p style={{ color: 'var(--fp-text-muted)' }}>
          Admin surface now trimmed to fully functional DB-backed flows: overview, merchants, bulk export, impersonation, auth/session.
        </p>
      </div>
    </div>
  );
}

function ProtectedApp() {
  if (!getToken()) {
    return <Navigate to="/login" replace />;
  }
  return <Layout />;
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
