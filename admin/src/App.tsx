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
  Download,
  Webhook,
  KeyRound,
  ClipboardList,
} from 'lucide-react';
import { useState } from 'react';
import AdminDashboard from './pages/AdminDashboard';
import Merchants from './pages/Merchants';
import KYCQueue from './pages/KYCQueue';
import AdminBulkExport from './pages/AdminBulkExport';
import AdminWebhookTesting from './pages/AdminWebhookTesting';
import NotificationDeliveryLogs from './pages/NotificationDeliveryLogs';
import NotificationTemplateEditor from './pages/NotificationTemplateEditor';
import KycAuditLog from './pages/KycAuditLog';
import TransactionSearch from './pages/TransactionSearch';
import TransactionDetail from './pages/TransactionDetail';
import SettlementManagement from './pages/SettlementManagement';
import RevenueDashboard from './pages/RevenueDashboard';
import CredentialManagement from './pages/CredentialManagement';
import TenantManagement from './pages/TenantManagement';
import AdminUserManagement from './pages/AdminUserManagement';
import AuditLogViewer from './pages/AuditLogViewer';
import SystemConfig from './pages/SystemConfig';

const NAV = [
  { path: '/', label: 'Overview', icon: LayoutDashboard },
  { path: '/merchants', label: 'Merchants', icon: Users },
  { path: '/kyc-queue', label: 'KYC Queue', icon: FileCheck },
  { path: '/bulk-export', label: 'Bulk Export', icon: Download },
  { path: '/webhook-testing', label: 'Webhook Testing', icon: Webhook },
  { path: '/notification-logs', label: 'Notification Logs', icon: Bell },
  { path: '/notification-templates', label: 'Notif Templates', icon: Bell },
  { path: '/kyc-audit', label: 'KYC Audit', icon: ClipboardList },
  { path: '/transactions', label: 'Transactions', icon: Search },
  { path: '/transaction-actions', label: 'Txn Actions', icon: ShieldAlert },
  { path: '/settlements', label: 'Global Settlements', icon: Wallet },
  { path: '/revenue', label: 'Revenue', icon: PieChart },
  { path: '/credentials', label: 'Credentials', icon: KeyRound },
  { path: '/tenants', label: 'Tenants', icon: Building2 },
  { path: '/admin-users', label: 'Admin Users', icon: Users },
  { path: '/audit-logs', label: 'Audit Logs', icon: ClipboardList },
  { path: '/disputes', label: 'Global Disputes', icon: ShieldAlert },
  { path: '/system', label: 'System Health', icon: PieChart },
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
              placeholder="Search..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              aria-label="Search transactions, merchants"
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
          <Route path="/bulk-export" element={<AdminBulkExport />} />
          <Route path="/webhook-testing" element={<AdminWebhookTesting />} />
          <Route path="/notification-logs" element={<NotificationDeliveryLogs />} />
          <Route path="/notification-templates" element={<NotificationTemplateEditor />} />
          <Route path="/kyc-audit" element={<KycAuditLog />} />
          <Route path="/transactions" element={<TransactionSearch />} />
          <Route path="/transaction-actions" element={<TransactionDetail />} />
          <Route path="/settlements" element={<SettlementManagement />} />
          <Route path="/revenue" element={<RevenueDashboard />} />
          <Route path="/credentials" element={<CredentialManagement />} />
          <Route path="/tenants" element={<TenantManagement />} />
          <Route path="/admin-users" element={<AdminUserManagement />} />
          <Route path="/audit-logs" element={<AuditLogViewer />} />
          <Route path="/settings" element={<SystemConfig />} />
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
