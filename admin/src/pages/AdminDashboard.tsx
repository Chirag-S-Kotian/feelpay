import { useEffect, useMemo, useState } from 'react';
import { Activity, ExternalLink } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

const DASHBOARD_BASE_URL = import.meta.env.VITE_DASHBOARD_BASE_URL || window.location.origin;

export interface Merchant {
  id: string;
  name: string;
  email?: string;
  business_name?: string | null;
  business_type?: string | null;
  onboarding_status?: string | null;
  created_at?: string;
}

export interface PlatformStats {
  total_volume: number;
  active_merchants: number;
  failed_payments: number;
  system_health: string;
}

interface BusinessAccountsResponse {
  merchants?: Merchant[];
  data?: Merchant[];
}

function formatDateTime(value?: string) {
  if (!value) return 'N/A';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('en-GB', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  }).format(date).replace(',', '');
}

export default function AdminDashboard() {
  const [stats, setStats] = useState<PlatformStats | null>(null);
  const [businessAccounts, setBusinessAccounts] = useState<Merchant[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [impersonating, setImpersonating] = useState<string | null>(null);

  const handleOpenAccount = async (businessAccountId: string) => {
    setImpersonating(businessAccountId);
    try {
      const res = await apiRequest<{ token: string }>('POST', `/admin/merchants/${businessAccountId}/impersonate`);
      const dashboardUrl = new URL('/login', DASHBOARD_BASE_URL);
      dashboardUrl.searchParams.set('token', res.token);
      window.open(dashboardUrl.toString(), '_blank');
    } catch (err) {
      console.error('Failed to open business account', err);
    } finally {
      setImpersonating(null);
    }
  };

  useEffect(() => {
    async function fetchData() {
      try {
        const [statsData, accountsData] = await Promise.all([
          apiRequest<PlatformStats>('GET', '/admin/stats'),
          apiRequest<Merchant[] | BusinessAccountsResponse>('GET', '/admin/merchants'),
        ]);
        setStats(statsData);
        setBusinessAccounts(Array.isArray(accountsData) ? accountsData : accountsData.data || accountsData.merchants || []);
      } catch (err) {
        console.error('Failed to fetch admin data:', err);
        setError('Could not load admin data from server.');
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, []);

  const recentBusinessAccounts = useMemo(() => businessAccounts.slice(0, 8), [businessAccounts]);

  if (loading) {
    return <div className="page">Loading platform data...</div>;
  }

  if (error) {
    return (
      <div className="page fade-in">
        <div className="stat-card warning" style={{ maxWidth: '420px', margin: '100px auto', textAlign: 'center' }}>
          <h3>Service Unavailable</h3>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Global Overview</h1>
          <p>Live admin metrics from production tables</p>
        </div>
        <div className="btn-secondary">
          <Activity size={16} />
          System: {stats?.system_health || 'Unknown'}
        </div>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Captured Volume</div>
          <div className="stat-value">₹{(stats?.total_volume || 0).toLocaleString('en-IN')}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Active Business Accounts</div>
          <div className="stat-value">{(stats?.active_merchants || 0).toLocaleString('en-IN')}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Failed Payments</div>
          <div className="stat-value warning">{(stats?.failed_payments || 0).toLocaleString('en-IN')}</div>
        </div>
      </div>

      <div className="table-card" style={{ marginTop: '20px' }}>
        <div className="page-header" style={{ marginBottom: '15px' }}>
          <h3>Recent Business Accounts</h3>
        </div>
        <table>
          <thead>
            <tr>
              <th>Business Account</th>
              <th>Business Account Email</th>
              <th>Business Type</th>
              <th>Status</th>
              <th>Date Joined</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {recentBusinessAccounts.map(m => (
              <tr key={m.id}>
                <td>{m.business_name || m.name}</td>
                <td>{m.email || 'N/A'}</td>
                <td>{m.business_type || 'N/A'}</td>
                <td>{m.onboarding_status || 'N/A'}</td>
                <td title={m.created_at || undefined}>{formatDateTime(m.created_at)}</td>
                <td>
                  <button
                    className="btn-primary"
                    style={{ padding: '4px 10px', fontSize: '12px', display: 'flex', alignItems: 'center', gap: '4px' }}
                    onClick={() => handleOpenAccount(m.id)}
                    disabled={impersonating === m.id}
                  >
                    <ExternalLink size={12} />
                    {impersonating === m.id ? 'Opening...' : 'Open'}
                  </button>
                </td>
              </tr>
            ))}
            {recentBusinessAccounts.length === 0 && (
              <tr>
                <td colSpan={6} style={{ textAlign: 'center', padding: '20px' }}>No business accounts found</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
