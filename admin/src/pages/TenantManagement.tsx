import { useEffect, useState } from 'react';
import { apiRequest } from '../hooks/useApi';

type Tenant = {
  id: string;
  name: string;
  slug: string;
  status: string;
  domain?: string | null;
  branding?: Record<string, unknown> | null;
  config?: Record<string, unknown> | null;
};

type TenantMerchant = {
  id: string;
  name: string;
  email: string;
  business_name?: string | null;
  status: string;
};

const emptyTenantForm = {
  name: '',
  slug: '',
  status: 'active',
  domain: '',
  logoUrl: '',
  primaryColor: '#2563eb',
  secondaryColor: '#0f172a',
  fontFamily: 'Inter',
  faviconUrl: '',
  customDomain: '',
  emailDomain: '',
  feeBps: '200',
  settlementDelayDays: '2',
  resellerCommissionBps: '0',
  revenueSharePercent: '0',
};

export default function TenantManagement() {
  const [rows, setRows] = useState<Tenant[]>([]);
  const [form, setForm] = useState(emptyTenantForm);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [selectedTenantId, setSelectedTenantId] = useState<string | null>(null);
  const [tenantMerchants, setTenantMerchants] = useState<TenantMerchant[]>([]);

  const load = () => apiRequest<Tenant[]>('GET', '/v1/admin/tenants').then(setRows).catch(() => setRows([]));

  useEffect(() => {
    void load();
  }, []);

  useEffect(() => {
    if (!selectedTenantId) {
      setTenantMerchants([]);
      return;
    }
    void apiRequest<TenantMerchant[]>('GET', `/v1/admin/tenants/${selectedTenantId}/merchants`)
      .then(setTenantMerchants)
      .catch(() => setTenantMerchants([]));
  }, [selectedTenantId]);

  const payload = {
    name: form.name,
    slug: form.slug,
    domain: form.domain || null,
    status: form.status,
    branding: {
      logo_url: form.logoUrl || null,
      primary_color: form.primaryColor || null,
      secondary_color: form.secondaryColor || null,
      font_family: form.fontFamily || null,
      favicon_url: form.faviconUrl || null,
      hosted_checkout_theme: {
        primary_color: form.primaryColor || null,
        secondary_color: form.secondaryColor || null,
        font_family: form.fontFamily || null,
      },
    },
    config: {
      custom_domain: form.customDomain || null,
      wildcard_ssl_enabled: true,
      email_sending_domain: form.emailDomain || null,
      fee_bps: Number(form.feeBps || 0),
      settlement_delay_days: Number(form.settlementDelayDays || 0),
      reseller_commission_bps: Number(form.resellerCommissionBps || 0),
      revenue_share_percent: Number(form.revenueSharePercent || 0),
    },
  };

  function setFormFromTenant(tenant: Tenant) {
    const branding = (tenant.branding ?? {}) as Record<string, unknown>;
    const config = (tenant.config ?? {}) as Record<string, unknown>;
    setForm({
      name: tenant.name,
      slug: tenant.slug,
      status: tenant.status,
      domain: tenant.domain ?? '',
      logoUrl: String(branding.logo_url ?? ''),
      primaryColor: String(branding.primary_color ?? '#2563eb'),
      secondaryColor: String(branding.secondary_color ?? '#0f172a'),
      fontFamily: String(branding.font_family ?? 'Inter'),
      faviconUrl: String(branding.favicon_url ?? ''),
      customDomain: String(config.custom_domain ?? ''),
      emailDomain: String(config.email_sending_domain ?? ''),
      feeBps: String(config.fee_bps ?? '200'),
      settlementDelayDays: String(config.settlement_delay_days ?? '2'),
      resellerCommissionBps: String(config.reseller_commission_bps ?? '0'),
      revenueSharePercent: String(config.revenue_share_percent ?? '0'),
    });
  }

  async function submit() {
    if (editingId) {
      await apiRequest('PUT', `/v1/admin/tenants/${editingId}`, payload);
    } else {
      await apiRequest('POST', '/v1/admin/tenants', payload);
    }
    setForm(emptyTenantForm);
    setEditingId(null);
    load();
  }

  async function deactivate(id: string) {
    await apiRequest('DELETE', `/v1/admin/tenants/${id}`);
    if (selectedTenantId === id) {
      setSelectedTenantId(null);
    }
    load();
  }

  return (
    <section className="page-shell">
      <header className="page-header">
        <h1>Tenant Management</h1>
        <button onClick={submit}>{editingId ? 'Update' : 'Create'}</button>
      </header>
      <div className="filter-grid">
        <input placeholder="Name" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
        <input placeholder="Slug" value={form.slug} onChange={e => setForm({ ...form, slug: e.target.value })} />
        <input placeholder="Tenant domain" value={form.domain} onChange={e => setForm({ ...form, domain: e.target.value })} />
        <input placeholder="Custom checkout domain" value={form.customDomain} onChange={e => setForm({ ...form, customDomain: e.target.value })} />
        <input placeholder="Email sending domain" value={form.emailDomain} onChange={e => setForm({ ...form, emailDomain: e.target.value })} />
        <input placeholder="Logo URL" value={form.logoUrl} onChange={e => setForm({ ...form, logoUrl: e.target.value })} />
        <input placeholder="Favicon URL" value={form.faviconUrl} onChange={e => setForm({ ...form, faviconUrl: e.target.value })} />
        <input placeholder="Font family" value={form.fontFamily} onChange={e => setForm({ ...form, fontFamily: e.target.value })} />
        <input placeholder="Primary color (#hex)" value={form.primaryColor} onChange={e => setForm({ ...form, primaryColor: e.target.value })} />
        <input placeholder="Secondary color (#hex)" value={form.secondaryColor} onChange={e => setForm({ ...form, secondaryColor: e.target.value })} />
        <input placeholder="Fee (bps)" value={form.feeBps} onChange={e => setForm({ ...form, feeBps: e.target.value })} />
        <input placeholder="Settlement delay days" value={form.settlementDelayDays} onChange={e => setForm({ ...form, settlementDelayDays: e.target.value })} />
        <input placeholder="Reseller commission (bps)" value={form.resellerCommissionBps} onChange={e => setForm({ ...form, resellerCommissionBps: e.target.value })} />
        <input placeholder="Revenue share (%)" value={form.revenueSharePercent} onChange={e => setForm({ ...form, revenueSharePercent: e.target.value })} />
      </div>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Slug</th>
              <th>Domain</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map(r => (
              <tr key={r.id}>
                <td>{r.name}</td>
                <td>{r.slug}</td>
                <td>{r.domain ?? '-'}</td>
                <td>{r.status}</td>
                <td>
                  <button onClick={() => { setEditingId(r.id); setFormFromTenant(r); }}>Edit</button>
                  <button onClick={() => setSelectedTenantId(r.id)}>Business Accounts</button>
                  <button onClick={() => deactivate(r.id)}>Deactivate</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {selectedTenantId && (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Tenant Business Account</th>
                <th>Email</th>
                <th>Business</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {tenantMerchants.map(m => (
                <tr key={m.id}>
                  <td>{m.name}</td>
                  <td>{m.email}</td>
                  <td>{m.business_name ?? '-'}</td>
                  <td>{m.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
