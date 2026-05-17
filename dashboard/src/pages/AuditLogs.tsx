import { useEffect, useState } from 'react';
import { ClipboardList, Search } from 'lucide-react';
import { SideDrawer } from '../components/SideDrawer';
import { apiRequest, MerchantAuthError } from '../hooks/useApi';

type AuditLog = {
  id: string;
  action: string;
  target: string;
  metadata?: Record<string, unknown>;
  actor_type?: string | null;
  actor_id?: string | null;
  ip_address?: string | null;
  user_agent?: string | null;
  created_at: string;
};

function formatAction(action: string) {
  return action.replace(/_/g, ' ');
}

function safeMetadata(metadata?: Record<string, unknown>) {
  if (!metadata) return '{}';
  return JSON.stringify(metadata, null, 2);
}

function SkeletonTable() {
  return (
    <div className="card audit-table-card">
      <table className="data-table">
        <tbody>
          {[...Array(5)].map((_, i) => (
            <tr key={i}>
              <td><div className="skeleton" style={{ width: 160, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 120, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 180, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 120, height: 16 }} /></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function AuditLogs() {
  const [items, setItems] = useState<AuditLog[]>([]);
  const [selected, setSelected] = useState<AuditLog | null>(null);
  const [action, setAction] = useState('all');
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    const loadAuditLogs = async () => {
      setLoading(true);
      setError('');
      try {
        const params = new URLSearchParams({ limit: '100' });
        if (action !== 'all') params.set('action', action);
        const response = await apiRequest<{ items?: AuditLog[] }>(
          'GET',
          `/v1/merchants/audit-logs?${params.toString()}`,
          undefined,
          undefined,
          { cacheMs: 0 },
        );
        setItems(response.items || []);
      } catch (err) {
        if (err instanceof MerchantAuthError) return;
        setError(err instanceof Error ? err.message : 'Unable to load audit logs');
      } finally {
        setLoading(false);
      }
    };
    void loadAuditLogs();
  }, [action]);

  const actions = Array.from(new Set(items.map((item) => item.action))).sort();
  const filtered = items.filter((item) => {
    const query = search.trim().toLowerCase();
    if (!query) return true;
    return `${item.action} ${item.target} ${item.actor_id || ''}`.toLowerCase().includes(query);
  });

  return (
    <>
      <SideDrawer
        isOpen={Boolean(selected)}
        onClose={() => setSelected(null)}
        title="Audit Event"
        subtitle={selected?.id}
      >
        {selected && (
          <div className="detail-section">
            <div className="detail-grid">
              <div className="detail-item"><label>Action</label><span>{formatAction(selected.action)}</span></div>
              <div className="detail-item"><label>Target</label><span>{selected.target}</span></div>
              <div className="detail-item"><label>Actor</label><span>{selected.actor_id || 'System / Unknown'}</span></div>
              <div className="detail-item"><label>IP</label><span>{selected.ip_address || 'Local / Unknown'}</span></div>
              <div className="detail-item"><label>User Agent</label><span>{selected.user_agent || 'Unknown'}</span></div>
              <div className="detail-item"><label>Created</label><span>{new Date(selected.created_at).toLocaleString('en-IN')}</span></div>
            </div>
            <pre className="audit-metadata">{safeMetadata(selected.metadata)}</pre>
          </div>
        )}
      </SideDrawer>

      <div className="fade-in">
        <div className="page-header">
          <div>
            <h1>Audit Logs</h1>
            <p>Review merchant-sensitive activity across settings, team, webhooks, and checkout</p>
          </div>
        </div>

        <div className="audit-toolbar">
          <div className="pill-tabs">
            <button type="button" className={`pill-tab ${action === 'all' ? 'active' : ''}`} onClick={() => setAction('all')}>All</button>
            {actions.map((item) => (
              <button key={item} type="button" className={`pill-tab ${action === item ? 'active' : ''}`} onClick={() => setAction(item)}>
                {formatAction(item)}
              </button>
            ))}
          </div>
          <label className="search-box">
            <Search size={16} aria-hidden={true} />
            <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search action, target, actor" />
          </label>
        </div>

        {error && <div className="error-banner" role="alert">{error}</div>}
        {loading ? <SkeletonTable /> : (
          <div className="card audit-table-card">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Action</th>
                  <th>Target</th>
                  <th>Actor</th>
                  <th>IP</th>
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((item) => (
                  <tr key={item.id} onClick={() => setSelected(item)} style={{ cursor: 'pointer' }}>
                    <td>{formatAction(item.action)}</td>
                    <td>{item.target}</td>
                    <td>{item.actor_id || 'System / Unknown'}</td>
                    <td>{item.ip_address || 'Local / Unknown'}</td>
                    <td>{new Date(item.created_at).toLocaleString('en-IN')}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {filtered.length === 0 && (
              <div className="empty-state">
                <ClipboardList size={28} aria-hidden={true} />
                <strong>No audit logs found</strong>
                <p>Sensitive merchant actions will appear here after they are recorded.</p>
              </div>
            )}
          </div>
        )}
      </div>

      <style>{`
        .audit-toolbar {
          display: flex;
          align-items: center;
          gap: 16px;
          margin-bottom: 24px;
          flex-wrap: wrap;
        }
        .audit-table-card {
          overflow-x: auto;
          -webkit-overflow-scrolling: touch;
        }
        .audit-metadata {
          margin-top: 16px;
          padding: 12px;
          border-radius: 8px;
          border: 1px solid var(--fp-border);
          background: var(--fp-bg-elevated);
          color: var(--fp-text);
          white-space: pre-wrap;
          word-break: break-word;
          font-size: 12px;
        }
        @media (max-width: 760px) {
          .audit-toolbar,
          .audit-toolbar .pill-tabs,
          .search-box {
            width: 100%;
          }
        }
      `}</style>
    </>
  );
}
