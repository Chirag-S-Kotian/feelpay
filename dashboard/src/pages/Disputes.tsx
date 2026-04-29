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

  const load = async () => {
    setLoading(true);
    try {
      const res = await apiRequest<{ disputes: Dispute[] }>(
        'GET',
        '/v1/disputes',
        undefined,
        undefined,
        { useApiKey: true },
      );
      setDisputes(res.disputes || []);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const acceptDispute = async (id: string) => {
    await apiRequest('POST', `/v1/disputes/${id}/accept`, {}, undefined, { useApiKey: true });
    await load();
  };

  const respondDispute = async (id: string) => {
    await apiRequest(
      'POST',
      `/v1/disputes/${id}/evidence`,
      { text: 'Evidence submitted from merchant dashboard.' },
      undefined,
      { useApiKey: true },
    );
    await load();
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Disputes</h1>
          <p>Chargeback defense command center</p>
        </div>
        <button className="btn btn-primary">Upload Evidence</button>
      </div>

      <div className="dispute-banner">
        <div className="dispute-banner__kicker">RISK SURFACE</div>
        <div className="dispute-banner__title">Action window is short. Reply with evidence before due-by cut.</div>
      </div>

      <div className="dispute-grid">
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
              <button className="btn btn-primary" onClick={() => void respondDispute(d.id)}>Respond</button>
            </div>
          </div>
        ))}
        {!loading && disputes.length === 0 && <div>No disputes found.</div>}
      </div>
    </div>
  );
}
