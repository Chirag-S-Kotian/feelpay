import { useState, useEffect } from 'react';
import { Download, FileText, Loader, RefreshCw, X } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import { formatPaise } from '../utils/money';

interface Settlement {
  id: string;
  settlement_date?: string;
  amount: number;
  fee_amount: number;
  tax_amount: number;
  net_amount: number;
  status: string;
  created_at: string;
  utr?: string;
  transaction_count?: number;
}

interface SettlementItem {
  transaction_id: string;
  amount: number;
  fee: number;
}

interface SettlementDetail {
  id: string;
  settlement_date?: string;
  amount: number;
  fee_amount: number;
  tax_amount: number;
  refund_adjustments?: number;
  chargeback_adjustments?: number;
  other_adjustments?: number;
  net_amount: number;
  status: string;
  utr?: string;
  bank_reference?: string;
  bank_account?: {
    account_holder_name: string;
    account_number_masked: string;
    ifsc_code: string;
  } | null;
  processed_at?: string;
  created_at: string;
  items: SettlementItem[];
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return 'N/A';
  const dateOnly = dateStr.split('T')[0] || dateStr;
  const parts = dateOnly.split('-');
  if (parts.length === 3) {
    return `${parts[2]}/${parts[1]}/${parts[0]}`;
  }
  return dateOnly;
};

export default function SettlementReports() {
  const [settlements, setSettlements] = useState<Settlement[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedFormat, setSelectedFormat] = useState<'pdf' | 'csv' | 'excel'>('pdf');
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [selectedSettlement, setSelectedSettlement] = useState<Settlement | null>(null);
  const [reportPreview, setReportPreview] = useState<SettlementDetail | null>(null);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    fetchSettlements();
  }, []);

  const fetchSettlements = async () => {
    try {
      setError('');
      const data = await apiRequest<{ settlements: Settlement[] }>(
        'GET',
        '/v1/settlements?page=1&per_page=50'
      );
      setSettlements(data.settlements || []);
    } catch (err) {
      console.error('Failed to fetch settlements:', err);
      const message = err instanceof Error ? err.message : 'Failed to load settlements';
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  const handlePreview = async (settlement: Settlement) => {
    setSelectedSettlement(settlement);
    try {
      setError('');
      const data = await apiRequest<SettlementDetail>(
        'GET',
        `/v1/settlements/${settlement.id}`
      );
      setReportPreview(data);
    } catch (err) {
      console.error('Failed to fetch report preview:', err);
      const message = err instanceof Error ? err.message : 'Failed to load settlement detail';
      setError(message);
    }
  };

  const buildCsv = (detail: SettlementDetail) => {
    const rows = [
      ['Settlement ID', detail.id],
      ['Settlement Date', detail.settlement_date || 'N/A'],
      ['Status', detail.status],
      ['UTR', detail.utr || 'N/A'],
      ['Gross Amount (INR)', formatPaise(detail.amount, { symbol: false })],
      ['Fee Amount (INR)', formatPaise(detail.fee_amount, { symbol: false })],
      ['Tax Amount (INR)', formatPaise(detail.tax_amount, { symbol: false })],
      ['Net Amount (INR)', formatPaise(detail.net_amount, { symbol: false })],
      ['Transaction Count', `${detail.items?.length || 0}`],
      [],
      ['Transaction ID', 'Amount (INR)', 'Fee (INR)'],
      ...(detail.items || []).map(item => [
        item.transaction_id,
        formatPaise(item.amount, { symbol: false }),
        formatPaise(item.fee, { symbol: false }),
      ]),
    ];

    return rows
      .map(row =>
        row
          .map(cell => `"${String(cell ?? '').replace(/"/g, '""')}"`)
          .join(',')
      )
      .join('\n');
  };

  const downloadTextFile = (fileName: string, content: string, mimeType: string) => {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = fileName;
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  };

  const buildPdf = (detail: SettlementDetail) => {
    const lines = [
      'FeelPay Settlement Report',
      `Settlement ID: ${detail.id}`,
      `Date: ${detail.settlement_date || detail.created_at}`,
      `Status: ${detail.status}`,
      `UTR: ${detail.utr || 'N/A'}`,
      `Gross: INR ${formatPaise(detail.amount, { symbol: false })}`,
      `Fees + Tax: INR ${formatPaise(detail.fee_amount + detail.tax_amount, { symbol: false })}`,
      `Net: INR ${formatPaise(detail.net_amount, { symbol: false })}`,
      `Rows: ${detail.items?.length || 0}`,
    ];
    const escaped = lines.join('\\n').replace(/[()\\]/g, '\\$&');
    const stream = `BT /F1 16 Tf 54 760 Td (${escaped}) Tj ET`;
    const objects = [
      '1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj',
      '2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj',
      '3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >> endobj',
      '4 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj',
      `5 0 obj << /Length ${stream.length} >> stream\n${stream}\nendstream endobj`,
    ];
    const body = objects.join('\n');
    return `%PDF-1.4\n${body}\ntrailer << /Root 1 0 R >>\n%%EOF`;
  };

  const handleDownload = async (settlementId: string, format: 'pdf' | 'csv' | 'excel') => {
    setDownloadingId(settlementId);
    try {
      setError('');
      let detail = reportPreview;

      if (!detail || detail.id !== settlementId) {
        detail = await apiRequest<SettlementDetail>('GET', `/v1/settlements/${settlementId}`);
        setReportPreview(detail);
      }

      if (!detail) {
        throw new Error('Settlement detail unavailable');
      }

      const csvContent = buildCsv(detail);
      const timestamp = new Date().toISOString().slice(0, 10);
      const baseName = `settlement-${detail.id}-${timestamp}`;

      if (format === 'csv') {
        downloadTextFile(`${baseName}.csv`, csvContent, 'text/csv;charset=utf-8');
      } else if (format === 'excel') {
        downloadTextFile(
          `${baseName}.xls`,
          csvContent,
          'application/vnd.ms-excel;charset=utf-8'
        );
      } else {
        downloadTextFile(`${baseName}.pdf`, buildPdf(detail), 'application/pdf');
      }
    } catch (err) {
      console.error('Download failed:', err);
      const message = err instanceof Error ? err.message : 'Download failed';
      setError(message);
    } finally {
      setDownloadingId(null);
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">Loading settlements...</div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Settlement Reports</h1>
        <p>Download detailed settlement reports</p>
      </div>
      {error && (
        <div className="error-banner" role="alert">
          {error}
        </div>
      )}

      <div className={selectedSettlement && reportPreview ? "grid grid-2" : ""}>
        <div className="card">
          <div className="card-title-row">
            <h2>Settlements</h2>
            <button className="refresh-btn" onClick={fetchSettlements} type="button">
              <RefreshCw size={14} />
              Refresh
            </button>
          </div>
          {settlements.length === 0 ? (
            <div className="empty-state">
              <FileText size={32} />
              <h3>No settlements found</h3>
              <p>No settlement batches exist yet. Complete sandbox checkout payments and add a verified settlement bank account to generate test-mode reports.</p>
              <a className="btn-primary" href="/bank-accounts">Set up bank account</a>
            </div>
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Status</th>
                    <th>Transactions</th>
                    <th>Gross Amount</th>
                    <th>Fee + Tax</th>
                    <th>Net Amount</th>
                    <th>Action</th>
                  </tr>
                </thead>
                <tbody>
                  {settlements.map(settlement => (
                    <tr key={settlement.id}>
                      <td>
                        {formatDate(settlement.settlement_date || settlement.created_at)}
                      </td>
                      <td>
                        <span className={`status-pill status-${settlement.status.toLowerCase()}`}>
                          {settlement.status}
                        </span>
                      </td>
                      <td>{settlement.transaction_count ?? 1}</td>
                      <td>{formatPaise(settlement.amount)}</td>
                      <td>{formatPaise(settlement.fee_amount + settlement.tax_amount)}</td>
                      <td className="font-bold">{formatPaise(settlement.net_amount)}</td>
                      <td>
                        <button
                          className="btn-small"
                          onClick={() => handlePreview(settlement)}
                        >
                          <FileText size={14} />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {selectedSettlement && reportPreview && (
          <div className="card">
            <div className="card-title-row">
              <h2>Report Preview</h2>
              <button
                className="close-btn"
                onClick={() => {
                  setSelectedSettlement(null);
                  setReportPreview(null);
                }}
                type="button"
                aria-label="Close preview"
              >
                <X size={18} />
              </button>
            </div>
            <div className="report-preview">
              <div className="preview-section">
                <h3>Settlement Summary</h3>
                <div className="preview-row">
                  <span>Settlement Date:</span>
                  <strong>
                    {formatDate(reportPreview.settlement_date || reportPreview.created_at)}
                  </strong>
                </div>
                <div className="preview-row">
                  <span>Transactions:</span>
                  <strong>{reportPreview.items?.length || 0}</strong>
                </div>
              </div>

              <div className="preview-section">
                <h3>Financial Breakdown</h3>
                <div className="preview-row">
                  <span>Gross Amount:</span>
                  <strong>{formatPaise(reportPreview.amount)}</strong>
                </div>
                <div className="preview-row">
                  <span>Fee:</span>
                  <strong>{formatPaise(reportPreview.fee_amount)}</strong>
                </div>
                <div className="preview-row">
                  <span>Tax on Fee:</span>
                  <strong>{formatPaise(reportPreview.tax_amount)}</strong>
                </div>
                <div className="preview-row">
                  <span>Refunds:</span>
                  <strong>{formatPaise(reportPreview.refund_adjustments || 0)}</strong>
                </div>
                <div className="preview-row">
                  <span>Chargebacks:</span>
                  <strong>{formatPaise(reportPreview.chargeback_adjustments || 0)}</strong>
                </div>
                <div className="preview-row">
                  <span>Adjustments:</span>
                  <strong>{formatPaise(reportPreview.other_adjustments || 0)}</strong>
                </div>
                <div className="preview-row">
                  <span>Net Amount:</span>
                  <strong>{formatPaise(reportPreview.net_amount)}</strong>
                </div>
              </div>

              <div className="preview-section">
                <h3>Bank Transfer</h3>
                <div className="preview-row">
                  <span>Status:</span>
                  <strong>{reportPreview.status}</strong>
                </div>
                <div className="preview-row">
                  <span>Transfer Date:</span>
                  <strong>
                    {formatDate(reportPreview.processed_at)}
                  </strong>
                </div>
                <div className="preview-row">
                  <span>UTR:</span>
                  <strong className="mono">{reportPreview.utr || 'N/A'}</strong>
                </div>
                <div className="preview-row">
                  <span>Bank Account:</span>
                  <strong>
                    {reportPreview.bank_account
                      ? `${reportPreview.bank_account.account_number_masked} / ${reportPreview.bank_account.ifsc_code}`
                      : 'N/A'}
                  </strong>
                </div>
              </div>

              <div className="format-selector">
                <label>Download Format:</label>
                <div className="format-buttons">
                  {(['pdf', 'csv', 'excel'] as const).map(fmt => (
                    <button
                      key={fmt}
                      className={`btn-format ${selectedFormat === fmt ? 'active' : ''}`}
                      onClick={() => setSelectedFormat(fmt)}
                    >
                      {fmt.toUpperCase()}
                    </button>
                  ))}
                </div>
              </div>

              <button
                className="btn-primary"
                onClick={() => handleDownload(selectedSettlement.id, selectedFormat)}
                disabled={downloadingId === selectedSettlement.id}
              >
                {downloadingId === selectedSettlement.id ? (
                  <>
                    <Loader size={16} className="spin" />
                    Downloading...
                  </>
                ) : (
                  <>
                    <Download size={16} />
                    Download Report
                  </>
                )}
              </button>
            </div>
          </div>
        )}
      </div>

      <style>{`
        .error-banner {
          margin-bottom: 16px;
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
          border-radius: 12px;
          padding: 10px 14px;
          font-size: 13px;
          font-weight: 600;
        }
        .card-title-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 12px;
          margin-bottom: 6px;
        }
        .refresh-btn {
          display: inline-flex;
          align-items: center;
          gap: 6px;
          border: 1px solid var(--fp-border);
          color: var(--fp-text);
          background: var(--fp-bg-elevated);
          border-radius: 10px;
          padding: 8px 12px;
          font-size: 12px;
          font-weight: 600;
          cursor: pointer;
          transition: all var(--fp-transition);
        }
        .refresh-btn:hover {
          background: var(--fp-bg-hover);
          border-color: var(--fp-text-dim);
          color: var(--fp-primary);
        }
        .grid-2 {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
        }
        @media (max-width: 1024px) {
          .grid-2 {
            grid-template-columns: 1fr;
          }
        }
        .report-preview {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }
        .preview-section {
          border: 1px solid var(--border-color);
          border-radius: 8px;
          padding: 12px;
          background: var(--bg-secondary);
        }
        .preview-section h3 {
          margin: 0 0 12px 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--text-secondary);
        }
        .preview-row {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          border-bottom: 1px solid var(--border-color);
          font-size: 14px;
        }
        .preview-row:last-child {
          border-bottom: none;
        }
        .preview-row.highlight {
          background: var(--bg-tertiary);
          padding: 8px;
          border-radius: 4px;
          font-weight: 600;
        }
        .status-pill {
          display: inline-flex;
          border-radius: 999px;
          padding: 3px 8px;
          font-size: 11px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.04em;
        }
        .status-processed,
        .status-completed,
        .status-success {
          background: #10b981;
          color: #ffffff;
          box-shadow: 0 0 8px rgba(16, 185, 129, 0.4);
        }
        .status-pending {
          background: #f59e0b;
          color: #ffffff;
          box-shadow: 0 0 8px rgba(245, 158, 11, 0.4);
        }
        .status-processing {
          background: #3b82f6;
          color: #ffffff;
          box-shadow: 0 0 8px rgba(59, 130, 246, 0.4);
        }
        .status-held {
          background: #8b5cf6;
          color: #ffffff;
          box-shadow: 0 0 8px rgba(139, 92, 246, 0.4);
        }
        .status-failed {
          background: #ef4444;
          color: #ffffff;
          box-shadow: 0 0 8px rgba(239, 68, 68, 0.4);
        }
        .format-selector {
          margin-top: 12px;
        }
        .format-buttons {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }
        .btn-format {
          flex: 1;
          padding: 10px 12px;
          border: 1px solid #6B6862;
          background: #FFF9EC;
          border-radius: 100px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
          color: #6B6862;
          transition: all 0.2s ease;
        }
        .btn-format:hover {
          background: #F5EFE2;
          border-color: #141413;
          color: #141413;
        }
        .btn-format.active {
          background: #EB001B;
          color: #FFF9EC;
          border-color: #EB001B;
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        .data-table th, .data-table td {
          white-space: nowrap;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        .close-btn {
          background: transparent;
          border: none;
          color: #6B6862;
          cursor: pointer;
          padding: 6px;
          display: flex;
          align-items: center;
          justify-content: center;
          border-radius: 50%;
          transition: all 0.2s ease;
        }
        .close-btn:hover {
          background: #F5EFE2;
          color: #EB001B;
        }
      `}</style>
    </div>
  );
}
