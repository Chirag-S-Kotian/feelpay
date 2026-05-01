import { useState, useEffect, useMemo } from 'react';
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

  const merchantId = useMemo(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  }, []);

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
        `/v1/merchants/${merchantId}/notification-preferences`,
        undefined,
        undefined,
        { cacheMs: 45_000 }
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
          border: 1px solid var(--fp-border);
          background: #e7decb;
          border-radius: 999px;
        }
        .tab {
          border: 1px solid transparent;
          background: transparent;
          color: var(--fp-text-muted) !important;
          border-radius: 999px;
          padding: 8px 14px;
          font-size: 12px;
          font-weight: 700;
          cursor: pointer;
        }
        .tab.active {
          border-color: var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text) !important;
        }
        .card {
          border-radius: 28px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
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
          border: 1px solid rgba(200, 31, 47, 0.35);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }
        .alert-success {
          border: 1px solid rgba(52, 122, 71, 0.34);
          background: rgba(52, 122, 71, 0.1);
          color: #2f6b40;
        }
        .preferences-matrix {
          border: 1px solid var(--fp-border);
          border-radius: 20px;
          overflow-x: auto;
          overflow-y: hidden;
          margin-bottom: 20px;
          background: #fffdf5;
        }
        .matrix-header {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          min-width: 560px;
          background: #f2ead8;
          border-bottom: 1px solid var(--fp-border);
          font-weight: 600;
          font-size: 13px;
          color: var(--fp-text-muted);
        }
        .matrix-row {
          display: grid;
          grid-template-columns: 200px repeat(3, 1fr);
          min-width: 560px;
          border-bottom: 1px solid #ece2cc;
          align-items: center;
          background: #fff9ec;
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
          color: var(--fp-text);
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
          background-color: #c7baa0;
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
          background: linear-gradient(135deg, #eb001b, #f57b88);
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
