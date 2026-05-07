import { useState, useEffect } from 'react';
import { Users, Plus, Search, CreditCard, Mail, Phone, Download } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

interface Customer {
  id: string;
  customer_reference: string;
  customer_reference_type: string;
  email?: string;
  phone?: string;
  name?: string;
  metadata?: { notes?: string; lifetime_value?: number };
  created_at: string;
}

interface CustomersResponse {
  data: Customer[];
  meta: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

interface SavedPaymentMethod {
  id: string;
  method_type: string;
  is_default: boolean;
  card_brand?: string;
  card_last4?: string;
  card_expiry_month?: number;
  card_expiry_year?: number;
  vpa?: string;
  bank_name?: string;
  bank_account_last4?: string;
  created_at: string;
}

export default function Customers() {
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null);
  const [paymentMethods, setPaymentMethods] = useState<SavedPaymentMethod[]>([]);
  const [methodCounts, setMethodCounts] = useState<Record<string, number>>({});
  const [exporting, setExporting] = useState(false);
  const [exportFormat, setExportFormat] = useState<'csv' | 'excel'>('csv');
  const [createError, setCreateError] = useState('');

  const merchantId = (() => {
    const raw = localStorage.getItem('feelpay_merchant');
    if (!raw) return '';
    try {
      return (JSON.parse(raw) as { id?: string }).id || '';
    } catch {
      return '';
    }
  })();

  // Form state
  const [refType, setRefType] = useState('email');
  const [refValue, setRefValue] = useState('');
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [phone, setPhone] = useState('');
  const [notes, setNotes] = useState('');

  useEffect(() => {
    fetchCustomers();
  }, []);

  const fetchCustomers = async () => {
    try {
      const data = await apiRequest<CustomersResponse>('GET', '/v1/customers');
      setCustomers(data.data || []);
      const countEntries = await Promise.all((data.data || []).slice(0, 50).map(async (customer) => {
        try {
          const methods = await apiRequest<SavedPaymentMethod[]>('GET', `/v1/customers/${customer.id}/payment-methods`, undefined, undefined, { cacheMs: 30_000 });
          return [customer.id, methods.length] as const;
        } catch {
          return [customer.id, 0] as const;
        }
      }));
      setMethodCounts(Object.fromEntries(countEntries));
    } catch (err) {
      console.error('Failed to fetch customers:', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchPaymentMethods = async (customerId: string) => {
    try {
      const data = await apiRequest<SavedPaymentMethod[]>('GET', `/v1/customers/${customerId}/payment-methods`);
      setPaymentMethods(data || []);
    } catch (err) {
      console.error('Failed to fetch payment methods:', err);
    }
  };

  const createCustomer = async () => {
    setCreateError('');

    if (!refValue.trim()) {
      setCreateError('Reference value required');
      return;
    }

    if (refType === 'email' && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(refValue.trim())) {
      setCreateError('Enter valid email reference');
      return;
    }

    if (refType === 'phone' && !/^\+?[0-9\s-]{8,20}$/.test(refValue.trim())) {
      setCreateError('Enter valid phone reference');
      return;
    }
    if (email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim())) {
      setCreateError('Enter valid email');
      return;
    }
    if (phone.trim() && !/^\+?[0-9\s-]{8,20}$/.test(phone.trim())) {
      setCreateError('Enter valid phone');
      return;
    }

    try {
      await apiRequest('POST', '/v1/customers', {
        customer_reference: refValue.trim(),
        customer_reference_type: refType,
        name: name || undefined,
        email: email.trim() || undefined,
        phone: phone.trim() || undefined,
        metadata: { notes: notes.trim() || undefined },
      });

      setShowCreate(false);
      setRefValue('');
      setName('');
      setEmail('');
      setPhone('');
      setNotes('');
      setCreateError('');
      fetchCustomers();
    } catch (err) {
      console.error('Failed to create customer:', err);
      setCreateError(err instanceof Error ? err.message : 'Failed to create customer');
    }
  };

  const deleteCustomer = async (id: string) => {
    if (!confirm('Are you sure you want to delete this customer?')) return;

    try {
      await apiRequest('DELETE', `/v1/customers/${id}`);
      fetchCustomers();
    } catch (err) {
      console.error('Failed to delete customer:', err);
    }
  };

  const exportCustomers = async () => {
    if (!merchantId) return;
    setExporting(true);
    try {
      const res = await apiRequest<{ job_id: string }>('POST', `/v1/merchants/${merchantId}/exports/customers`, {
        format: exportFormat,
      });
      window.location.href = `/v1/merchants/${merchantId}/exports/${res.job_id}/file`;
    } catch (err) {
      console.error('Failed to export customers:', err);
      const header = ['name', 'email', 'phone', 'reference_type', 'reference', 'created_at'];
      const rows = customers.map((customer) => [
        customer.name || '',
        customer.email || '',
        customer.phone || '',
        customer.customer_reference_type,
        customer.customer_reference,
        customer.created_at,
      ]);
      const csv = [header, ...rows].map((row) => row.map((value) => `"${String(value).replaceAll('"', '""')}"`).join(',')).join('\n');
      const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }));
      const link = document.createElement('a');
      link.href = url;
      link.download = `feelpay-customers-${new Date().toISOString().slice(0, 10)}.csv`;
      link.click();
      URL.revokeObjectURL(url);
    } finally {
      setExporting(false);
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-IN', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  };

  const filteredCustomers = customers.filter((c) => {
    if (!search) return true;
    const searchLower = search.toLowerCase();
    return (
      c.customer_reference.toLowerCase().includes(searchLower) ||
      c.name?.toLowerCase().includes(searchLower) ||
      c.email?.toLowerCase().includes(searchLower) ||
      c.phone?.includes(search)
    );
  });

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Customers</h1>
          <p>Manage your customer database and saved payment methods</p>
        </div>
        <div className="customers-toolbar">
          <select
            className="form-input customers-toolbar-select"
            value={exportFormat}
            onChange={(e) => setExportFormat(e.target.value as 'csv' | 'excel')}
            aria-label="Export format"
          >
            <option value="csv">CSV</option>
            <option value="excel">Excel</option>
          </select>
          <button className="btn btn-secondary" onClick={exportCustomers} disabled={exporting || !merchantId}>
            <Download size={18} /> {exporting ? 'Exporting...' : 'Export'}
          </button>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Add Customer
          </button>
        </div>
      </div>

      {showCreate && (
        <div className="card customers-create-card">
          <div className="customers-create-header">
            <h3>Add Customer</h3>
            <p>Use strong reference values to dedupe and track returning buyers.</p>
          </div>
          {createError && <div className="customers-create-error">{createError}</div>}

          <div className="customers-form-grid">
            <div className="customers-field">
              <label className="form-label">Reference Type</label>
              <select className="form-input" value={refType} onChange={(e) => setRefType(e.target.value)}>
                <option value="email">Email</option>
                <option value="phone">Phone</option>
                <option value="external_id">External ID</option>
              </select>
            </div>
            <div className="customers-field">
              <label className="form-label">Reference Value</label>
              <input
                type="text"
                className="form-input"
                value={refValue}
                onChange={(e) => setRefValue(e.target.value)}
                placeholder={refType === 'email' ? 'customer@example.com' : refType === 'phone' ? '+91 98765 43210' : 'EXT-1001'}
              />
            </div>
            <div className="customers-field">
              <label className="form-label">Name (optional)</label>
              <input
                type="text"
                className="form-input"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Customer name"
              />
            </div>
            <div className="customers-field">
              <label className="form-label">Email (optional)</label>
              <input
                type="email"
                className="form-input"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="customer@example.com"
              />
            </div>
            <div className="customers-field customers-span-2">
              <label className="form-label">Phone (optional)</label>
              <input
                type="tel"
                className="form-input"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="+91 98765 43210"
              />
            </div>
            <div className="customers-field customers-span-2">
              <label className="form-label">Notes (optional)</label>
              <textarea
                className="form-input customers-notes"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder="Internal customer note"
              />
            </div>
          </div>

          <div className="customers-form-actions">
            <button className="btn btn-primary" onClick={createCustomer}>
              Add Customer
            </button>
            <button className="btn btn-secondary" onClick={() => setShowCreate(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="card customers-search-card">
        <div className="customers-search-wrap">
          <Search size={18} className="customers-search-icon" />
          <input
            type="text"
            className="form-input"
            placeholder="Search customers by reference, name, email, phone"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>

      {loading ? (
        <div className="card">Loading...</div>
      ) : filteredCustomers.length === 0 ? (
        <div className="card customers-empty-state">
          <div className="customers-empty-icon">
            <Users size={42} style={{ color: 'var(--fp-primary-soft)' }} />
          </div>
          <h3>No Customers Yet</h3>
          <p>Add your first customer to start tracking payment methods and repeat payments.</p>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Add Customer
          </button>
        </div>
      ) : (
        <div className="card customers-list-card">
          <div className="customers-list-header">
            <h3>Customer Records</h3>
            <div className="customers-list-stats">
              <span>{filteredCustomers.length} shown</span>
              <span>{customers.length} total</span>
            </div>
          </div>

          <table className="data-table">
            <thead>
              <tr>
                <th>Customer</th>
                <th>Contact</th>
                <th>Reference</th>
                <th>Created</th>
                <th>Methods</th>
                <th>LTV</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredCustomers.map((customer) => (
                <tr key={customer.id}>
                  <td>
                    <div className="customers-name">{customer.name || 'N/A'}</div>
                    <div className="customers-sub">ID: {customer.id.slice(0, 8)}...</div>
                  </td>
                  <td>
                    {customer.email && (
                      <div className="customers-contact-row">
                        <Mail size={14} className="customers-contact-ico" />
                        <span>{customer.email}</span>
                      </div>
                    )}
                    {customer.phone && (
                      <div className="customers-contact-row">
                        <Phone size={14} className="customers-contact-ico" />
                        <span>{customer.phone}</span>
                      </div>
                    )}
                    {!customer.email && !customer.phone && '-'}
                  </td>
                  <td>
                    <span className="badge">{customer.customer_reference_type}</span>
                    <div className="customers-sub" style={{ marginTop: 4 }}>{customer.customer_reference}</div>
                  </td>
                  <td className="customers-date">{formatDate(customer.created_at)}</td>
                  <td>{methodCounts[customer.id] ?? 0}</td>
                  <td>{new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format((customer.metadata?.lifetime_value || 0) / 100)}</td>
                  <td>
                    <div className="customers-action-row">
                      <button
                        className="btn btn-sm btn-secondary"
                        onClick={() => {
                          setSelectedCustomer(customer);
                          fetchPaymentMethods(customer.id);
                        }}
                      >
                        <CreditCard size={14} /> Payment Methods
                      </button>
                      <button className="btn btn-sm btn-danger" onClick={() => deleteCustomer(customer.id)}>
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selectedCustomer && (
        <div className="customers-modal-overlay" onClick={() => setSelectedCustomer(null)}>
          <div className="card customers-modal" onClick={(e) => e.stopPropagation()}>
            <h3 className="customers-modal-title">
              Payment Methods - {selectedCustomer.name || selectedCustomer.customer_reference}
            </h3>
            {paymentMethods.length === 0 ? (
              <p className="customers-modal-empty">No saved payment methods</p>
            ) : (
              <div className="customers-pm-list">
                {paymentMethods.map((pm) => (
                  <div key={pm.id} className="customers-pm-item">
                    <CreditCard size={24} style={{ color: 'var(--fp-primary)' }} />
                    <div style={{ flex: 1 }}>
                      {pm.method_type === 'card' && (
                        <>
                          <div className="customers-name">{pm.card_brand} •••• {pm.card_last4}</div>
                          <div className="customers-sub">Expires {pm.card_expiry_month}/{pm.card_expiry_year}</div>
                        </>
                      )}
                      {pm.method_type === 'upi' && (
                        <>
                          <div className="customers-name">UPI</div>
                          <div className="customers-sub">{pm.vpa}</div>
                        </>
                      )}
                      {pm.method_type === 'bank_account' && (
                        <>
                          <div className="customers-name">{pm.bank_name}</div>
                          <div className="customers-sub">•••• {pm.bank_account_last4}</div>
                        </>
                      )}
                    </div>
                    {pm.is_default && <span className="badge success">Default</span>}
                  </div>
                ))}
              </div>
            )}
            <button className="btn btn-secondary" style={{ marginTop: 16, width: '100%' }} onClick={() => setSelectedCustomer(null)}>
              Close
            </button>
          </div>
        </div>
      )}

      <style>{`
        .customers-toolbar {
          display: flex;
          gap: 12px;
          align-items: center;
          flex-wrap: wrap;
        }

        .customers-toolbar-select {
          width: 140px;
        }

        .customers-create-card {
          margin-bottom: 24px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
        }

        .customers-create-header h3 {
          font-size: 18px;
          margin-bottom: 6px;
        }

        .customers-create-header p {
          color: var(--fp-text-muted);
          margin-bottom: 16px;
          font-size: 14px;
        }

        .customers-create-error {
          margin: 0 0 14px;
          padding: 10px 12px;
          border-radius: 10px;
          font-size: 13px;
          border: 1px solid color-mix(in srgb, #ef4444 55%, transparent);
          background: rgba(200, 31, 47, 0.1);
          color: #991627;
        }

        .customers-form-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 16px;
        }

        .customers-field {
          display: flex;
          flex-direction: column;
          gap: 8px;
          min-width: 0;
        }

        .customers-field .form-label {
          margin: 0;
          font-size: 13px;
          font-weight: 600;
          color: var(--fp-text-muted);
        }

        .customers-field .form-input,
        .customers-toolbar-select,
        .customers-search-wrap .form-input {
          height: 42px;
          width: 100%;
          min-width: 0;
          padding: 0 12px;
          font-size: 14px;
          border-radius: 999px;
          border: 1px solid var(--fp-border);
          background: #fff9ec;
          color: var(--fp-text);
        }

        .customers-field .form-input::placeholder,
        .customers-search-wrap .form-input::placeholder {
          color: color-mix(in srgb, var(--fp-text-muted) 88%, #ffffff 12%);
        }

        .customers-field .form-input:focus,
        .customers-toolbar-select:focus,
        .customers-search-wrap .form-input:focus {
          border-color: var(--fp-cta);
          box-shadow: 0 0 0 3px rgba(235, 0, 27, 0.14);
        }

        .customers-field select.form-input,
        .customers-toolbar-select {
          appearance: none;
          background-image:
            linear-gradient(45deg, transparent 50%, #8a857d 50%),
            linear-gradient(135deg, #8a857d 50%, transparent 50%);
          background-position:
            calc(100% - 16px) calc(50% + 1px),
            calc(100% - 11px) calc(50% + 1px);
          background-size: 5px 5px, 5px 5px;
          background-repeat: no-repeat;
          padding-right: 30px;
        }

        .customers-span-2 {
          grid-column: span 2;
        }

        .customers-form-actions {
          display: flex;
          gap: 12px;
          margin-top: 20px;
        }

        .customers-search-card {
          margin-bottom: 24px;
          padding: 14px;
        }

        .customers-search-wrap {
          position: relative;
        }

        .customers-search-icon {
          position: absolute;
          left: 12px;
          top: 50%;
          transform: translateY(-50%);
          color: var(--fp-text-dim);
          pointer-events: none;
        }

        .customers-search-wrap .form-input {
          padding-left: 40px;
        }

        .customers-empty-state {
          text-align: center;
          padding: 56px 20px;
          background: #fff9ec;
        }

        .customers-empty-icon {
          width: 74px;
          height: 74px;
          margin: 0 auto 16px;
          border-radius: 20px;
          display: grid;
          place-items: center;
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .customers-empty-state h3 {
          margin-bottom: 8px;
        }

        .customers-empty-state p {
          color: var(--fp-text-muted);
          margin-bottom: 22px;
        }

        .customers-list-card {
          overflow: hidden;
        }

        .customers-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
          gap: 10px;
        }

        .customers-list-header h3 {
          margin: 0;
          font-size: 16px;
        }

        .customers-list-stats {
          display: flex;
          gap: 8px;
        }

        .customers-list-stats span {
          padding: 5px 10px;
          border-radius: 999px;
          font-size: 12px;
          color: var(--fp-text-muted);
          border: 1px solid var(--fp-border);
          background: #f2ead8;
        }

        .customers-name {
          font-weight: 600;
          font-size: 14px;
        }

        .customers-sub,
        .customers-date {
          font-size: 12px;
          color: var(--fp-text-muted);
        }

        .customers-contact-row {
          display: flex;
          align-items: center;
          gap: 6px;
          margin-bottom: 4px;
          font-size: 13px;
        }

        .customers-contact-ico {
          color: var(--fp-text-dim);
        }

        .customers-action-row {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }

        .customers-modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .customers-modal {
          width: 520px;
          max-height: 80vh;
          overflow: auto;
        }

        .customers-modal-title {
          font-size: 16px;
          font-weight: 600;
          margin-bottom: 20px;
        }

        .customers-modal-empty {
          color: var(--fp-text-muted);
          text-align: center;
          padding: 24px;
        }

        .customers-pm-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .customers-pm-item {
          padding: 12px;
          border: 1px solid var(--fp-border);
          border-radius: 8px;
          display: flex;
          align-items: center;
          gap: 12px;
        }

        @media (max-width: 960px) {
          .customers-form-grid {
            grid-template-columns: 1fr;
          }

          .customers-span-2 {
            grid-column: span 1;
          }

          .customers-list-header {
            flex-direction: column;
            align-items: flex-start;
          }

          .customers-modal {
            width: min(92vw, 520px);
          }
        }
      `}</style>
    </div>
  );
}
