import { Link } from 'react-router-dom';
import { useState } from 'react';
import { BarChart3, Calendar, Download, FileText, Repeat2, Wallet } from 'lucide-react';

const reportSurfaces = [
  {
    title: 'Transaction Exports',
    description: 'Create CSV or Excel exports with transaction status, payment method, amount, refund, and date filters.',
    path: '/exports',
    icon: Download,
  },
  {
    title: 'Settlement Reports',
    description: 'Review settlement rows, UTR references, gross amount, fees, taxes, net amount, and linked transactions.',
    path: '/settlement-reports',
    icon: Wallet,
  },
  {
    title: 'Scheduled Reports',
    description: 'Manage recurring report delivery for finance and reconciliation workflows.',
    path: '/scheduled-reports',
    icon: Calendar,
  },
  {
    title: 'Refund Analysis',
    description: 'Filter refunds by status, date, order, customer, and amount from the dedicated refunds workspace.',
    path: '/refunds',
    icon: Repeat2,
  },
  {
    title: 'Fees and Taxes',
    description: 'Use analytics and settlement reports to reconcile fee and tax totals against processed volume.',
    path: '/analytics',
    icon: BarChart3,
  },
];

export default function Reports() {
  const today = new Date().toISOString().slice(0, 10);
  const [endDate, setEndDate] = useState(today);
  const [startDate, setStartDate] = useState(() => {
    const date = new Date();
    date.setDate(date.getDate() - 30);
    return date.toISOString().slice(0, 10);
  });

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Reports</h1>
        <p>Transactions, settlements, refunds, fees, taxes, and export controls for merchant reconciliation.</p>
      </div>

      <div className="card">
        <div className="card-title-row">
          <div>
            <h2>Report Window</h2>
            <p className="muted">Use the same date window when opening report modules.</p>
          </div>
          <FileText size={22} />
        </div>
        <div className="date-range" style={{ marginTop: 16 }}>
          <Calendar size={16} className="date-range-icon" />
          <input 
            type="date" 
            value={startDate} 
            onChange={(event) => setStartDate(event.target.value)} 
            onClick={(e) => (e.target as any).showPicker?.()}
          />
          <span className="date-separator">to</span>
          <input 
            type="date" 
            value={endDate} 
            onChange={(event) => setEndDate(event.target.value)} 
            onClick={(e) => (e.target as any).showPicker?.()}
          />
        </div>
      </div>

      <div className="grid grid-2">
        {reportSurfaces.map((surface) => {
          const Icon = surface.icon;
          return (
            <div className="card" key={surface.path}>
              <div className="card-title-row">
                <div>
                  <h2>{surface.title}</h2>
                  <p className="muted">{surface.description}</p>
                </div>
                <Icon size={22} />
              </div>
              <Link className="btn-primary" to={`${surface.path}?start=${startDate}&end=${endDate}`}>
                Open
              </Link>
            </div>
          );
        })}
      </div>
    </div>
  );
}
