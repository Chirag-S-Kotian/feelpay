import { useState, useEffect } from 'react';
import { apiRequest } from '../hooks/useApi';

export interface Merchant {
  id: string;
  name: string;
  email?: string;
  business_name?: string | null;
  business_type?: string | null;
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

import {
  Activity,
  ArrowUpRight,
  ExternalLink,
} from 'lucide-react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  BarChart,
  Bar,
} from 'recharts';


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
      // Open dashboard in new tab with token
      const dashboardUrl = `http://localhost:5173/login?token=${res.token}`;
      window.open(dashboardUrl, '_blank');
    } catch (err) {
      alert('Failed to impersonate merchant');
    } finally {
      setImpersonating(null);
    }
  };

  useEffect(() => {
    async function fetchData() {
      try {
        const [statsData, merchantsData] = await Promise.all([
          apiRequest<PlatformStats>('GET', '/admin/stats'),
          apiRequest<Merchant[] | MerchantsResponse>('GET', '/admin/merchants')
        ]);
        setStats(statsData);
        setMerchants(Array.isArray(merchantsData) ? merchantsData : merchantsData.merchants || []);
      } catch (err) {
        console.error('Failed to fetch admin data:', err);
        setError('Connection to admin server failed. Ensure server is running on port 8081.');
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, []);

  if (loading) {
    return <div className="page">Loading platform data...</div>;
  }

  if (error) {
    return (
      <div className="page fade-in">
        <div className="stat-card warning" style={{ maxWidth: '400px', margin: '100px auto', textAlign: 'center' }}>
          <h3>Service Unavailable</h3>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  // No chart data if server doesn't provide it yet
  const chartData = stats ? [
    { name: 'Global', volume: stats.total_volume, tx: stats.active_merchants }
  ] : [];

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Global Overview</h1>
          <p>Real-time platform performance monitoring</p>
        </div>
        <div className="btn-secondary">
          <Activity size={16} />
          System Status: {stats?.system_health || 'Checking...'}
        </div>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Total Volume (Global)</div>
          <div className="stat-value">₹{(stats?.total_volume || 0).toLocaleString('en-IN')}.00</div>
          <div className="stat-change positive">
            <ArrowUpRight size={14} /> +12.4% vs last week
          </div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Total Merchants</div>
          <div className="stat-value">{(stats?.active_merchants || 0).toLocaleString()}</div>
          <div className="stat-change positive">
            <ArrowUpRight size={14} /> +42 new this month
          </div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Failed Payments</div>
          <div className="stat-value warning">{(stats?.failed_payments || 0).toLocaleString()}</div>
          <div className="stat-change negative">
             Requires Review
          </div>
        </div>
        <div className="stat-card">
          <div className="stat-label">System Uptime</div>
          <div className="stat-value success">99.98%</div>
          <div className="stat-change positive">
            L7 Nodes: 4 Active
          </div>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-card">
          <h3>Transaction Flow (TPV)</h3>
          <div style={{ width: '100%', height: 300 }}>
            <ResponsiveContainer>
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#f59e0b" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#2e3f60" vertical={false} />
                <XAxis dataKey="name" stroke="#7a879e" />
                <YAxis stroke="#7a879e" />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#121d34', border: '1px solid #2e3f60', borderRadius: '10px' }}
                />
                <Area type="monotone" dataKey="volume" stroke="#f59e0b" fillOpacity={1} fill="url(#colorVolume)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div className="chart-card">
          <h3>Merchant Distribution</h3>
          <div style={{ width: '100%', height: 300 }}>
            <ResponsiveContainer>
              <BarChart data={chartData} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#2e3f60" horizontal={false} />
                <XAxis type="number" hide />
                <YAxis dataKey="name" type="category" stroke="#7a879e" />
                <Tooltip 
                   cursor={{fill: 'rgba(255,255,255,0.05)'}}
                   contentStyle={{ backgroundColor: '#121d34', border: '1px solid #2e3f60', borderRadius: '10px' }}
                />
                <Bar dataKey="volume" radius={[0, 4, 4, 0]} fill="#f59e0b" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      <div className="table-card" style={{ marginTop: '20px' }}>
        <div className="page-header" style={{ marginBottom: '15px' }}>
           <h3>Recent Merchants</h3>
           <button className="btn-secondary">View All</button>
        </div>
        <table>
          <thead>
            <tr>
              <th>Merchant</th>
              <th>Email</th>
              <th>Business Type</th>
              <th>Date Joined</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {merchants.map(m => (
              <tr key={m.id}>
                <td>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                     <div className="logo-icon" style={{ width: '32px', height: '32px', fontSize: '12px' }}>
                       {(m.business_name || m.name).charAt(0)}
                     </div>
                     {m.business_name || m.name}
                  </div>
                </td>
                <td>{m.email || 'N/A'}</td>
                <td>{m.business_type || 'N/A'}</td>
                <td>{m.created_at ? new Date(m.created_at).toLocaleDateString() : 'N/A'}</td>
                <td>
                  <div style={{ display: 'flex', gap: '8px' }}>
                    <button className="btn-secondary" style={{ padding: '4px 10px', fontSize: '12px' }}>
                      Review
                    </button>
                    <button 
                      className="btn-primary" 
                      style={{ padding: '4px 10px', fontSize: '12px', display: 'flex', alignItems: 'center', gap: '4px' }}
                      onClick={() => handleImpersonate(m.id)}
                      disabled={impersonating === m.id}
                    >
                      <ExternalLink size={12} />
                      {impersonating === m.id ? 'Loading...' : 'View'}
                    </button>
                  </div>
                </td>
              </tr>
            ))}
            {merchants.length === 0 && (
              <tr>
                <td colSpan={5} style={{ textAlign: 'center', padding: '20px' }}>No merchants found</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
