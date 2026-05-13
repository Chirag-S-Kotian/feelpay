import { useState, useEffect } from 'react';
import { Search, Download } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { SideDrawer } from '../components/SideDrawer';

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
  const [balance, setBalance] = useState<{ available: number; pending: number }>({ available: 0, pending: 0 });
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [error, setError] = useState('');
  const [reco, setReco] = useState<ReconciliationSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedSettlement, setSelectedSettlement] = useState<Settlement | null>(null);
  const [settlementDetails, setSettlementDetails] = useState<any>(null);
  const [detailsLoading, setDetailsLoading] = useState(false);

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
        const [settlementRes, balanceRes, recoRes] = await Promise.all([
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
          }>('GET', '/v1/settlements', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
          apiRequest<{ available: number; pending: number }>('GET', '/v1/settlements/balance', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
          apiRequest<ReconciliationSummary>('GET', '/v1/settlements/reconciliation', undefined, null, { useApiKey: true, cacheMs: 20_000 }),
        ]);

        setSettlements(
          settlementRes.settlements.map((s) => ({
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
        setBalance(balanceRes);
        setReco(recoRes);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load settlements');
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const formatCurrency = (val: number) => new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 0 }).format(val);

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
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--fp-success)' }}>{formatCurrency(selectedSettlement.net)}</div>
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
                  <span>{formatCurrency(selectedSettlement.amount)}</span>
                </div>
                <div className="fee-row">
                  <span>Merchant Fee</span>
                  <span className="warning">- {formatCurrency(selectedSettlement.fee)}</span>
                </div>
                <div className="fee-row">
                  <span>Service Tax (GST)</span>
                  <span className="warning">- {formatCurrency(selectedSettlement.tax)}</span>
                </div>
                <div className="fee-row">
                  <span>Refunds</span>
                  <span className="warning">- {formatCurrency(settlementDetails?.refund_adjustments || 0)}</span>
                </div>
                <div className="fee-row">
                  <span>Chargebacks</span>
                  <span className="warning">- {formatCurrency(settlementDetails?.chargeback_adjustments || 0)}</span>
                </div>
                <div className="fee-row">
                  <span>Adjustments</span>
                  <span>{formatCurrency(settlementDetails?.other_adjustments || 0)}</span>
                </div>
                <div className="fee-row total">
                  <span>Final Payout</span>
                  <span className="success">{formatCurrency(selectedSettlement.net)}</span>
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
                        <div style={{ fontSize: 13, fontWeight: 600 }}>{formatCurrency(item.amount)}</div>
                        <div style={{ fontSize: 11, color: 'var(--fp-text-muted)' }}>Fee: {formatCurrency(item.fee)}</div>
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
        <button className="btn-secondary" onClick={exportCsv}>
          <Download size={16} /> Export CSV
        </button>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Available Balance</div>
          <div className="stat-value success">{formatCurrency(balance.available)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Pending Settlement</div>
          <div className="stat-value warning">{formatCurrency(balance.pending)}</div>
        </div>
      </div>

      {reco && (
        <div className="stats-grid" style={{ marginBottom: 16 }}>
          <div className="stat-card">
            <div className="stat-label">Gross</div>
            <div className="stat-value">{formatCurrency(reco.gross)}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Fees + Tax</div>
            <div className="stat-value warning">{formatCurrency(reco.fees + reco.tax)}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Net</div>
            <div className="stat-value success">{formatCurrency(reco.net)}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Counts (C/P/F)</div>
            <div className="stat-value">{reco.counts.completed}/{reco.counts.pending}/{reco.counts.failed}</div>
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
                <td>{formatCurrency(settlement.amount)}</td>
                <td>{formatCurrency(settlement.fee)}</td>
                <td>{formatCurrency(settlement.tax)}</td>
                <td className="net">{formatCurrency(settlement.net)}</td>
                <td className="mono">{settlement.utr || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
