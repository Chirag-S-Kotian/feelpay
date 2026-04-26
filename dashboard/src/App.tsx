import { useState } from 'react';
import { BrowserRouter, Routes, Route, NavLink, Navigate } from 'react-router-dom';
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

const NAV = [
  { path: '/', label: 'Dashboard', icon: LayoutDashboard },
  { path: '/payments', label: 'Payments', icon: CreditCard },
  { path: '/settlements', label: 'Settlements', icon: Wallet },
  { path: '/analytics', label: 'Analytics', icon: BarChart3 },
  { path: '/disputes', label: 'Disputes', icon: ShieldAlert },
  { path: '/developers', label: 'Developers', icon: Code2 },
  { path: '/settings', label: 'Settings', icon: SettingsIcon },
  { path: '/onboarding', label: 'Onboarding', icon: Building2 },
];

function Sidebar({ onNavigate }: { onNavigate?: () => void }) {
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
        <div className="env-badge">
          <span className="env-dot"></span>
          Test Mode
        </div>
      </div>
    </aside>
  );
}

function Layout() {
  const [open, setOpen] = useState(false);

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
        <Sidebar onNavigate={() => setOpen(false)} />
      </div>
      <div className="desktop-sidebar">
        <Sidebar />
      </div>
      <main className="main-content">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/payments" element={<Payments />} />
          <Route path="/settlements" element={<Settlements />} />
          <Route path="/analytics" element={<Analytics />} />
          <Route path="/disputes" element={<Disputes />} />
          <Route path="/developers" element={<Developers />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/onboarding" element={<Onboarding />} />
          <Route path="*" element={<Navigate to="/" />} />
        </Routes>
      </main>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/*" element={<Layout />} />
      </Routes>
    </BrowserRouter>
  );
}
