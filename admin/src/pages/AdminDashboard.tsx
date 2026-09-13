import { useEffect, useMemo, useState } from 'react';
import { Activity, ExternalLink, TrendingUp, Building2, AlertTriangle } from 'lucide-react';
import { apiRequest, clearToken } from '../hooks/useApi';

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

function getOnboardingBadgeClass(status?: string | null) {
  const s = String(status || '').toLowerCase();
  if (['completed', 'verified', 'active', 'enabled'].includes(s)) return 'badge-completed';
  if (['pending', 'in_progress', 'review', 'submitted'].includes(s)) return 'badge-pending';
  if (['rejected', 'failed', 'suspended', 'disabled'].includes(s)) return 'badge-failed';
  return 'badge-pending';
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
    return <div className="page fade-in">Loading platform data...</div>;
  }

  if (error) {
    return (
      <div className="page fade-in">
        <div className="stat-card warning" style={{ maxWidth: '420px', margin: '100px auto', textAlign: 'center' }}>
          <h3>Access / Service Issue</h3>
          <p>{error}</p>
          <button
            className="btn-primary"
            style={{ marginTop: '16px', display: 'inline-block' }}
            onClick={() => {
              clearToken();
              window.location.href = '/login';
            }}
          >
            Sign In with Admin Account
          </button>
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
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '8px',
            padding: '8px 16px',
            borderRadius: '20px',
            background: 'var(--fp-bg-card)',
            border: '1px solid var(--fp-border)',
            fontSize: '13px',
            fontWeight: 500,
          }}
        >
          <span
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: stats?.system_health === 'healthy' ? 'var(--spotify-tertiary)' : 'var(--fp-warning)',
              display: 'inline-block',
            }}
          />
          <Activity size={14} style={{ color: 'var(--fp-text-muted)' }} />
          <span>System: <strong style={{ textTransform: 'capitalize', color: 'var(--fp-text)' }}>{stats?.system_health || 'Healthy'}</strong></span>
        </div>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">
            <TrendingUp size={14} /> Captured Volume
          </div>
          <div className="stat-value">₹{(stats?.total_volume || 0).toLocaleString('en-IN')}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">
            <Building2 size={14} /> Active Business Accounts
          </div>
          <div className="stat-value">{(stats?.active_merchants || 0).toLocaleString('en-IN')}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">
            <AlertTriangle size={14} /> Failed Payments
          </div>
          <div className="stat-value warning">{(stats?.failed_payments || 0).toLocaleString('en-IN')}</div>
        </div>
      </div>

      <div className="table-card" style={{ marginTop: '24px' }}>
        <div className="page-header" style={{ marginBottom: '16px' }}>
          <div>
            <h3 style={{ margin: 0 }}>Recent Business Accounts</h3>
            <span style={{ fontSize: '13px', color: 'var(--fp-text-muted)' }}>Latest registered partner merchants and platforms</span>
          </div>
        </div>
        <div className="table-scroll">
          <table>
            <thead>
              <tr>
                <th>Business Account</th>
                <th>Account Email</th>
                <th>Business Type</th>
                <th>Status</th>
                <th>Date Joined</th>
                <th className="actions-cell">Actions</th>
              </tr>
            </thead>
            <tbody>
              {recentBusinessAccounts.map(m => (
                <tr key={m.id}>
                  <td>
                    <div style={{ fontWeight: 600, color: 'var(--fp-text)' }}>{m.business_name || m.name}</div>
                    {m.business_name && m.name && m.business_name !== m.name && (
                      <div style={{ fontSize: '12px', color: 'var(--fp-text-muted)' }}>{m.name}</div>
                    )}
                  </td>
                  <td>{m.email || 'N/A'}</td>
                  <td>
                    <span className="badge" style={{ background: 'var(--fp-bg-elevated)', border: '1px solid var(--fp-border)' }}>
                      {m.business_type || 'Standard'}
                    </span>
                  </td>
                  <td>
                    <span className={`badge ${getOnboardingBadgeClass(m.onboarding_status)}`}>
                      {m.onboarding_status ? m.onboarding_status.replace('_', ' ') : 'Pending'}
                    </span>
                  </td>
                  <td title={m.created_at || undefined}>{formatDateTime(m.created_at)}</td>
                  <td className="actions-cell">
                    <button
                      type="button"
                      className="btn-action-primary"
                      onClick={() => handleOpenAccount(m.id)}
                      disabled={impersonating === m.id}
                      title="Open Business Dashboard"
                    >
                      <ExternalLink size={12} />
                      <span>{impersonating === m.id ? 'Opening...' : 'Open'}</span>
                    </button>
                  </td>
                </tr>
              ))}
              {recentBusinessAccounts.length === 0 && (
                <tr>
                  <td colSpan={6} style={{ textAlign: 'center', padding: '32px', color: 'var(--fp-text-muted)' }}>
                    No business accounts found
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
