import { BrowserRouter, Routes, Route, NavLink, Navigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Users,
  Building2,
  ShieldAlert,
  Settings as SettingsIcon,
  PieChart,
  Wallet,
  Menu,
  X,
  FileCheck,
  Bell,
  Search,
} from 'lucide-react';
import { useState } from 'react';
import AdminDashboard from './pages/AdminDashboard';
import Merchants from './pages/Merchants';
import KYCQueue from './pages/KYCQueue';

const NAV = [
  { path: '/', label: 'Overview', icon: LayoutDashboard },
  { path: '/merchants', label: 'Merchants', icon: Users },
  { path: '/kyc-queue', label: 'KYC Queue', icon: FileCheck },
  { path: '/settlements', label: 'Global Settlements', icon: Wallet },
  { path: '/disputes', label: 'Global Disputes', icon: ShieldAlert },
  { path: '/system', label: 'System Health', icon: PieChart },
  { path: '/settings', label: 'Platform Settings', icon: SettingsIcon },
];

function Sidebar({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <div className="logo-icon">A</div>
        <span className="logo-text">FeelPay Admin</span>
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
            {n.label}
          </NavLink>
        ))}
      </nav>
      <div className="env-panel">
        <div className="env-label">Platform Mode</div>
        <div className="env-badge">
          <span className="env-dot"></span>
          Super Admin
        </div>
      </div>
    </aside>
  );
}

function Layout() {
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [notificationCount] = useState(2); // TODO: fetch from API

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
          <div className="global-search">
            <Search size={16} />
            <input
              type="text"
              placeholder="Search merchants, transactions, users..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
            />
          </div>
          <button className="notification-bell" aria-label="Notifications">
            <Bell size={20} />
            {notificationCount > 0 && (
              <span className="notification-badge">{notificationCount}</span>
            )}
          </button>
        </header>
        <Routes>
          <Route path="/" element={<AdminDashboard />} />
          <Route path="/merchants" element={<Merchants />} />
          <Route path="/kyc-queue" element={<KYCQueue />} />
          <Route path="*" element={<Navigate to="/" />} />
        </Routes>
      </main>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Layout />
    </BrowserRouter>
  );
}
