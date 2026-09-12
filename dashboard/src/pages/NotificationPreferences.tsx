import { useState, useEffect, useMemo } from 'react';
import { Save, AlertCircle, Bell, Check } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';
import PerUserNotificationPreferences from '../components/PerUserNotificationPreferences';

interface NotificationPreference {
  event_type: string;
  email_enabled: boolean;
  sms_enabled: boolean;
  in_app_enabled: boolean;
}

interface NotificationItem {
  id: string;
  notification_type: string;
  subject?: string | null;
  body?: string | null;
  status: string;
  entity_type?: string | null;
  is_read: boolean;
  created_at: string;
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
const EVENT_COPY: Record<string, string> = {
  'payment.success': 'Payment captured successfully',
  'payment.failed': 'Payment attempt failed',
  'refund.initiated': 'Refund was initiated',
  'refund.success': 'Refund completed',
  'dispute.created': 'New payment issue requires attention',
  'settlement.processed': 'Settlement batch processed',
  'subscription.charged': 'Subscription charge succeeded',
  'subscription.failed': 'Subscription charge failed',
  'kyc.approved': 'KYC verification approved',
  'kyc.rejected': 'KYC verification rejected',
  'kyc.resubmit_required': 'KYC needs resubmission',
};

export default function NotificationPreferences() {
  const [preferences, setPreferences] = useState<Record<string, NotificationPreference>>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showPerUser, setShowPerUser] = useState(false);
  const [profile, setProfile] = useState<{ phone?: string | null; email?: string | null }>({});
  const [notifications, setNotifications] = useState<NotificationItem[]>([]);
  const [notificationLoading, setNotificationLoading] = useState(true);
  const [notificationError, setNotificationError] = useState('');

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
    fetchNotifications();
  }, []);

  const fetchNotifications = async () => {
    setNotificationLoading(true);
    setNotificationError('');
    try {
      const data = await apiRequest<NotificationItem[]>('GET', '/v1/notifications?per_page=20', undefined, undefined, { cacheMs: 0 });
      setNotifications(data || []);
    } catch (err) {
      setNotificationError(err instanceof Error ? err.message : 'Failed to load notification alerts');
    } finally {
      setNotificationLoading(false);
    }
  };

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
      const profileRes = await apiRequest<{ phone?: string | null; email?: string | null }>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 45_000 });
      setProfile(profileRes);

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

  const markRead = async (id: string) => {
    await apiRequest('POST', `/v1/notifications/${id}/read`, {});
    await fetchNotifications();
  };

  const markAllRead = async () => {
    await apiRequest('POST', '/v1/notifications/mark-all-read', {});
    await fetchNotifications();
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
        <h1>Notifications</h1>
        <p>Review merchant alerts and configure delivery preferences</p>
      </div>

      <section className="card notification-inbox">
        <div className="notification-header">
          <h2><Bell size={18} /> Alerts</h2>
          <button className="btn-secondary" type="button" onClick={() => void markAllRead()} disabled={!notifications.some((item) => !item.is_read)}>
            <Check size={15} />
            Mark all read
          </button>
        </div>
        {notificationError && <div className="alert alert-error"><AlertCircle size={16} />{notificationError}</div>}
        {notificationLoading ? (
          <div className="loading">Loading notifications...</div>
        ) : notifications.length ? (
          <div className="notification-list">
            {notifications.map((item) => (
              <article key={item.id} className={`notification-item ${item.is_read ? '' : 'unread'}`}>
                <div>
                  <strong>{item.subject || item.entity_type || item.notification_type}</strong>
                  <p>{item.body || 'No message body provided.'}</p>
                  <span>{new Date(item.created_at).toLocaleString('en-IN')} • {item.status}</span>
                </div>
                {!item.is_read && (
                  <button className="btn-secondary" type="button" onClick={() => void markRead(item.id)}>Mark read</button>
                )}
              </article>
            ))}
          </div>
        ) : (
          <div className="empty-state">
            <Bell size={28} aria-hidden={true} />
            <strong>No notifications</strong>
            <p>KYC, settlement, webhook, payment issue, and API key alerts will appear here.</p>
          </div>
        )}
      </section>

      <div className="tabs">
        <button
          className={`tab ${!showPerUser ? 'active' : ''}`}
          onClick={() => setShowPerUser(false)}
        >
          My Preferences
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
                  <span className="event-description">{EVENT_COPY[eventType]}</span>
                </div>
                {CHANNELS.map(channel => (
                  <div key={`${eventType}-${channel}`} className="channel-column">
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={Boolean(preferences[eventType]?.[`${channel}_enabled`])}
                        disabled={(channel === 'sms' && !profile.phone) || (channel === 'email' && !profile.email)}
                        title={channel === 'sms' && !profile.phone ? 'Verify phone before enabling SMS' : channel === 'email' && !profile.email ? 'Verify email before enabling email notifications' : undefined}
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
        .notification-inbox {
          margin-bottom: 16px;
        }
        .notification-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
          margin-bottom: 14px;
        }
        .notification-header h2 {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          font-size: 16px;
          margin: 0;
        }
        .notification-list {
          display: grid;
          gap: 10px;
        }
        .notification-item {
          display: flex;
          justify-content: space-between;
          gap: 12px;
          border: 1px solid var(--fp-border);
          border-radius: 12px;
          padding: 12px;
          background: #fffdf5;
        }
        .notification-item.unread {
          border-color: rgba(235, 0, 27, 0.34);
          background: rgba(235, 0, 27, 0.06);
        }
        .notification-item p {
          margin: 4px 0;
          color: var(--fp-text-muted);
          font-size: 13px;
        }
        .notification-item span {
          color: var(--fp-text-muted);
          font-size: 12px;
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
        .event-description {
          display: block;
          color: var(--fp-text-muted);
          font-size: 12px;
          margin-top: 2px;
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
