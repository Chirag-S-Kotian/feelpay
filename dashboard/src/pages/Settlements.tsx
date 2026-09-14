import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Search, Download, FileText, AlertCircle, ArrowRight } from 'lucide-react';
import { apiRequest, getEnvironmentMode, type EnvironmentMode } from '../hooks/useApi';
import { SideDrawer } from '../components/SideDrawer';
import { formatPaise } from '../utils/money';

interface Settlement {
  id: string;
  date: string | null;
  status: string;
  amount: number;
  fee: number;
  tax: number;
  net: number;
  utr: string | null;
}

interface ReconciliationSummary {
  gross: number;
  fees: number;
  tax: number;
  net: number;
  variance: number;
  counts: {
    completed: number;
    pending: number;
    failed: number;
  };
}

export default function Settlements() {
  const [settlements, setSettlements] = useState<Settlement[]>([]);
  const [balance, setBalance] = useState<{ available: number; pending: number; hold: number; reserve: number }>({ available: 0, pending: 0, hold: 0, reserve: 0 });
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [error, setError] = useState('');
  const [reco, setReco] = useState<ReconciliationSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedSettlement, setSelectedSettlement] = useState<Settlement | null>(null);
  const [settlementDetails, setSettlementDetails] = useState<any>(null);
  const [detailsLoading, setDetailsLoading] = useState(false);
  const [mode, setMode] = useState<EnvironmentMode>(getEnvironmentMode());
  const [onboardingStatus, setOnboardingStatus] = useState<string | null>(null);

  useEffect(() => {
    const handleModeChange = (e: any) => {
      setMode(e.detail?.mode || getEnvironmentMode());
    };
    window.addEventListener('feelpay_env_mode_changed', handleModeChange);
    return () => window.removeEventListener('feelpay_env_mode_changed', handleModeChange);
  }, []);

  useEffect(() => {
    if (!selectedSettlement) return;
    const loadDetails = async () => {
      setDetailsLoading(true);
      try {
        const res = await apiRequest<any>('GET', `/v1/settlements/${selectedSettlement.id}`, undefined, undefined, { useApiKey: true });
        setSettlementDetails(res);
      } catch (e) {
        console.error('Failed to load settlement details', e);
      } finally {
        setDetailsLoading(false);
      }
    };
    void loadDetails();
  }, [selectedSettlement?.id]);

  useEffect(() => {
    const load = async () => {
      try {
        const [settlementRes, balanceRes, recoRes, onboardingRes] = await Promise.all([
          apiRequest<{
            settlements: Array<{
              id: string;
              settlement_date: string | null;
              status: string;
              amount: number;
              fee_amount: number;
              tax_amount: number;
              net_amount: number;
              utr: string | null;
            }>;
            onboarding_required?: boolean;
            message?: string;
          }>('GET', '/v1/settlements', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
          apiRequest<{ available: number; pending: number; hold: number; reserve: number; onboarding_required?: boolean }>('GET', '/v1/settlements/balance', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
          apiRequest<ReconciliationSummary>('GET', '/v1/settlements/reconciliation', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
          apiRequest<{ onboarding_status: string }>('GET', '/v1/onboarding/status', undefined, undefined, { cacheMs: 30_000 }).catch(() => ({ onboarding_status: 'pending' })),
        ]);

        setSettlements(
          (settlementRes?.settlements || []).map((s) => ({
            id: s.id,
            date: s.settlement_date,
            status: s.status,
            amount: s.amount,
            fee: s.fee_amount,
            tax: s.tax_amount,
            net: s.net_amount,
            utr: s.utr,
          })),
        );
        setBalance(balanceRes || { available: 0, pending: 0, hold: 0, reserve: 0 });
        setReco(recoRes || null);
        setOnboardingStatus(onboardingRes?.onboarding_status || 'pending');
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load settlements');
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [mode]);

  const filteredSettlements = settlements.filter((s) => {
    const statusMatch = filter === 'all' || s.status === filter;
    const q = search.trim().toLowerCase();
    const searchMatch =
      !q ||
      s.id.toLowerCase().includes(q) ||
      (s.utr || '').toLowerCase().includes(q);
    return statusMatch && searchMatch;
  });

  const exportCsv = () => {
    const rows = filteredSettlements.map((s) => [
      s.id,
      s.date || '',
      s.status,
      s.amount,
      s.fee,
      s.tax,
      s.net,
      s.utr || '',
    ]);
    const csv = [
      'settlement_id,date,status,amount,fee,tax,net,utr',
      ...rows.map((r) => r.join(',')),
    ].join('\n');
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `settlements-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="fade-in">
      {/* Detail Drawer */}
      <SideDrawer
        isOpen={!!selectedSettlement}
        onClose={() => setSelectedSettlement(null)}
        title="Settlement Details"
        subtitle={selectedSettlement?.id}
      >
        {selectedSettlement && (
          <>
            <div className="detail-section">
              <div style={{ display: 'flex', gap: 12, marginBottom: 24 }}>
                <div style={{ flex: 1, padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12, border: '1px solid var(--fp-border)' }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 4 }}>Net Amount</label>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--fp-success)' }}>{formatPaise(selectedSettlement.net)}</div>
                </div>
                <div style={{ flex: 1, padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12, border: '1px solid var(--fp-border)' }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--fp-text-muted)', marginBottom: 4 }}>Status</label>
                  <span className={`status-badge ${selectedSettlement.status}`}>{selectedSettlement.status}</span>
                </div>
              </div>

              <div className="detail-grid">
                <div className="detail-item">
                  <label>UTR Number</label>
                  <span className="mono">{selectedSettlement.utr || 'Pending'}</span>
                </div>
                <div className="detail-item">
                  <label>Settlement Date</label>
                  <span>{selectedSettlement.date || 'Pending'}</span>
                </div>
              </div>
            </div>

            <div className="detail-section">
              <h4>Fee Breakdown</h4>
              <div className="fee-card" style={{ padding: 16, background: 'var(--fp-bg-elevated)', borderRadius: 12 }}>
                <div className="fee-row">
                  <span>Gross Amount</span>
                  <span>{formatPaise(selectedSettlement.amount)}</span>
                </div>
                <div className="fee-row">
                  <span>Merchant Fee</span>
                  <span className="warning">- {formatPaise(selectedSettlement.fee)}</span>
                </div>
                <div className="fee-row">
                  <span>Service Tax (GST)</span>
                  <span className="warning">- {formatPaise(selectedSettlement.tax)}</span>
                </div>
                <div className="fee-row">
                  <span>Refunds</span>
                  <span className="warning">- {formatPaise(settlementDetails?.refund_adjustments || 0)}</span>
                </div>
                <div className="fee-row">
                  <span>Chargebacks</span>
                  <span className="warning">- {formatPaise(settlementDetails?.chargeback_adjustments || 0)}</span>
                </div>
                <div className="fee-row">
                  <span>Adjustments</span>
                  <span>{formatPaise(settlementDetails?.other_adjustments || 0)}</span>
                </div>
                <div className="fee-row total">
                  <span>Final Payout</span>
                  <span className="success">{formatPaise(selectedSettlement.net)}</span>
                </div>
              </div>
            </div>

            <div className="detail-section">
              <h4>Bank Account</h4>
              <div className="detail-grid">
                <div><span>Account</span><strong>{settlementDetails?.bank_account?.account_number_masked || 'N/A'}</strong></div>
                <div><span>IFSC</span><strong>{settlementDetails?.bank_account?.ifsc_code || 'N/A'}</strong></div>
              </div>
            </div>

            <div className="detail-section">
              <h4>Included Transactions</h4>
              {detailsLoading ? <div className="skeleton" style={{ height: 100 }} /> : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {settlementDetails?.items?.map((item: any, i: number) => (
                    <div key={i} style={{ padding: 12, background: 'var(--fp-bg-elevated)', borderRadius: 8, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div>
                        <div style={{ fontSize: 13, fontWeight: 500, fontFamily: 'monospace' }}>{item.transaction_id.slice(0, 8)}...</div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted)' }}>{new Date(item.created_at).toLocaleDateString()}</div>
                      </div>
                      <div style={{ textAlign: 'right' }}>
                        <div style={{ fontSize: 13, fontWeight: 600 }}>{formatPaise(item.amount)}</div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted)' }}>Fee: {formatPaise(item.fee)}</div>
                      </div>
                    </div>
                  ))}
                  {(!settlementDetails?.items || settlementDetails.items.length === 0) && (
                    <p style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>No individual transaction breakdown available.</p>
                  )}
                </div>
              )}
            </div>
          </>
        )}
      </SideDrawer>

      <div className="page-header">
        <h1>Settlements</h1>
        <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
          <Link to="/settlement-reports" className="btn-secondary" style={{ display: 'inline-flex', alignItems: 'center', gap: '0.5rem' }}>
            <FileText size={16} /> Detailed Reports
          </Link>
          <button className="btn-secondary" onClick={exportCsv}>
            <Download size={16} /> Export CSV
          </button>
        </div>
      </div>

      {error && <div className="error-message" style={{ marginBottom: 16 }}>{error}</div>}

      {mode === 'live' && onboardingStatus !== 'approved' && !loading ? (
        <div className="kyc-warning-card" style={{ margin: '24px 0' }}>
          <div className="kyc-warning-blur"></div>
          <div className="kyc-warning-content">
            <div className="kyc-icon-wrapper">
              <AlertCircle size={36} className="kyc-warning-icon" />
            </div>
            <h2>Live Settlements Inactive</h2>
            <p className="kyc-warning-subtitle">
              Your FeelPay merchant account is in the <strong style={{ textTransform: 'uppercase' }}>{onboardingStatus || 'pending'}</strong> verification state.
            </p>
            <p className="kyc-warning-description">
              Live payouts and settlement transfers to your bank account are strictly held until your KYC onboarding profile and business verification are reviewed and approved.
            </p>
            <div className="kyc-actions">
              <Link to="/onboarding" className="btn btn-kyc-action">
                Complete Onboarding Profile
              </Link>
            </div>
          </div>
        </div>
      ) : (
        <>
          {mode === 'test' && onboardingStatus !== 'approved' && !loading && (
            <div className="sandbox-notice-banner">
              <div className="sandbox-notice-left">
                <div className="sandbox-notice-icon">
                  <AlertCircle size={22} />
                </div>
                <div>
                  <div className="sandbox-notice-title-row">
                    <span>Sandbox Test Mode • Simulated Settlements</span>
                    <span className="sandbox-notice-badge">
                      KYC {onboardingStatus || 'pending'}
                    </span>
                  </div>
                  <div className="sandbox-notice-desc">
                    These transactions and balances are simulated test-mode records for sandbox testing. Real settlements to your bank account will activate once KYC onboarding is approved.
                  </div>
                </div>
              </div>
              <Link to="/onboarding" className="sandbox-notice-action">
                Complete KYC Onboarding <ArrowRight size={15} />
              </Link>
            </div>
          )}
          {mode === 'test' && onboardingStatus === 'approved' && !loading && (
            <div className="sandbox-info-banner">
              <span style={{ width: 10, height: 10, borderRadius: '50%', background: '#2563eb', flexShrink: 0 }}></span>
              <span><strong>Sandbox Environment:</strong> Viewing simulated test settlements. Switch to Live mode in the top bar to view live transactions.</span>
            </div>
          )}

          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-label">Available Balance</div>
              <div className="stat-value success">{formatPaise(balance.available)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Pending Settlement</div>
              <div className="stat-value warning">{formatPaise(balance.pending)}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">On Hold</div>
              <div className="stat-value warning">{formatPaise(balance.hold)}</div>
            </div>
          </div>

          {reco && (
            <div className="stats-grid" style={{ marginBottom: 16 }}>
              <div className="stat-card">
                <div className="stat-label">Gross</div>
                <div className="stat-value">{formatPaise(reco.gross)}</div>
              </div>
              <div className="stat-card">
                <div className="stat-label">Fees + Tax</div>
                <div className="stat-value warning">{formatPaise(reco.fees + reco.tax)}</div>
              </div>
              <div className="stat-card">
                <div className="stat-label">Net</div>
                <div className="stat-value success">{formatPaise(reco.net)}</div>
              </div>
            </div>
          )}

          <div className="filter-bar settlements-filter-bar">
            <div className="search-box settlements-search-box">
              <Search size={16} />
              <input type="text" placeholder="Search by ID or UTR..." value={search} onChange={(e) => setSearch(e.target.value)} />
            </div>
            <select className="settlements-filter-select" value={filter} onChange={(e) => setFilter(e.target.value)}>
              <option value="all">All Status</option>
              <option value="pending">Pending</option>
              <option value="completed">Completed</option>
              <option value="failed">Failed</option>
            </select>
          </div>

          <div className="table-card">
            {loading && <div>Loading settlements...</div>}
            {error && <div className="error-message">{error}</div>}
            <table>
              <thead>
                <tr>
                  <th>Settlement ID</th>
                  <th>Date</th>
                  <th>Status</th>
                  <th>Amount</th>
                  <th>Fee</th>
                  <th>Tax</th>
                  <th>Net</th>
                  <th>UTR</th>
                </tr>
              </thead>
              <tbody>
                {filteredSettlements.map((settlement) => (
                  <tr key={settlement.id} onClick={() => setSelectedSettlement(settlement)} style={{ cursor: 'pointer' }}>
                    <td className="mono">{settlement.id}</td>
                    <td>{settlement.date || '-'}</td>
                    <td>
                      <span className={`status-badge ${settlement.status}`}>
                        {settlement.status}
                      </span>
                    </td>
                    <td>{formatPaise(settlement.amount)}</td>
                    <td>{formatPaise(settlement.fee)}</td>
                    <td>{formatPaise(settlement.tax)}</td>
                    <td className="net">{formatPaise(settlement.net)}</td>
                    <td className="mono">{settlement.utr || '-'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
