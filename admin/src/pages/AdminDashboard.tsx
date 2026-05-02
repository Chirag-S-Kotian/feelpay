import { useEffect, useMemo, useState } from 'react';
import { Activity, ExternalLink } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

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

interface MerchantsResponse {
  merchants: Merchant[];
}

export default function AdminDashboard() {
  const [stats, setStats] = useState<PlatformStats | null>(null);
  const [merchants, setMerchants] = useState<Merchant[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [impersonating, setImpersonating] = useState<string | null>(null);

  const handleImpersonate = async (merchantId: string) => {
    setImpersonating(merchantId);
    try {
      const res = await apiRequest<{ token: string }>('POST', `/admin/merchants/${merchantId}/impersonate`);
      window.open(`http://localhost:5173/login?token=${res.token}`, '_blank');
    } catch (err) {
      console.error('Failed to impersonate merchant', err);
    } finally {
      setImpersonating(null);
    }
  };

  useEffect(() => {
    async function fetchData() {
      try {
        const [statsData, merchantsData] = await Promise.all([
          apiRequest<PlatformStats>('GET', '/admin/stats'),
          apiRequest<Merchant[] | MerchantsResponse>('GET', '/admin/merchants'),
        ]);
        setStats(statsData);
        setMerchants(Array.isArray(merchantsData) ? merchantsData : merchantsData.merchants || []);
      } catch (err) {
        console.error('Failed to fetch admin data:', err);
        setError('Could not load admin data from server.');
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, []);

  const recentMerchants = useMemo(() => merchants.slice(0, 8), [merchants]);

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
          <div className="stat-label">Active Merchants</div>
          <div className="stat-value">{(stats?.active_merchants || 0).toLocaleString('en-IN')}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Failed Payments</div>
          <div className="stat-value warning">{(stats?.failed_payments || 0).toLocaleString('en-IN')}</div>
        </div>
      </div>

      <div className="table-card" style={{ marginTop: '20px' }}>
        <div className="page-header" style={{ marginBottom: '15px' }}>
          <h3>Recent Merchants</h3>
        </div>
        <table>
          <thead>
            <tr>
              <th>Merchant</th>
              <th>Email</th>
              <th>Business Type</th>
              <th>Status</th>
              <th>Date Joined</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {recentMerchants.map(m => (
              <tr key={m.id}>
                <td>{m.business_name || m.name}</td>
                <td>{m.email || 'N/A'}</td>
                <td>{m.business_type || 'N/A'}</td>
                <td>{m.onboarding_status || 'N/A'}</td>
                <td>{m.created_at ? new Date(m.created_at).toLocaleDateString() : 'N/A'}</td>
                <td>
                  <button
                    className="btn-primary"
                    style={{ padding: '4px 10px', fontSize: '12px', display: 'flex', alignItems: 'center', gap: '4px' }}
                    onClick={() => handleImpersonate(m.id)}
                    disabled={impersonating === m.id}
                  >
                    <ExternalLink size={12} />
                    {impersonating === m.id ? 'Opening...' : 'Open'}
                  </button>
                </td>
              </tr>
            ))}
            {recentMerchants.length === 0 && (
              <tr>
                <td colSpan={6} style={{ textAlign: 'center', padding: '20px' }}>No merchants found</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
