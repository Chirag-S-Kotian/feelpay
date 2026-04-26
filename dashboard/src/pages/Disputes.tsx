const DISPUTES = [
  {
    id: 'dsp_001',
    paymentId: 'fp_7e4f5a6b',
    amount: 15999,
    reason: 'fraudulent',
    status: 'needs_response',
    dueBy: '2026-05-01',
  },
  {
    id: 'dsp_002',
    paymentId: 'fp_3a2b1c0d',
    amount: 7499,
    reason: 'service_not_received',
    status: 'under_review',
    dueBy: '2026-04-29',
  },
  {
    id: 'dsp_003',
    paymentId: 'fp_1a0b9c8d',
    amount: 4999,
    reason: 'duplicate',
    status: 'won',
    dueBy: '2026-04-20',
  },
];

export default function Disputes() {
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
        {DISPUTES.map((d) => (
          <div key={d.id} className="dispute-card">
            <div className="dispute-card__head">
              <span className={`badge ${d.status}`}>{d.status}</span>
              <span className="dispute-card__id">{d.id}</span>
            </div>
            <div className="dispute-card__amount">₹{d.amount.toLocaleString('en-IN')}</div>
            <div className="dispute-card__row">
              <span>Payment</span>
              <code>{d.paymentId}</code>
            </div>
            <div className="dispute-card__row">
              <span>Reason</span>
              <strong>{d.reason.replaceAll('_', ' ')}</strong>
            </div>
            <div className="dispute-card__row">
              <span>Evidence due</span>
              <strong>{d.dueBy}</strong>
            </div>
            <div className="dispute-card__actions">
              <button className="btn btn-secondary">Accept</button>
              <button className="btn btn-primary">Respond</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
