import { useState } from 'react';
import { X, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface ScheduledReport {
  id: string;
  report_type: string;
  frequency: 'daily' | 'weekly' | 'monthly';
  day_of_week?: number;
  day_of_month?: number;
  time_of_day: string;
  recipient_emails: string[];
  format: 'csv' | 'pdf' | 'excel';
  is_active: boolean;
}

interface Props {
  report?: ScheduledReport | null;
  onClose: () => void;
}

export default function ScheduleReportForm({ report, onClose }: Props) {
  const [formData, setFormData] = useState({
    report_type: report?.report_type || 'transaction_summary',
    frequency: report?.frequency || 'daily' as const,
    day_of_week: report?.day_of_week || 1,
    day_of_month: report?.day_of_month || 1,
    time_of_day: report?.time_of_day || '09:00',
    recipient_emails: report?.recipient_emails.join(', ') || '',
    format: report?.format || 'pdf' as const,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  const validate = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.recipient_emails.trim()) {
      newErrors.recipient_emails = 'At least one email is required';
    } else {
      const emails = formData.recipient_emails.split(',').map(e => e.trim());
      const invalidEmails = emails.filter(e => !e.match(/^[^\s@]+@[^\s@]+\.[^\s@]+$/));
      if (invalidEmails.length > 0) {
        newErrors.recipient_emails = 'Invalid email format';
      }
    }

    if (formData.frequency === 'weekly' && (formData.day_of_week < 0 || formData.day_of_week > 6)) {
      newErrors.day_of_week = 'Invalid day of week';
    }

    if (formData.frequency === 'monthly' && (formData.day_of_month < 1 || formData.day_of_month > 31)) {
      newErrors.day_of_month = 'Day must be between 1 and 31';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;
    if (!merchantId) {
      setErrors({ submit: 'Merchant session missing. Login again.' });
      return;
    }

    setLoading(true);

    try {
      const payload = {
        report_type: formData.report_type,
        frequency: formData.frequency,
        day_of_week: formData.frequency === 'weekly' ? formData.day_of_week : undefined,
        day_of_month: formData.frequency === 'monthly' ? formData.day_of_month : undefined,
        time_of_day: formData.time_of_day,
        recipient_emails: formData.recipient_emails.split(',').map(e => e.trim()),
        format: formData.format,
      };

      if (report) {
        await apiRequest(
          'PUT',
          `/v1/merchants/${merchantId}/scheduled-reports/${report.id}`,
          payload
        );
      } else {
        await apiRequest(
          'POST',
          `/v1/merchants/${merchantId}/scheduled-reports`,
          payload
        );
      }

      onClose();
    } catch (err) {
      setErrors({
        submit: err instanceof Error ? err.message : 'Failed to save schedule',
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{report ? 'Edit Schedule' : 'Schedule Report'}</h2>
          <button className="btn-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="form">
          {errors.submit && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {errors.submit}
            </div>
          )}

          <div className="form-group">
            <label>Report Type *</label>
            <select
              value={formData.report_type}
              onChange={e => setFormData({ ...formData, report_type: e.target.value })}
            >
              <option value="transaction_summary">Transaction Summary</option>
              <option value="settlement_reconciliation">Settlement Reconciliation</option>
              <option value="refund_report">Refund Report</option>
              <option value="dispute_report">Payment Issues Report</option>
              <option value="gst_report">GST Report</option>
              <option value="custom">Custom Date Range</option>
            </select>
          </div>

          <div className="form-group">
            <label>Frequency *</label>
            <select
              value={formData.frequency}
              onChange={e => setFormData({ ...formData, frequency: e.target.value as any })}
            >
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
            </select>
          </div>

          {formData.frequency === 'weekly' && (
            <div className="form-group">
              <label>Day of Week *</label>
              <select
                value={formData.day_of_week}
                onChange={e => setFormData({ ...formData, day_of_week: parseInt(e.target.value) })}
              >
                <option value={0}>Sunday</option>
                <option value={1}>Monday</option>
                <option value={2}>Tuesday</option>
                <option value={3}>Wednesday</option>
                <option value={4}>Thursday</option>
                <option value={5}>Friday</option>
                <option value={6}>Saturday</option>
              </select>
              {errors.day_of_week && <span className="error-text">{errors.day_of_week}</span>}
            </div>
          )}

          {formData.frequency === 'monthly' && (
            <div className="form-group">
              <label>Day of Month *</label>
              <input
                type="number"
                min="1"
                max="31"
                value={formData.day_of_month}
                onChange={e => setFormData({ ...formData, day_of_month: parseInt(e.target.value) })}
              />
              {errors.day_of_month && <span className="error-text">{errors.day_of_month}</span>}
            </div>
          )}

          <div className="form-group">
            <label>Time of Day (HH:MM) *</label>
            <input
              type="time"
              value={formData.time_of_day}
              onChange={e => setFormData({ ...formData, time_of_day: e.target.value })}
            />
          </div>

          <div className="form-group">
            <label>Recipient Emails * (comma-separated)</label>
            <textarea
              value={formData.recipient_emails}
              onChange={e => setFormData({ ...formData, recipient_emails: e.target.value })}
              placeholder="finance@company.com, accounting@company.com"
              rows={3}
            />
            {errors.recipient_emails && <span className="error-text">{errors.recipient_emails}</span>}
          </div>

          <div className="form-group">
            <label>Format *</label>
            <select
              value={formData.format}
              onChange={e => setFormData({ ...formData, format: e.target.value as any })}
            >
              <option value="csv">CSV</option>
              <option value="pdf">PDF</option>
              <option value="excel">Excel</option>
            </select>
          </div>

          <div className="form-actions">
            <button type="button" className="btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? 'Saving...' : 'Save Schedule'}
            </button>
          </div>
        </form>
      </div>

      <style>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(20, 20, 19, 0.38);
          backdrop-filter: blur(5px);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 2000;
          padding: 16px;
        }
        .modal-content {
          background: #fff9ec;
          border-radius: 28px;
          border: 1px solid var(--fp-border);
          box-shadow: 0 18px 70px rgba(20, 20, 19, 0.2);
          max-width: 620px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
          isolation: isolate;
        }
        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 22px;
          border-bottom: 1px solid var(--fp-border);
        }
        .modal-header h2 {
          margin: 0;
          font-size: 20px;
          font-weight: 800;
          color: var(--fp-text);
        }
        .btn-close {
          background: #f2ead8;
          border: 1px solid var(--fp-border);
          border-radius: 999px;
          cursor: pointer;
          padding: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--fp-text-muted);
        }
        .btn-close:hover {
          border-color: var(--fp-cta);
          color: var(--fp-text);
        }
        .form {
          padding: 20px 22px 24px;
        }
        .form-group {
          margin-bottom: 18px;
        }
        .form-group label {
          display: block;
          margin-bottom: 7px;
          font-weight: 700;
          font-size: 13px;
          letter-spacing: 0.02em;
          color: var(--fp-text-muted);
        }
        .form-group input,
        .form-group select,
        .form-group textarea {
          width: 100%;
          padding: 11px 13px;
          border: 1px solid var(--fp-border);
          border-radius: 999px;
          font-size: 14px;
          font-family: inherit;
          color: var(--fp-text);
          background: #fff9ec;
          outline: none;
          transition: border-color 0.2s ease, box-shadow 0.2s ease;
        }
        .form-group input:focus,
        .form-group select:focus,
        .form-group textarea:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }
        .form-group textarea {
          min-height: 92px;
          resize: vertical;
          border-radius: 20px;
        }
        .form-group input::placeholder,
        .form-group textarea::placeholder {
          color: var(--fp-text-dim);
        }
        .error-text {
          display: block;
          color: #991627;
          font-size: 12px;
          margin-top: 6px;
        }
        .form-actions {
          display: flex;
          gap: 12px;
          margin-top: 26px;
        }
        .form-actions button {
          flex: 1;
          min-height: 40px;
        }
        .alert.alert-error {
          display: flex;
          align-items: center;
          gap: 8px;
          border: 1px solid rgba(200, 31, 47, 0.35);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
          border-radius: 16px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
}
