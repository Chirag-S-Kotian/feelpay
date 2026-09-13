import React, { useState, useEffect } from 'react';
import { Mail, MessageSquare, Save, Plus, Edit2, Trash2, AlertCircle } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface NotificationTemplate {
  id: string;
  event_type: string;
  channel: 'email' | 'sms' | 'whatsapp';
  subject?: string;
  body: string;
  variables: string[];
  is_active: boolean;
}

interface TemplateFormData {
  event_type: string;
  channel: 'email' | 'sms' | 'whatsapp';
  subject?: string;
  body: string;
}

export const NotificationTemplateEditor: React.FC = () => {
  const [templates, setTemplates] = useState<NotificationTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<TemplateFormData>({
    event_type: 'payment_receipt',
    channel: 'email',
    subject: 'Payment Successful - FeelPay Receipt #{{transaction_id}}',
    body: 'Hi {{customer_name}},\n\nYour payment of {{amount}} has been successfully processed.\nTransaction ID: {{transaction_id}}\nDate: {{date}}\n\nThank you for choosing FeelPay.',
  });

  const eventTypes = [
    'payment_receipt',
    'payment_failed',
    'refund_initiated',
    'refund_success',
    'dispute_created',
    'settlement_processed',
    'kyc_approved',
    'kyc_rejected',
    'subscription_charged',
    'subscription_failed',
  ];

  const channels = ['email', 'sms', 'whatsapp'] as const;

  const variableExamples: Record<string, string[]> = {
    payment_receipt: ['customer_name', 'amount', 'transaction_id', 'date'],
    payment_failed: ['customer_name', 'amount', 'reason', 'transaction_id'],
    refund_initiated: ['customer_name', 'amount', 'transaction_id', 'reason'],
    settlement_processed: ['merchant_name', 'amount', 'utr', 'date'],
    kyc_approved: ['merchant_name', 'mcc_code'],
    kyc_rejected: ['merchant_name', 'reason', 'resubmit_url'],
  };

  useEffect(() => {
    fetchTemplates();
  }, []);

  const fetchTemplates = async () => {
    try {
      setLoading(true);
      const data = await apiRequest<{ data: NotificationTemplate[] }>(
        'GET',
        '/v1/admin/notification-templates'
      );
      setTemplates(data.data || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load templates');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveTemplate = async () => {
    try {
      setSaving(true);
      setError(null);

      const url = editingId
        ? `/v1/admin/notification-templates/${editingId}`
        : '/v1/admin/notification-templates';

      const method = editingId ? 'PUT' : 'POST';

      await apiRequest(method, url, formData);

      await fetchTemplates();
      setShowForm(false);
      setEditingId(null);
      setFormData({
        event_type: 'payment_receipt',
        channel: 'email',
        subject: 'Payment Successful - FeelPay Receipt #{{transaction_id}}',
        body: 'Hi {{customer_name}},\n\nYour payment of {{amount}} has been successfully processed.\nTransaction ID: {{transaction_id}}\nDate: {{date}}\n\nThank you for choosing FeelPay.',
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save template');
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteTemplate = async (id: string) => {
    if (!confirm('Are you sure you want to delete this template?')) return;

    try {
      await apiRequest('DELETE', `/v1/admin/notification-templates/${id}`);
      await fetchTemplates();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete template');
    }
  };

  const handleEditTemplate = (template: NotificationTemplate) => {
    setFormData({
      event_type: template.event_type,
      channel: template.channel,
      subject: template.subject,
      body: template.body,
    });
    setEditingId(template.id);
    setShowForm(true);
  };

  const getChannelIcon = (channel: string) => {
    switch (channel) {
      case 'email':
        return <Mail className="w-4 h-4" />;
      case 'sms':
        return <MessageSquare className="w-4 h-4" />;
      default:
        return <MessageSquare className="w-4 h-4" />;
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
    <div className="max-w-6xl mx-auto p-6">
      <div className="mb-8 flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Notification Templates</h1>
          <p className="text-gray-600 mt-2">Manage email, SMS, and WhatsApp templates</p>
        </div>
        <button
          onClick={() => {
            setShowForm(true);
            setEditingId(null);
            setFormData({
              event_type: 'payment_receipt',
              channel: 'email',
              subject: '',
              body: '',
            });
          }}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4" />
          New Template
        </button>
      </div>

      {error && (
        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
          <p className="text-red-700">{error}</p>
        </div>
      )}

      {showForm && (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            {editingId ? 'Edit Template' : 'Create Template'}
          </h2>

          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Event Type</label>
              <select
                value={formData.event_type}
                onChange={(e) => setFormData({ ...formData, event_type: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {eventTypes.map((type) => (
                  <option key={type} value={type}>
                    {type.replace(/_/g, ' ')}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Channel</label>
              <select
                value={formData.channel}
                onChange={(e) => setFormData({ ...formData, channel: e.target.value as any })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {channels.map((channel) => (
                  <option key={channel} value={channel}>
                    {channel.charAt(0).toUpperCase() + channel.slice(1)}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {formData.channel === 'email' && (
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">Subject</label>
              <input
                type="text"
                value={formData.subject || ''}
                onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
                placeholder="e.g., Payment Received: {{amount}}"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">Body</label>
            <textarea
              value={formData.body}
              onChange={(e) => setFormData({ ...formData, body: e.target.value })}
              placeholder="Use {{variable}} for dynamic content"
              rows={8}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
            />
            <p className="text-xs text-gray-600 mt-2">
              Available variables: {variableExamples[formData.event_type]?.join(', ') || 'customer_name, amount, transaction_id, date'}
            </p>
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleSaveTemplate}
              disabled={saving}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400"
            >
              <Save className="w-4 h-4" />
              {saving ? 'Saving...' : 'Save Template'}
            </button>
            <button
              onClick={() => setShowForm(false)}
              className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Templates List */}
      <div className="grid gap-4">
        {templates.map((template) => (
          <div key={template.id} className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  {getChannelIcon(template.channel)}
                  <h3 className="font-semibold text-gray-900">
                    {template.event_type.replace(/_/g, ' ')}
                  </h3>
                  <span className="text-xs bg-gray-100 text-gray-700 px-2 py-1 rounded">
                    {template.channel}
                  </span>
                  {template.is_active ? (
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">Active</span>
                  ) : (
                    <span className="text-xs bg-gray-100 text-gray-700 px-2 py-1 rounded">Inactive</span>
                  )}
                </div>
                {template.subject && (
                  <p className="text-sm text-gray-600 mb-2">
                    <strong>Subject:</strong> {template.subject}
                  </p>
                )}
                <p className="text-sm text-gray-600 line-clamp-2">{template.body}</p>
              </div>
              <div className="flex gap-2 ml-4">
                <button
                  onClick={() => handleEditTemplate(template)}
                  className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg"
                >
                  <Edit2 className="w-4 h-4" />
                </button>
                <button
                  onClick={() => handleDeleteTemplate(template.id)}
                  className="p-2 text-red-600 hover:bg-red-50 rounded-lg"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {templates.length === 0 && !showForm && (
        <div className="text-center py-12">
          <p className="text-gray-600">No templates yet. Create one to get started.</p>
        </div>
      )}
    </div>
  );
};

export default NotificationTemplateEditor;
