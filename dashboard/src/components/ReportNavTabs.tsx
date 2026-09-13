import { NavLink } from 'react-router-dom';
import { BarChart3, Download, FileText, Calendar } from 'lucide-react';

interface ReportNavTabsProps {
  className?: string;
}

export function ReportNavTabs({ className = '' }: ReportNavTabsProps) {
  const tabs = [
    { to: '/reports', label: 'Overview', icon: BarChart3, end: true },
    { to: '/exports', label: 'Transaction Exports', icon: Download, end: false },
    { to: '/settlement-reports', label: 'Settlement Reports', icon: FileText, end: false },
    { to: '/scheduled-reports', label: 'Scheduled Reports', icon: Calendar, end: false },
  ];

  return (
    <div className={`report-nav-tabs-container ${className}`} style={{ marginBottom: 20 }}>
      <nav className="pill-tabs" aria-label="Reports and Exports Navigation" style={{ display: 'inline-flex', flexWrap: 'wrap' }}>
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <NavLink
              key={tab.to}
              to={tab.to}
              end={tab.end}
              className={({ isActive }) => `pill-tab ${isActive ? 'active' : ''}`}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                textDecoration: 'none',
                fontWeight: 500,
              }}
            >
              <Icon size={15} aria-hidden="true" />
              <span>{tab.label}</span>
            </NavLink>
          );
        })}
      </nav>
    </div>
  );
}

export default ReportNavTabs;
