import { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, Download, Landmark, Scale } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

type OperationsSummary = {
  today_volume: number;
  failed_last_24h: number;
  disputes_needing_response: number;
  pending_settlements: number;
  queued_exports: number;
  capture_rate_bps: number;
};

type Transaction = {
  id: string;
  merchant_id: string;
  amount: number;
  status: string;
  payment_method?: string | null;
  created_at: string;
};

type Dispute = {
  id: string;
  payment_order_id: string;
  amount: number;
  status: string;
  reason?: string | null;
  created_at: string;
};

type Settlement = {
  id: string;
  merchant_id: string;
  amount: number;
  status: string;
  created_at: string;
};

type ExportJob = {
  job_id: string;
  status: string;
  created_at: string;
  error_message?: string | null;
};

function formatInr(value: number) {
  return `₹${(value || 0).toLocaleString('en-IN')}`;
}

function formatDate(value: string) {
  return new Date(value).toLocaleString();
}

export default function AdminOperations() {
  const [summary, setSummary] = useState<OperationsSummary | null>(null);
  const [failedTransactions, setFailedTransactions] = useState<Transaction[]>([]);
  const [disputes, setDisputes] = useState<Dispute[]>([]);
  const [settlements, setSettlements] = useState<Settlement[]>([]);
  const [exports, setExports] = useState<ExportJob[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const [summaryData, transactionData, disputeData, settlementData, exportData] =
          await Promise.all([
            apiRequest<OperationsSummary>('GET', '/admin/operations/summary'),
            apiRequest<{ items: Transaction[] }>('GET', '/admin/transactions?status=failed&limit=10'),
            apiRequest<{ items: Dispute[] }>('GET', '/admin/disputes?status=needs_response&limit=10'),
            apiRequest<{ items: Settlement[] }>('GET', '/admin/settlements?status=pending&limit=10'),
            apiRequest<{ exports: ExportJob[] }>('GET', '/admin/exports?limit=10'),
          ]);
        setSummary(summaryData);
        setFailedTransactions(transactionData.items || []);
        setDisputes(disputeData.items || []);
        setSettlements(settlementData.items || []);
        setExports((exportData.exports || []).filter(job => job.status !== 'completed'));
      } catch (err) {
        console.error('Failed to load operations dashboard', err);
        setSummary(null);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const captureRate = useMemo(() => {
    if (!summary) return '0.00%';
    return `${(summary.capture_rate_bps / 100).toFixed(2)}%`;
  }, [summary]);

  if (loading) {
    return <div className="page">Loading operations...</div>;
  }

  return (
    <div className="page fade-in">
      <div className="page-header">
        <div>
          <h1>Operations</h1>
          <p>Exception queues for payment ops, risk, settlement, and reports</p>
        </div>
      </div>

      <div className="stats-grid">
        <article className="stat-card">
          <div className="stat-label">Today Captured</div>
          <div className="stat-value">{formatInr(summary?.today_volume || 0)}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">24h Capture Rate</div>
          <div className="stat-value success">{captureRate}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">Failed 24h</div>
          <div className="stat-value warning">{summary?.failed_last_24h || 0}</div>
        </article>
        <article className="stat-card">
          <div className="stat-label">Disputes Need Response</div>
          <div className="stat-value warning">{summary?.disputes_needing_response || 0}</div>
        </article>
      </div>

      <div className="grid grid-2">
        <section className="table-card">
          <div className="page-header" style={{ marginBottom: 12 }}>
            <h2><AlertTriangle size={18} /> Failed Payments</h2>
          </div>
          <table>
            <thead>
              <tr>
                <th>Payment</th>
                <th>Amount</th>
                <th>Method</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {failedTransactions.map(tx => (
                <tr key={tx.id}>
                  <td>{tx.id}</td>
                  <td>{formatInr(tx.amount)}</td>
                  <td>{tx.payment_method || '-'}</td>
                  <td>{formatDate(tx.created_at)}</td>
                </tr>
              ))}
              {failedTransactions.length === 0 && (
                <tr><td colSpan={4}>No failed payments in queue</td></tr>
              )}
            </tbody>
          </table>
        </section>

        <section className="table-card">
          <div className="page-header" style={{ marginBottom: 12 }}>
            <h2><Scale size={18} /> Dispute Response</h2>
          </div>
          <table>
            <thead>
              <tr>
                <th>Dispute</th>
                <th>Payment</th>
                <th>Amount</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              {disputes.map(dispute => (
                <tr key={dispute.id}>
                  <td>{dispute.id}</td>
                  <td>{dispute.payment_order_id}</td>
                  <td>{formatInr(dispute.amount)}</td>
                  <td>{dispute.reason || '-'}</td>
                </tr>
              ))}
              {disputes.length === 0 && (
                <tr><td colSpan={4}>No disputes need response</td></tr>
              )}
            </tbody>
          </table>
        </section>

        <section className="table-card">
          <div className="page-header" style={{ marginBottom: 12 }}>
            <h2><Landmark size={18} /> Settlement Queue</h2>
          </div>
          <table>
            <thead>
              <tr>
                <th>Settlement</th>
                <th>Merchant</th>
                <th>Amount</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {settlements.map(settlement => (
                <tr key={settlement.id}>
                  <td>{settlement.id}</td>
                  <td>{settlement.merchant_id}</td>
                  <td>{formatInr(settlement.amount)}</td>
                  <td>{formatDate(settlement.created_at)}</td>
                </tr>
              ))}
              {settlements.length === 0 && (
                <tr><td colSpan={4}>No pending settlements</td></tr>
              )}
            </tbody>
          </table>
        </section>

        <section className="table-card">
          <div className="page-header" style={{ marginBottom: 12 }}>
            <h2><Download size={18} /> Report Jobs</h2>
          </div>
          <table>
            <thead>
              <tr>
                <th>Job</th>
                <th>Status</th>
                <th>Created</th>
                <th>Issue</th>
              </tr>
            </thead>
            <tbody>
              {exports.map(job => (
                <tr key={job.job_id}>
                  <td>{job.job_id}</td>
                  <td><span className={`badge ${job.status}`}>{job.status}</span></td>
                  <td>{formatDate(job.created_at)}</td>
                  <td>{job.error_message || '-'}</td>
                </tr>
              ))}
              {exports.length === 0 && (
                <tr><td colSpan={4}>No active report jobs</td></tr>
              )}
            </tbody>
          </table>
        </section>
      </div>
    </div>
  );
}
