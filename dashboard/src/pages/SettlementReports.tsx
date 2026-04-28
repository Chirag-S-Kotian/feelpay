import { useState, useEffect } from 'react';
import { Download, FileText, Loader } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Settlement {
  id: string;
  settlement_date: string;
  gross_amount: number;
  fees_deducted: number;
  net_amount: number;
  transaction_count: number;
  status: string;
  bank_account?: string;
  transfer_date?: string;
  utr?: string;
}

interface SettlementReport {
  settlement_id: string;
  settlement_date: string;
  transaction_count: number;
  gross_amount: number;
  mdr_fee: number;
  gst_on_fee: number;
  refund_deductions: number;
  net_amount: number;
  bank_account: string;
  transfer_date: string;
  utr: string;
}

export default function SettlementReports() {
  const [settlements, setSettlements] = useState<Settlement[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedFormat, setSelectedFormat] = useState<'pdf' | 'csv' | 'excel'>('pdf');
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [selectedSettlement, setSelectedSettlement] = useState<Settlement | null>(null);
  const [reportPreview, setReportPreview] = useState<SettlementReport | null>(null);

  const merchantId = localStorage.getItem('feelpay_merchant_id') || 'test-merchant';

  useEffect(() => {
    fetchSettlements();
  }, []);

  const fetchSettlements = async () => {
    try {
      const data = await apiRequest<{ settlements: Settlement[] }>(
        'GET',
        `/api/v1/merchants/${merchantId}/settlements?limit=50&offset=0`
      );
      setSettlements(data.settlements || []);
    } catch (err) {
      console.error('Failed to fetch settlements:', err);
    } finally {
      setLoading(false);
    }
  };

  const handlePreview = async (settlement: Settlement) => {
    setSelectedSettlement(settlement);
    try {
      const data = await apiRequest<SettlementReport>(
        'GET',
        `/api/v1/merchants/${merchantId}/settlements/${settlement.id}/report?format=json`
      );
      setReportPreview(data);
    } catch (err) {
      console.error('Failed to fetch report preview:', err);
    }
  };

  const handleDownload = async (settlementId: string, format: 'pdf' | 'csv' | 'excel') => {
    setDownloadingId(settlementId);
    try {
      const response = await apiRequest<{ download_url: string }>(
        'GET',
        `/api/v1/merchants/${merchantId}/settlements/${settlementId}/report?format=${format}`
      );
      window.location.href = response.download_url;
    } catch (err) {
      console.error('Download failed:', err);
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

      <div className="grid grid-2">
        <div className="card">
          <h2>Settlements</h2>
          {settlements.length === 0 ? (
            <div className="empty-state">
              <FileText size={32} />
              <p>No settlements found</p>
            </div>
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Transactions</th>
                    <th>Gross Amount</th>
                    <th>Fees</th>
                    <th>Net Amount</th>
                    <th>Action</th>
                  </tr>
                </thead>
                <tbody>
                  {settlements.map(settlement => (
                    <tr key={settlement.id}>
                      <td>{new Date(settlement.settlement_date).toLocaleDateString()}</td>
                      <td>{settlement.transaction_count}</td>
                      <td>₹{(settlement.gross_amount / 100).toFixed(2)}</td>
                      <td>₹{(settlement.fees_deducted / 100).toFixed(2)}</td>
                      <td className="font-bold">₹{(settlement.net_amount / 100).toFixed(2)}</td>
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
            <h2>Report Preview</h2>
            <div className="report-preview">
              <div className="preview-section">
                <h3>Settlement Summary</h3>
                <div className="preview-row">
                  <span>Settlement Date:</span>
                  <strong>{new Date(reportPreview.settlement_date).toLocaleDateString()}</strong>
                </div>
                <div className="preview-row">
                  <span>Transactions:</span>
                  <strong>{reportPreview.transaction_count}</strong>
                </div>
              </div>

              <div className="preview-section">
                <h3>Financial Breakdown</h3>
                <div className="preview-row">
                  <span>Gross Amount:</span>
                  <strong>₹{(reportPreview.gross_amount / 100).toFixed(2)}</strong>
                </div>
                <div className="preview-row">
                  <span>MDR Fee:</span>
                  <strong>₹{(reportPreview.mdr_fee / 100).toFixed(2)}</strong>
                </div>
                <div className="preview-row">
                  <span>GST on Fee:</span>
                  <strong>₹{(reportPreview.gst_on_fee / 100).toFixed(2)}</strong>
                </div>
                <div className="preview-row">
                  <span>Refund Deductions:</span>
                  <strong>₹{(reportPreview.refund_deductions / 100).toFixed(2)}</strong>
                </div>
                <div className="preview-row highlight">
                  <span>Net Amount:</span>
                  <strong>₹{(reportPreview.net_amount / 100).toFixed(2)}</strong>
                </div>
              </div>

              <div className="preview-section">
                <h3>Bank Transfer</h3>
                <div className="preview-row">
                  <span>Bank Account:</span>
                  <strong>{reportPreview.bank_account}</strong>
                </div>
                <div className="preview-row">
                  <span>Transfer Date:</span>
                  <strong>{new Date(reportPreview.transfer_date).toLocaleDateString()}</strong>
                </div>
                <div className="preview-row">
                  <span>UTR:</span>
                  <strong className="mono">{reportPreview.utr}</strong>
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
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          background: var(--bg-secondary);
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
          transition: all 0.2s;
        }
        .btn-format.active {
          background: var(--primary);
          color: white;
          border-color: var(--primary);
        }
        .spin {
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
