import React, { useState, useEffect } from 'react';
import { Bell, Mail, MessageSquare, Save, AlertCircle } from 'lucide-react';

interface NotificationSetting {
  event_type: string;
  email_enabled: boolean;
  sms_enabled: boolean;
  in_app_enabled: boolean;
}

interface MerchantSettings {
  send_customer_receipt_email: boolean;
  send_customer_receipt_sms: boolean;
}

export const NotificationSettings: React.FC = () => {
  const [settings, setSettings] = useState<NotificationSetting[]>([]);
  const [merchantSettings, setMerchantSettings] = useState<MerchantSettings>({
    send_customer_receipt_email: true,
    send_customer_receipt_sms: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    fetchSettings();
  }, []);

  const fetchSettings = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/v1/notification-preferences', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('token')}` },
      });
      if (!response.ok) throw new Error('Failed to fetch settings');
      const data = await response.json();
      setSettings(data.data || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const handleSettingChange = (eventType: string, channel: 'email' | 'sms' | 'in_app', value: boolean) => {
    setSettings(settings.map(s =>
      s.event_type === eventType
        ? { ...s, [`${channel}_enabled`]: value }
        : s
    ));
  };

  const handleMerchantSettingChange = (key: keyof MerchantSettings, value: boolean) => {
    setMerchantSettings({ ...merchantSettings, [key]: value });
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      setError(null);

      // Save notification preferences
      const prefsResponse = await fetch('/api/v1/notification-preferences', {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ preferences: settings }),
      });

      if (!prefsResponse.ok) throw new Error('Failed to save preferences');

      // Save merchant settings
      const merchantResponse = await fetch('/api/v1/merchant-settings', {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(merchantSettings),
      });

      if (!merchantResponse.ok) throw new Error('Failed to save merchant settings');

      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-2">
          <Bell className="w-8 h-8" />
          Notification Settings
        </h1>
        <p className="text-gray-600 mt-2">Manage how you receive notifications about your account</p>
      </div>

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
          <div>
            <h3 className="font-semibold text-red-900">Error</h3>
            <p className="text-red-700 text-sm">{error}</p>
          </div>
        </div>
      )}

      {success && (
        <div className="mb-6 p-4 bg-green-50 border border-green-200 rounded-lg">
          <p className="text-green-700 font-semibold">✓ Settings saved successfully</p>
        </div>
      )}

      {/* Customer Receipt Settings */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Customer Receipts</h2>
        <div className="space-y-4">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={merchantSettings.send_customer_receipt_email}
              onChange={(e) => handleMerchantSettingChange('send_customer_receipt_email', e.target.checked)}
              className="w-4 h-4 rounded border-gray-300"
            />
            <div>
              <p className="font-medium text-gray-900 flex items-center gap-2">
                <Mail className="w-4 h-4" />
                Send payment receipts via email
              </p>
              <p className="text-sm text-gray-600">Customers receive email confirmation after payment</p>
            </div>
          </label>

          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={merchantSettings.send_customer_receipt_sms}
              onChange={(e) => handleMerchantSettingChange('send_customer_receipt_sms', e.target.checked)}
              className="w-4 h-4 rounded border-gray-300"
            />
            <div>
              <p className="font-medium text-gray-900 flex items-center gap-2">
                <MessageSquare className="w-4 h-4" />
                Send payment receipts via SMS
              </p>
              <p className="text-sm text-gray-600">Customers receive SMS confirmation after payment</p>
            </div>
          </label>
        </div>
      </div>

      {/* Event-Based Notifications */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Event Notifications</h2>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left py-3 px-4 font-semibold text-gray-900">Event</th>
                <th className="text-center py-3 px-4 font-semibold text-gray-900">
                  <Mail className="w-4 h-4 inline" /> Email
                </th>
                <th className="text-center py-3 px-4 font-semibold text-gray-900">
                  <MessageSquare className="w-4 h-4 inline" /> SMS
                </th>
                <th className="text-center py-3 px-4 font-semibold text-gray-900">
                  <Bell className="w-4 h-4 inline" /> In-App
                </th>
              </tr>
            </thead>
            <tbody>
              {settings.map((setting) => (
                <tr key={setting.event_type} className="border-b border-gray-100 hover:bg-gray-50">
                  <td className="py-3 px-4 text-gray-900 font-medium capitalize">
                    {setting.event_type.replace(/_/g, ' ')}
                  </td>
                  <td className="py-3 px-4 text-center">
                    <input
                      type="checkbox"
                      checked={setting.email_enabled}
                      onChange={(e) => handleSettingChange(setting.event_type, 'email', e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300"
                    />
                  </td>
                  <td className="py-3 px-4 text-center">
                    <input
                      type="checkbox"
                      checked={setting.sms_enabled}
                      onChange={(e) => handleSettingChange(setting.event_type, 'sms', e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300"
                    />
                  </td>
                  <td className="py-3 px-4 text-center">
                    <input
                      type="checkbox"
                      checked={setting.in_app_enabled}
                      onChange={(e) => handleSettingChange(setting.event_type, 'in_app', e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300"
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Save Button */}
      <div className="mt-6 flex justify-end">
        <button
          onClick={handleSave}
          disabled={saving}
          className="flex items-center gap-2 px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 transition-colors"
        >
          <Save className="w-4 h-4" />
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
};

export default NotificationSettings;
