import { useState, useEffect } from 'react';
import { Save, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import PerUserNotificationPreferences from '../components/PerUserNotificationPreferences';

interface NotificationPreference {
  event_type: string;
  email_enabled: boolean;
  sms_enabled: boolean;
  in_app_enabled: boolean;
}

const EVENT_TYPES = [
  'payment.success',
  'payment.failed',
  'refund.initiated',
  'refund.success',
  'dispute.created',
  'settlement.processed',
  'subscription.charged',
  'subscription.failed',
  'kyc.approved',
  'kyc.rejected',
  'kyc.resubmit_required',
];

const CHANNELS = ['email', 'sms', 'in_app'] as const;

export default function NotificationPreferences() {
  const [preferences, setPreferences] = useState<Record<string, NotificationPreference>>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showPerUser, setShowPerUser] = useState(false);

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  useEffect(() => {
    fetchPreferences();
  }, []);

  const fetchPreferences = async () => {
    if (!merchantId) {
      setError('Merchant session missing. Login again.');
      setLoading(false);
      return;
    }
    try {
      const data = await apiRequest<{ preferences: NotificationPreference[] }>(
        'GET',
        `/v1/merchants/${merchantId}/notification-preferences`
      );

      const prefs: Record<string, NotificationPreference> = {};
      EVENT_TYPES.forEach(eventType => {
        const existing = data.preferences?.find(p => p.event_type === eventType);
        prefs[eventType] = existing || {
          event_type: eventType,
          email_enabled: true,
          sms_enabled: false,
          in_app_enabled: true,
        };
      });

      setPreferences(prefs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load preferences');
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = (eventType: string, channel: typeof CHANNELS[number]) => {
    const current = preferences[eventType];
    if (!current) return;
    setPreferences(prev => ({
      ...prev,
      [eventType]: {
        ...current,
        [`${channel}_enabled`]: !current[`${channel}_enabled`],
      },
    }));
    setSuccess(false);
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSuccess(false);

    try {
      const updates = Object.values(preferences);
      await Promise.all(
        updates.map(pref =>
          apiRequest(
            'PUT',
            `/v1/merchants/${merchantId}/notification-preferences/${pref.event_type}`,
            {
              email_enabled: pref.email_enabled,
              sms_enabled: pref.sms_enabled,
              in_app_enabled: pref.in_app_enabled,
            }
          )
        )
      );
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save preferences');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">Loading preferences...</div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>Notification Preferences</h1>
        <p>Configure how you receive notifications for different events</p>
      </div>

      <div className="tabs">
        <button
          className={`tab ${!showPerUser ? 'active' : ''}`}
          onClick={() => setShowPerUser(false)}
        >
          Merchant Preferences
        </button>
        <button
          className={`tab ${showPerUser ? 'active' : ''}`}
          onClick={() => setShowPerUser(true)}
        >
          Team Member Preferences
        </button>
      </div>

      {showPerUser ? (
        <PerUserNotificationPreferences />
      ) : (
        <div className="card">
          {error && (
            <div className="alert alert-error">
              <AlertCircle size={16} />
              {error}
            </div>
          )}

          {success && (
            <div className="alert alert-success">
              Preferences saved successfully!
            </div>
          )}

          <div className="preferences-matrix">
            <div className="matrix-header">
              <div className="event-column">Event Type</div>
              {CHANNELS.map(channel => (
                <div key={channel} className="channel-column">
                  {channel === 'in_app' ? 'In-App' : channel.charAt(0).toUpperCase() + channel.slice(1)}
                </div>
              ))}
            </div>

            {EVENT_TYPES.map(eventType => (
              <div key={eventType} className="matrix-row">
                <div className="event-column">
                  <span className="event-label">{eventType}</span>
                </div>
                {CHANNELS.map(channel => (
                  <div key={`${eventType}-${channel}`} className="channel-column">
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={Boolean(preferences[eventType]?.[`${channel}_enabled`])}
                        onChange={() => handleToggle(eventType, channel)}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>
                ))}
              </div>
            ))}
          </div>

          <div className="form-actions">
            <button
              className="btn-primary"
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? 'Saving...' : (
                <>
                  <Save size={16} />
                  Save Preferences
                </>
              )}
            </button>
          </div>
        </div>
      )}

      <style>{`
        .tabs {
          display: inline-flex;
          gap: 8px;
          margin-bottom: 14px;
          padding: 4px;
          border: 1px solid rgba(77, 141, 235, 0.25);
          background: rgba(10, 25, 48, 0.6);
          border-radius: 12px;
        }
        .tab {
          border: 1px solid transparent;
          background: transparent;
          color: #9ebbe6;
          border-radius: 9px;
          padding: 8px 12px;
          font-size: 12px;
          font-weight: 700;
          cursor: pointer;
        }
        .tab.active {
          border-color: rgba(94, 155, 247, 0.45);
          background: linear-gradient(180deg, rgba(25, 55, 96, 0.95), rgba(17, 41, 74, 0.95));
          color: #edf5ff;
        }
        .card {
          border-radius: 14px;
          border: 1px solid rgba(77, 141, 235, 0.3);
          background: linear-gradient(180deg, rgba(17, 39, 72, 0.96), rgba(10, 25, 47, 0.96));
        }
        .alert {
          display: flex;
          align-items: center;
          gap: 8px;
          border-radius: 10px;
          padding: 10px 12px;
          margin-bottom: 12px;
          font-size: 13px;
          font-weight: 600;
        }
        .alert-error {
          border: 1px solid rgba(247, 83, 96, 0.45);
          background: rgba(247, 83, 96, 0.14);
          color: #ffadb6;
        }
        .alert-success {
          border: 1px solid rgba(55, 198, 130, 0.42);
          background: rgba(55, 198, 130, 0.12);
          color: #aaf4ce;
        }
        .preferences-matrix {
          border: 1px solid rgba(91, 154, 247, 0.3);
          border-radius: 12px;
          overflow: hidden;
          margin-bottom: 20px;
        }
        .matrix-header {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          background: rgba(21, 49, 87, 0.82);
          border-bottom: 1px solid rgba(91, 154, 247, 0.34);
          font-weight: 600;
          font-size: 13px;
          color: #d9e9ff;
        }
        .matrix-row {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          border-bottom: 1px solid rgba(91, 154, 247, 0.22);
          align-items: center;
          background: rgba(9, 24, 45, 0.45);
        }
        .matrix-row:last-child {
          border-bottom: none;
        }
        .event-column,
        .channel-column {
          padding: 12px 16px;
        }
        .event-label {
          font-size: 13px;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
          color: #d2e4ff;
        }
        .channel-column {
          display: flex;
          justify-content: center;
        }
        .toggle-switch {
          position: relative;
          display: inline-block;
          width: 40px;
          height: 24px;
        }
        .toggle-switch input {
          opacity: 0;
          width: 0;
          height: 0;
        }
        .toggle-slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #8a98ac;
          transition: 0.3s;
          border-radius: 24px;
        }
        .toggle-slider:before {
          position: absolute;
          content: "";
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }
        input:checked + .toggle-slider {
          background: linear-gradient(135deg, #f8a429, #9f5cff);
        }
        input:checked + .toggle-slider:before {
          transform: translateX(16px);
        }
        @media (max-width: 768px) {
          .matrix-header,
          .matrix-row {
            grid-template-columns: 120px repeat(3, 1fr);
          }
          .event-column,
          .channel-column {
            padding: 8px 12px;
            font-size: 12px;
          }
        }
      `}</style>
    </div>
  );
}
