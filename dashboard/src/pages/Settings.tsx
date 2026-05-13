import { useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  Check,
  Copy,
  KeyRound,
  RefreshCw,
  ShieldCheck,
  Webhook,
  Zap,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';

interface MerchantProfile {
  id: string;
  name: string;
  email: string;
  business_name?: string | null;
  business_type?: string | null;
  webhook_url?: string | null;
  phone?: string | null;
  avatar_url?: string | null;
  daily_limit: number;
  monthly_limit: number;
  onboarding_status: string;
  status: string;
}

interface SecuritySettings {
  mfa_enabled: boolean;
  settlement_cycle: string;
  per_transaction_limit: number;
  daily_limit: number;
  monthly_limit: number;
  allowed_payment_methods: Record<string, unknown>;
  webhook_secret_hint: string;
  updated_at: string;
}

interface TeamMember {
  id: string;
  email: string;
  full_name?: string | null;
  status: string;
  created_at: string;
  role: string;
}

const MERCHANT_ROLES = [
  { value: 'merchant_owner', label: 'Owner' },
  { value: 'merchant_admin', label: 'Admin' },
  { value: 'merchant_developer', label: 'Developer' },
  { value: 'merchant_finance', label: 'Finance' },
  { value: 'merchant_support', label: 'Support' },
  { value: 'merchant_read_only', label: 'Read Only' },
  { value: 'merchant_staff', label: 'Staff (Legacy)' },
  { value: 'merchant_viewer', label: 'Viewer (Legacy)' },
];

interface HubLink {
  key: string;
  label: string;
  route: string;
  enabled: boolean;
}

interface HubSection {
  key: string;
  title: string;
  icon: string;
  links: HubLink[];
}

const FALLBACK_HUB_SECTIONS: HubSection[] = [
  {
    key: 'website_app',
    title: 'Website and App Settings',
    icon: 'monitor',
    links: [
      { key: 'api_keys', label: 'API keys', route: '/developers', enabled: true },
      { key: 'webhooks', label: 'Webhooks', route: '/webhooks', enabled: true },
      { key: 'checkout_preview', label: 'Checkout preview', route: '/checkout-preview', enabled: true },
    ],
  },
  {
    key: 'business',
    title: 'Business Settings',
    icon: 'briefcase',
    links: [
      { key: 'account_details', label: 'Account details', route: '/onboarding', enabled: true },
      { key: 'business_details', label: 'Business details', route: '/settings', enabled: true },
      { key: 'manage_team', label: 'Manage team', route: '/settings', enabled: true },
    ],
  },
  {
    key: 'payments',
    title: 'Payments and Refunds',
    icon: 'wallet',
    links: [
      { key: 'payments', label: 'Payments', route: '/payments', enabled: true },
      { key: 'settlements', label: 'Settlements', route: '/settlements', enabled: true },
      { key: 'disputes', label: 'Disputes', route: '/disputes', enabled: true },
    ],
  },
  {
    key: 'notifications',
    title: 'Notification Settings',
    icon: 'bell',
    links: [
      { key: 'notification_channels', label: 'Email / SMS / Alerts', route: '/notifications', enabled: true },
      { key: 'user_preferences', label: 'User preferences', route: '/notifications', enabled: true },
    ],
  },
];

const EVENTS = [
  'payment.success',
  'payment.failed',
  'refund.initiated',
  'settlement.processed',
  'dispute.created',
];

export default function Settings() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [name, setName] = useState('');
  const [businessName, setBusinessName] = useState('');
  const [businessType, setBusinessType] = useState('individual');
  const [phone, setPhone] = useState('');
  const [avatarUrl, setAvatarUrl] = useState('');
  const [uploadProgress, setUploadProgress] = useState(0);
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [webhookUrl, setWebhookUrl] = useState('');
  const [profile, setProfile] = useState<MerchantProfile | null>(null);
  const [security, setSecurity] = useState<SecuritySettings | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [notice, setNotice] = useState<string>('');
  const [warning, setWarning] = useState<string>('');
  const [team, setTeam] = useState<TeamMember[]>([]);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteName, setInviteName] = useState('');
  const [inviteRole, setInviteRole] = useState('merchant_support');
  const [hubSections, setHubSections] = useState<HubSection[]>([]);

  const merchantId = useMemo(() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  }, []);

  const loadAll = async () => {
    setLoading(true);
    try {
      const [profileRes, keysRes, securityResult, teamRes, hubRes] = await Promise.all([
        apiRequest<MerchantProfile>('GET', '/v1/merchants/profile', undefined, undefined, { cacheMs: 30_000 }),
        apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys', undefined, undefined, { cacheMs: 30_000 }),
        apiRequest<SecuritySettings>('GET', '/v1/merchants/settings/security', undefined, undefined, { cacheMs: 30_000 })
          .then((value) => ({ ok: true as const, value }))
          .catch((err: unknown) => ({ ok: false as const, err })),
        apiRequest<TeamMember[]>('GET', '/v1/merchants/team', undefined, undefined, { cacheMs: 20_000 }).catch(() => []),
        apiRequest<{ sections?: HubSection[] }>('GET', '/v1/merchants/account-settings-hub', undefined, undefined, { cacheMs: 60_000 })
          .then((r) => r.sections || [])
          .catch(() => [] as HubSection[]),
      ]);
      let securityRes: SecuritySettings;
      if (securityResult.ok) {
        securityRes = securityResult.value;
        setWarning('');
      } else {
        const msg = securityResult.err instanceof Error ? securityResult.err.message : '';
        if (!msg.includes('(404)')) {
          throw securityResult.err;
        }
        securityRes = {
          mfa_enabled: false,
          settlement_cycle: 'T+2',
          per_transaction_limit: 0,
          daily_limit: profileRes.daily_limit || 0,
          monthly_limit: profileRes.monthly_limit || 0,
          allowed_payment_methods: {},
          webhook_secret_hint: 'unsupported_on_current_backend',
          updated_at: new Date().toISOString(),
        };
        setWarning('Security endpoint unavailable on running backend. Restart/update API server to enable full controls.');
      }
      setProfile(profileRes);
      setSecurity(securityRes);
      setApiKey(keysRes.api_key || '');
      setName(profileRes.name || '');
      setBusinessName(profileRes.business_name || '');
      setBusinessType((profileRes.business_type || 'individual').toLowerCase());
      setPhone(profileRes.phone || '');
      setAvatarUrl(profileRes.avatar_url || '');
      setWebhookUrl(profileRes.webhook_url || '');
      setTeam(teamRes || []);
      setHubSections(hubRes || []);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadAll();
  }, []);

  const copy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 1600);
  };

  const rotateApiKeys = async () => {
    setNotice('');
    await apiRequest('POST', '/v1/merchants/keys/roll', {});
    const keys = await apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys');
    setApiKey(keys.api_key);
    setNotice('API keys rotated. Update server env immediately.');
  };

  const rotateWebhookSecret = async () => {
    setNotice('');
    setWarning('');
    try {
      await apiRequest('POST', '/v1/merchants/webhook-secret/roll', {});
      await loadAll();
      setNotice('Webhook secret rotated. Update signature verification key.');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '';
      if (msg.includes('(404)')) {
        setWarning('Webhook secret rotation route unavailable on running backend. Deploy/restart latest API build.');
        return;
      }
      setWarning(msg || 'Failed to rotate webhook secret.');
    }
  };

  const saveProfile = async () => {
    setSaving(true);
    setNotice('');
    setWarning('');
    try {
      if (!name.trim() || !businessName.trim()) {
        setWarning('Account name and business name are required.');
        return;
      }
      if (newPassword && newPassword !== confirmPassword) {
        setWarning('Confirm password does not match.');
        return;
      }
      await apiRequest('PUT', '/v1/merchants/profile', {
        name: name.trim(),
        business_name: businessName.trim(),
        business_type: businessType,
        webhook_url: webhookUrl.trim() || null,
        phone: phone.trim() || null,
        avatar_url: avatarUrl.trim() || null,
        current_password: newPassword ? currentPassword : undefined,
        new_password: newPassword || undefined,
        confirm_password: newPassword ? confirmPassword : undefined,
      });
      await loadAll();
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setNotice('Settings saved.');
    } finally {
      setSaving(false);
    }
  };

  const uploadAvatar = async (file: File | null) => {
    setWarning('');
    if (!file) return;
    const allowed = ['image/png', 'image/jpeg', 'image/webp'];
    if (!allowed.includes(file.type)) {
      setWarning('Avatar must be PNG, JPEG, or WEBP.');
      return;
    }
    if (file.size < 100 * 1024 || file.size > 5 * 1024 * 1024) {
      setWarning('Avatar must be between 100KB and 5MB.');
      return;
    }
    setUploadProgress(20);
    const dataBase64 = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error('Unable to read avatar file.'));
      reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '');
      reader.readAsDataURL(file);
    });
    setUploadProgress(60);
    const res = await apiRequest<{ avatar_url: string }>('POST', '/v1/merchants/profile/avatar-upload', {
      filename: file.name,
      content_type: file.type,
      data_base64: dataBase64,
    });
    setUploadProgress(100);
    setAvatarUrl(res.avatar_url);
    setTimeout(() => setUploadProgress(0), 800);
  };

  const sendTestWebhook = async () => {
    if (!merchantId) {
      setNotice('Merchant session missing. Re-login required.');
      return;
    }
    const normalizedWebhookUrl = webhookUrl.trim();
    if (!normalizedWebhookUrl) {
      setWarning('Set and save a webhook URL before sending a test event.');
      return;
    }
    if (!normalizedWebhookUrl.startsWith('https://')) {
      setWarning('Webhook URL must start with https:// for test delivery.');
      return;
    }
    setNotice('');
    try {
      await apiRequest('POST', `/v1/merchants/${merchantId}/webhooks/test`, {
        event_type: 'payment.success',
        webhook_url: normalizedWebhookUrl,
      });
      setNotice('Test webhook sent for payment.success.');
      setWarning('');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '';
      setWarning(msg || 'Failed to send test webhook.');
    }
  };

  const inviteTeamMember = async () => {
    if (!inviteEmail.trim() || !inviteName.trim()) {
      setWarning('Invite name + email required.');
      return;
    }
    setSaving(true);
    try {
      await apiRequest('POST', '/v1/merchants/team/invite', {
        email: inviteEmail.trim(),
        full_name: inviteName.trim(),
        role: inviteRole,
      });
      setInviteEmail('');
      setInviteName('');
      await loadAll();
      setNotice('Team member invited.');
      setWarning('');
    } finally {
      setSaving(false);
    }
  };

  const updateRole = async (userId: string, role: string) => {
    await apiRequest('PUT', `/v1/merchants/team/${userId}/role`, { role });
    await loadAll();
    setNotice('Team role updated.');
  };

  const removeTeamMember = async (userId: string) => {
    await apiRequest('DELETE', `/v1/merchants/team/${userId}`);
    await loadAll();
    setNotice('Team member removed.');
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1>Gateway Settings</h1>
          <p>Production controls for identity, API auth, webhooks, and operational safety.</p>
        </div>
      </div>

      {notice && (
        <div className="card" style={{ marginBottom: 16, borderColor: 'rgba(88, 174, 116, 0.42)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: '#97efb2' }}>
            <Check size={16} />
            <span>{notice}</span>
          </div>
        </div>
      )}
      {warning && (
        <div className="card" style={{ marginBottom: 16, borderColor: 'rgba(245, 158, 11, 0.42)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: '#f8c786' }}>
            <AlertTriangle size={16} />
            <span>{warning}</span>
          </div>
        </div>
      )}

      <div className="settings-grid">
        <form className="card" onSubmit={(e) => { e.preventDefault(); void saveProfile(); }}>
          <h3 className="section-title"><ShieldCheck size={17} /> Business Identity</h3>
          <div className="input-group">
            <label htmlFor="settings-avatar">Avatar</label>
            <div className="avatar-row">
              {avatarUrl ? <img src={avatarUrl} alt="" className="avatar-preview" /> : <div className="avatar-preview avatar-empty" />}
              <input id="settings-avatar" type="file" accept="image/png,image/jpeg,image/webp" onChange={(e) => void uploadAvatar(e.target.files?.[0] || null)} />
            </div>
            {uploadProgress > 0 && <progress value={uploadProgress} max={100} />}
          </div>
          <div className="input-group">
            <label htmlFor="settings-name">Owner / Account Name</label>
            <input id="settings-name" name="name" value={name} onChange={(e) => setName(e.target.value)} placeholder="Merchant account name" autoComplete="name" />
          </div>
          <div className="input-group">
            <label htmlFor="settings-business-name">Business Name</label>
            <input id="settings-business-name" name="organization" value={businessName} onChange={(e) => setBusinessName(e.target.value)} placeholder="Legal business name" autoComplete="organization" />
          </div>
          <div className="input-group">
            <label htmlFor="settings-business-type">Business Type</label>
            <select id="settings-business-type" value={businessType} onChange={(e) => setBusinessType(e.target.value)}>
              <option value="individual">Individual</option>
              <option value="sole_proprietorship">Sole Proprietorship</option>
              <option value="partnership">Partnership</option>
              <option value="corporation">Corporation</option>
              <option value="llc">LLC</option>
            </select>
          </div>
          <div className="input-group">
            <label htmlFor="settings-phone">Support Phone</label>
            <input id="settings-phone" name="tel" value={phone} onChange={(e) => setPhone(e.target.value)} placeholder="+919876543210" autoComplete="tel" />
          </div>
          <div className="input-group">
            <label htmlFor="settings-email">Email</label>
            <input id="settings-email" value={profile?.email || ''} readOnly autoComplete="email" />
          </div>
          <div className="password-grid">
            <input id="settings-current-password" name="current-password" type="password" value={currentPassword} onChange={(e) => setCurrentPassword(e.target.value)} placeholder="Current password" autoComplete="current-password" />
            <input id="settings-new-password" name="new-password" type="password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} placeholder="New password" autoComplete="new-password" />
            <input id="settings-confirm-password" name="confirm-password" type="password" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} placeholder="Confirm new password" autoComplete="new-password" />
          </div>
          <button type="submit" className="btn-primary" disabled={saving}>
            {saving ? 'Saving...' : 'Save Business Settings'}
          </button>
        </form>

        <section className="card">
          <h3 className="section-title"><KeyRound size={17} /> API Authentication</h3>
          <div className="input-group">
            <label>API Key</label>
            <div className="key-row">
              <input value={apiKey} readOnly style={{ fontFamily: 'Fira Code, monospace', fontSize: 12 }} />
              <button className="btn-secondary" onClick={() => copy(apiKey, 'api')}>
                {copied === 'api' ? <Check size={14} /> : <Copy size={14} />}
              </button>
            </div>
          </div>
          <div className="security-list">
            <div className="security-row">
              <span>MFA status</span>
              <strong>{security?.mfa_enabled ? 'Enabled' : 'Disabled'}</strong>
            </div>
            <div className="security-row">
              <span>Merchant status</span>
              <strong>{profile?.status || 'unknown'}</strong>
            </div>
            <div className="security-row">
              <span>Onboarding</span>
              <strong>{profile?.onboarding_status || 'unknown'}</strong>
            </div>
          </div>
          <button className="btn-secondary" onClick={() => void rotateApiKeys()}>
            <RefreshCw size={16} /> Rotate API Keys
          </button>
          <p className="mini-help">Rotation invalidates prior secret. Roll one deployment window, not random time.</p>
        </section>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 className="section-title">Account and Product Settings</h3>
        <div className="settings-hub-grid">
          {(hubSections.length ? hubSections : FALLBACK_HUB_SECTIONS).map((section) => (
            <div key={section.key} className="settings-hub-card">
              <h4>{section.title}</h4>
              {section.links.filter((link) => link.enabled).map((link) => (
                <button key={link.key} className="link-btn" onClick={() => navigate(link.route)}>
                  {link.label}
                </button>
              ))}
            </div>
          ))}
        </div>
      </div>

      <div className="settings-grid" style={{ marginTop: 16 }}>
        <section className="card">
          <h3 className="section-title"><Webhook size={17} /> Webhook Security</h3>
          <div className="input-group">
            <label>Webhook URL</label>
            <input
              type="url"
              value={webhookUrl}
              onChange={(e) => setWebhookUrl(e.target.value)}
              placeholder="https://merchant.example/webhooks/feelpay"
            />
          </div>
          <div className="input-group">
            <label>Webhook Secret Hint</label>
            <input value={security?.webhook_secret_hint || 'not_set'} readOnly />
          </div>
          <div className="webhook-events">
            {EVENTS.map((evt) => (
              <div key={evt} className="event-chip">{evt}</div>
            ))}
          </div>
          <div className="settings-actions">
            <button className="btn-primary" onClick={() => void saveProfile()} disabled={saving}>Save Webhook URL</button>
            <button className="btn-secondary" onClick={() => void rotateWebhookSecret()}>Rotate Webhook Secret</button>
            <button className="btn-secondary" onClick={() => void sendTestWebhook()}>Send Test Webhook</button>
          </div>
          <p className="mini-help">Enforce `https`. Verify `X-FeelPay-Signature` against raw body with HMAC-SHA256.</p>
        </section>

        <section className="card">
          <h3 className="section-title"><Zap size={17} /> Operational Limits</h3>
          <div className="ops-grid">
            <div className="ops-tile">
              <span>Per transaction</span>
              <strong>INR {security?.per_transaction_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Daily limit</span>
              <strong>INR {security?.daily_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Monthly limit</span>
              <strong>INR {security?.monthly_limit?.toLocaleString() || 0}</strong>
            </div>
            <div className="ops-tile">
              <span>Settlement cycle</span>
              <strong>{security?.settlement_cycle || 'T+2'}</strong>
            </div>
          </div>
          <div className="method-box">
            <label>Allowed Payment Methods JSON</label>
            <pre>{JSON.stringify(security?.allowed_payment_methods || {}, null, 2)}</pre>
          </div>
          <div className="risk-note">
            <AlertTriangle size={14} />
            <span>Limit updates require compliance approval. Use onboarding/ops workflow for changes.</span>
          </div>
        </section>
      </div>

      <div className="settings-grid" style={{ marginTop: 16 }}>
        <section className="card">
          <h3 className="section-title"><ShieldCheck size={17} /> Team & Access Control</h3>
          <div className="ops-grid">
            <div className="input-group" style={{ marginBottom: 0 }}>
              <label>Member Name</label>
              <input value={inviteName} onChange={(e) => setInviteName(e.target.value)} placeholder="Jane Doe" />
            </div>
            <div className="input-group" style={{ marginBottom: 0 }}>
              <label>Member Email</label>
              <input value={inviteEmail} onChange={(e) => setInviteEmail(e.target.value)} placeholder="jane@company.com" />
            </div>
            <div className="input-group" style={{ marginBottom: 0 }}>
              <label>Role</label>
              <select value={inviteRole} onChange={(e) => setInviteRole(e.target.value)}>
                {MERCHANT_ROLES.map((role) => (
                  <option key={role.value} value={role.value}>{role.label}</option>
                ))}
              </select>
            </div>
            <div style={{ display: 'flex', alignItems: 'end' }}>
              <button className="btn-primary" style={{ width: '100%' }} onClick={() => void inviteTeamMember()} disabled={saving}>
                Invite Team Member
              </button>
            </div>
          </div>

          <div className="error-table-wrap">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Email</th>
                  <th>Role</th>
                  <th>Status</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {team.map((member) => (
                  <tr key={member.id}>
                    <td>{member.full_name || '-'}</td>
                    <td>{member.email}</td>
                    <td>
                      <select value={member.role} onChange={(e) => void updateRole(member.id, e.target.value)}>
                        {MERCHANT_ROLES.map((role) => (
                          <option key={role.value} value={role.value}>{role.label}</option>
                        ))}
                      </select>
                    </td>
                    <td>{member.status}</td>
                    <td>
                      <button className="btn-secondary" onClick={() => void removeTeamMember(member.id)}>Remove</button>
                    </td>
                  </tr>
                ))}
                {team.length === 0 && (
                  <tr>
                    <td colSpan={5}>No team members yet.</td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </section>
      </div>

      <style>{`
        .settings-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(min(100%, 360px), 1fr));
          gap: 16px;
        }
        .settings-grid > .card {
          min-width: 0;
          overflow: hidden;
        }
        .section-title {
          font-size: 16px;
          margin-bottom: 14px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
        }
        .key-row {
          display: flex;
          gap: 8px;
          min-width: 0;
          align-items: stretch;
        }
        .key-row input {
          flex: 1 1 auto;
          min-width: 0;
        }
        .key-row .btn-secondary {
          min-width: 42px;
          display: inline-flex;
          justify-content: center;
          align-items: center;
        }
        .security-list {
          border: 1px solid rgba(103, 166, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          margin: 12px 0;
          background: rgba(14, 34, 63, 0.56);
        }
        .avatar-row {
          display: grid;
          grid-template-columns: 56px minmax(0, 1fr);
          gap: 10px;
          align-items: center;
        }
        .avatar-preview {
          width: 56px;
          height: 56px;
          object-fit: cover;
          border-radius: 50%;
          border: 1px solid rgba(103, 166, 255, 0.32);
          background: rgba(14, 34, 63, 0.72);
        }
        .avatar-empty::after {
          content: '';
          display: block;
          width: 18px;
          height: 18px;
          border-radius: 50%;
          margin: 18px;
          background: rgba(158, 197, 255, 0.55);
        }
        .password-grid {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 8px;
          margin-bottom: 12px;
        }
        .security-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 10px;
          padding: 7px 0;
          font-size: 13px;
          border-bottom: 1px solid rgba(100, 158, 245, 0.14);
        }
        .security-row strong {
          text-align: right;
          overflow-wrap: anywhere;
        }
        .security-row:last-child {
          border-bottom: none;
        }
        .webhook-events {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin: 10px 0 12px;
        }
        .event-chip {
          font-size: 12px;
          border: 1px solid rgba(99, 164, 253, 0.28);
          background: rgba(14, 33, 60, 0.72);
          border-radius: 999px;
          padding: 5px 9px;
          color: #c8ddfa;
        }
        .settings-actions {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }
        .ops-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 10px;
          margin-bottom: 12px;
        }
        .ops-tile {
          border: 1px solid rgba(98, 163, 253, 0.26);
          border-radius: 10px;
          padding: 10px;
          background: rgba(13, 34, 62, 0.6);
          display: flex;
          flex-direction: column;
          gap: 4px;
          font-size: 12px;
          color: #aac7ea;
        }
        .ops-tile strong {
          font-size: 14px;
          color: #eef5ff;
        }
        .method-box label {
          font-size: 12px;
          color: #b6cff0;
          display: block;
          margin-bottom: 6px;
        }
        .method-box pre {
          margin: 0;
          background: rgba(9, 22, 42, 0.84);
          border: 1px solid rgba(102, 164, 255, 0.22);
          border-radius: 10px;
          padding: 10px;
          max-height: 170px;
          overflow: auto;
          max-width: 100%;
          color: #d2e6ff;
          font-size: 12px;
        }
        .risk-note {
          margin-top: 10px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: #f7c98c;
          font-size: 12px;
          border: 1px solid rgba(245, 158, 11, 0.28);
          background: rgba(245, 158, 11, 0.1);
          border-radius: 8px;
          padding: 8px 10px;
        }
        .mini-help {
          margin-top: 8px;
          font-size: 12px;
          color: #9ebbe0;
        }
        .error-table-wrap {
          overflow-x: auto;
          max-width: 100%;
        }
        .settings-hub-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
          gap: 12px;
          margin-top: 8px;
        }
        .settings-hub-card {
          border: 1px solid #d5c9af;
          border-radius: 12px;
          padding: 12px;
          background: #f2ead8;
          display: grid;
          gap: 8px;
        }
        .settings-hub-card h4 {
          font-size: 14px;
          margin: 0 0 2px 0;
          color: #141413;
        }
        .link-btn {
          text-align: left;
          border: none;
          background: transparent;
          color: #2d5ea8;
          font-size: 14px;
          cursor: pointer;
          padding: 0;
        }
        .link-btn:hover {
          color: #193d73;
          text-decoration: underline;
        }
        @media (max-width: 640px) {
          .settings-grid,
          .ops-grid {
            grid-template-columns: 1fr;
          }
          .key-row,
          .settings-actions,
          .risk-note,
          .security-row {
            display: grid;
            grid-template-columns: 1fr;
            align-items: stretch;
          }
          .data-table {
            min-width: 720px;
          }
        }
      `}</style>
    </div>
  );
}
