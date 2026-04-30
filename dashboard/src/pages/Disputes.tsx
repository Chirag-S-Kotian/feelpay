import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Dispute = {
  id: string;
  payment_order_id: string;
  amount: number;
  reason_code?: string | null;
  reason_message?: string | null;
  status: string;
  evidence_due_by?: string | null;
};

export default function Disputes() {
  const [disputes, setDisputes] = useState<Dispute[]>([]);
  const [loading, setLoading] = useState(true);
  const [showEvidenceModal, setShowEvidenceModal] = useState(false);
  const [targetDisputeId, setTargetDisputeId] = useState<string | null>(null);
  const [evidenceText, setEvidenceText] = useState('');
  const [evidenceFiles, setEvidenceFiles] = useState<File[]>([]);
  const [submittingEvidence, setSubmittingEvidence] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const actionableDisputes = disputes.filter(d =>
    ['open', 'needs_response', 'under_review'].includes(d.status)
  );

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiRequest<{ disputes: Dispute[] }>(
        'GET',
        '/v1/disputes',
      );
      setDisputes(res.disputes || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load disputes');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const acceptDispute = async (id: string) => {
    try {
      setError(null);
      await apiRequest('POST', `/v1/disputes/${id}/accept`, {});
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to accept dispute');
    }
  };

  const openEvidenceModal = (id?: string) => {
    setTargetDisputeId(id || actionableDisputes[0]?.id || null);
    setEvidenceText('');
    setEvidenceFiles([]);
    setError(null);
    setShowEvidenceModal(true);
  };

  const fileToDataUrl = (file: File) =>
    new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result || ''));
      reader.onerror = () => reject(new Error('Failed to read file'));
      reader.readAsDataURL(file);
    });

  const submitEvidence = async () => {
    if (!targetDisputeId) return;
    setSubmittingEvidence(true);
    setError(null);
    try {
      const docs = await Promise.all(evidenceFiles.map(fileToDataUrl));
      await apiRequest(
        'POST',
        `/v1/disputes/${targetDisputeId}/evidence`,
        {
          text: evidenceText || 'Evidence submitted from merchant dashboard.',
          documents: docs,
        },
      );
      setShowEvidenceModal(false);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Evidence upload failed');
    } finally {
      setSubmittingEvidence(false);
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Disputes</h1>
          <p>Chargeback defense command center</p>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => openEvidenceModal()}
        >
          Upload Evidence
        </button>
      </div>

      <div className="dispute-banner">
        <div className="dispute-banner__kicker">RISK SURFACE</div>
        <div className="dispute-banner__title">Action window is short. Reply with evidence before due-by cut.</div>
      </div>

      <div className="dispute-grid">
        {error && <div className="modal-error">{error}</div>}
        {!loading && disputes.map((d) => (
          <div key={d.id} className="dispute-card">
            <div className="dispute-card__head">
              <span className={`badge ${d.status}`}>{d.status}</span>
              <span className="dispute-card__id">{d.id}</span>
            </div>
            <div className="dispute-card__amount">₹{d.amount.toLocaleString('en-IN')}</div>
            <div className="dispute-card__row">
              <span>Payment</span>
              <code>{d.payment_order_id}</code>
            </div>
            <div className="dispute-card__row">
              <span>Reason</span>
              <strong>{(d.reason_message || d.reason_code || 'unknown').replaceAll('_', ' ')}</strong>
            </div>
            <div className="dispute-card__row">
              <span>Evidence due</span>
              <strong>{d.evidence_due_by ? new Date(d.evidence_due_by).toLocaleDateString('en-IN') : '-'}</strong>
            </div>
            <div className="dispute-card__actions">
              <button className="btn btn-secondary" onClick={() => void acceptDispute(d.id)}>Accept</button>
              <button
                className="btn btn-primary"
                onClick={() => openEvidenceModal(d.id)}
                disabled={!['open', 'needs_response', 'under_review'].includes(d.status)}
              >
                Respond
              </button>
            </div>
          </div>
        ))}
        {!loading && disputes.length === 0 && <div>No disputes found.</div>}
      </div>

      {showEvidenceModal && (
        <div className="modal-overlay" onClick={() => setShowEvidenceModal(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-kicker">Chargeback Defense</div>
            <h3>Upload Evidence</h3>
            <p className="modal-subtitle">Send supporting docs and context to strengthen your dispute response.</p>
            {error && <div className="modal-error">{error}</div>}
            {actionableDisputes.length === 0 && (
              <div className="modal-hint">
                No open disputes right now. Add dispute via webhook/test flow first.
              </div>
            )}
            {actionableDisputes.length > 0 && (
              <>
                <label>Select dispute</label>
                <select
                  value={targetDisputeId || ''}
                  onChange={e => setTargetDisputeId(e.target.value)}
                >
                  {actionableDisputes.map(d => (
                    <option key={d.id} value={d.id}>
                      {d.id} ({d.status})
                    </option>
                  ))}
                </select>
              </>
            )}
            <label>Evidence note</label>
            <textarea
              value={evidenceText}
              onChange={e => setEvidenceText(e.target.value)}
              placeholder="Explain why charge valid. Add order timeline, delivery proof, customer communication."
              rows={4}
            />
            <label>Evidence files</label>
            <label className="file-drop">
              <input
                type="file"
                multiple
                accept=".pdf,.png,.jpg,.jpeg,.txt"
                onChange={e => setEvidenceFiles(Array.from(e.target.files || []))}
              />
              <span className="file-drop-title">Choose files or drag into picker</span>
              <span className="file-drop-sub">PDF, PNG, JPG, TXT</span>
            </label>
            <div className="file-count">{evidenceFiles.length} file(s) selected</div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowEvidenceModal(false)}>
                Cancel
              </button>
              <button
                className="btn btn-primary"
                onClick={() => void submitEvidence()}
                disabled={submittingEvidence || actionableDisputes.length === 0 || !targetDisputeId}
              >
                {submittingEvidence ? 'Submitting...' : 'Submit Evidence'}
              </button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(3, 8, 20, 0.74);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2200;
          padding: 16px;
        }
        .modal-content {
          width: min(620px, 100%);
          background: radial-gradient(140% 160% at 20% 0%, rgba(34, 85, 152, 0.36), rgba(16, 38, 74, 0.98));
          border: 1px solid rgba(108, 168, 255, 0.35);
          border-radius: 18px;
          padding: 20px;
          color: #e8f2ff;
          display: flex;
          flex-direction: column;
          gap: 11px;
          box-shadow: 0 18px 60px rgba(0, 0, 0, 0.42);
        }
        .modal-kicker {
          font-size: 11px;
          letter-spacing: 0.16em;
          text-transform: uppercase;
          color: #9cc3ff;
          font-weight: 700;
        }
        .modal-content h3 {
          margin: 0;
          font-size: 28px;
          font-weight: 800;
          line-height: 1.05;
          color: #f0f6ff;
        }
        .modal-subtitle {
          margin: 0 0 2px;
          font-size: 13px;
          color: #a8c4ec;
          line-height: 1.5;
        }
        .modal-content label {
          font-size: 13px;
          font-weight: 700;
          color: #c8dbfb;
        }
        .modal-content textarea,
        .modal-content input[type="file"],
        .modal-content select {
          width: 100%;
          padding: 10px 12px;
          border-radius: 10px;
          border: 1px solid rgba(91, 154, 247, 0.34);
          background: linear-gradient(180deg, rgba(17, 41, 77, 0.94), rgba(12, 29, 56, 0.94));
          color: #eaf3ff;
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .modal-content textarea:focus,
        .modal-content select:focus {
          border-color: rgba(122, 182, 255, 0.95);
          box-shadow: 0 0 0 3px rgba(77, 141, 235, 0.2);
        }
        .modal-content textarea {
          resize: vertical;
          min-height: 100px;
        }
        .modal-content textarea::placeholder {
          color: #93b2de;
        }
        .file-drop {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 6px;
          border: 1px dashed rgba(122, 182, 255, 0.55);
          border-radius: 12px;
          min-height: 84px;
          padding: 12px;
          background: linear-gradient(180deg, rgba(22, 51, 92, 0.9), rgba(16, 38, 70, 0.9));
          cursor: pointer;
        }
        .file-drop input[type="file"] {
          position: absolute;
          opacity: 0;
          width: 1px;
          height: 1px;
          pointer-events: none;
        }
        .file-drop-title {
          font-size: 13px;
          font-weight: 700;
          color: #dbe9ff;
        }
        .file-drop-sub {
          font-size: 12px;
          color: #92b4e2;
        }
        .file-count {
          font-size: 12px;
          color: #b5ccef;
        }
        .modal-hint {
          border: 1px solid rgba(255, 180, 85, 0.35);
          background: rgba(255, 180, 85, 0.12);
          color: #ffd79d;
          border-radius: 10px;
          padding: 9px 11px;
          font-size: 12px;
          font-weight: 600;
        }
        .modal-actions {
          display: flex;
          gap: 10px;
          justify-content: flex-end;
          margin-top: 10px;
        }
        .modal-error {
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
          border-radius: 10px;
          padding: 8px 10px;
          font-size: 13px;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
}
