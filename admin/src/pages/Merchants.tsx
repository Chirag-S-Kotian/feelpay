import { useEffect, useMemo, useState } from 'react';
import { ExternalLink, Search } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

const DASHBOARD_BASE_URL = import.meta.env.VITE_DASHBOARD_BASE_URL || window.location.origin;

interface Merchant {
  id: string;
  name: string;
  email?: string | null;
  business_name?: string | null;
  business_type?: string | null;
  onboarding_status?: string | null;
  created_at?: string | null;
}

interface MerchantListResponse {
  merchants: Merchant[];
}

export default function Merchants() {
  const [items, setItems] = useState<Merchant[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState('');
  const [impersonating, setImpersonating] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await apiRequest<MerchantListResponse>('GET', '/admin/merchants');
        setItems(data.merchants || []);
      } catch (err) {
        console.error('Failed to load merchants', err);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter((m) =>
      [m.name, m.email, m.business_name, m.id].some((v) => (v || '').toLowerCase().includes(q)),
    );
  }, [items, query]);

  const handleImpersonate = async (merchantId: string) => {
    setImpersonating(merchantId);
    try {
      const res = await apiRequest<{ token: string }>('POST', `/admin/merchants/${merchantId}/impersonate`);
      const dashboardUrl = new URL('/login', DASHBOARD_BASE_URL);
      dashboardUrl.searchParams.set('token', res.token);
      window.open(dashboardUrl.toString(), '_blank');
    } catch (err) {
      console.error('Failed to impersonate merchant', err);
    } finally {
      setImpersonating(null);
    }
  };

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Business Accounts</h1>
          <p>Business account directory from live DB</p>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 20 }}>
        <div className="global-search" style={{ maxWidth: '100%' }}>
          <Search size={16} />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search by id, name, email"
            aria-label="Search business accounts"
          />
        </div>
      </div>

      {loading ? (
        <div className="card">Loading business accounts...</div>
      ) : (
        <div className="table-card">
          <table>
            <thead>
              <tr>
                <th>Business Account</th>
                <th>Email</th>
                <th>Business</th>
                <th>Status</th>
                <th>Joined</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((m) => (
                <tr key={m.id}>
                  <td>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                      <div className="logo-icon" style={{ width: 32, height: 32, fontSize: 12 }}>
                        {(m.business_name || m.name || '?').charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <div style={{ fontWeight: 600 }}>{m.business_name || m.name}</div>
                        <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>{m.id}</div>
                      </div>
                    </div>
                  </td>
                  <td>{m.email || 'N/A'}</td>
                  <td>{m.business_type || 'N/A'}</td>
                  <td>{m.onboarding_status || 'N/A'}</td>
                  <td>{m.created_at ? new Date(m.created_at).toLocaleDateString() : 'N/A'}</td>
                  <td>
                    <button
                      className="btn-primary"
                      style={{ minHeight: 34, padding: '6px 10px' }}
                      onClick={() => handleImpersonate(m.id)}
                      disabled={impersonating === m.id}
                    >
                      <ExternalLink size={14} />
                      {impersonating === m.id ? 'Opening...' : 'Open Business Account'}
                    </button>
                  </td>
                </tr>
              ))}
              {filtered.length === 0 && (
                <tr>
                  <td colSpan={6} style={{ textAlign: 'center', padding: 28 }}>No business accounts found</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
