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
  const [exporting, setExporting] = useState(false);
  const [exportFormat, setExportFormat] = useState<'csv' | 'excel'>('csv');
  
  const merchantId = localStorage.getItem('feelpay_merchant_id') || 'test-merchant';
  
  // Form state
  const [refType, setRefType] = useState('email');
  const [refValue, setRefValue] = useState('');
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [phone, setPhone] = useState('');

  useEffect(() => {
    fetchCustomers();
  }, []);

  const fetchCustomers = async () => {
    try {
      const data = await apiRequest<CustomersResponse>('GET', '/v1/customers');
      setCustomers(data.data || []);
    } catch (err) {
      console.error('Failed to fetch customers:', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchPaymentMethods = async (customerId: string) => {
    try {
      const data = await apiRequest<SavedPaymentMethod[]>(
        'GET',
        `/v1/customers/${customerId}/payment-methods`
      );
      setPaymentMethods(data || []);
    } catch (err) {
      console.error('Failed to fetch payment methods:', err);
    }
  };

  const createCustomer = async () => {
    if (!refValue) return;
    
    try {
      await apiRequest('POST', '/v1/customers', {
        customer_reference: refValue,
        customer_reference_type: refType,
        name: name || undefined,
        email: email || undefined,
        phone: phone || undefined,
      });
      
      setShowCreate(false);
      setRefValue('');
      setName('');
      setEmail('');
      setPhone('');
      fetchCustomers();
    } catch (err) {
      console.error('Failed to create customer:', err);
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
    setExporting(true);
    try {
      const res = await apiRequest<{ job_id: string }>(
        'POST',
        `/api/v1/merchants/${merchantId}/exports/customers`,
        { format: exportFormat }
      );
      window.location.href = `/api/v1/merchants/${merchantId}/exports/${res.job_id}/file`;
    } catch (err) {
      console.error('Failed to export customers:', err);
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
        <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
          <select
            className="form-input"
            value={exportFormat}
            onChange={e => setExportFormat(e.target.value as 'csv' | 'excel')}
            aria-label="Export format"
            style={{ width: 140 }}
          >
            <option value="csv">CSV</option>
            <option value="excel">Excel</option>
          </select>
          <button className="btn btn-secondary" onClick={exportCustomers} disabled={exporting}>
            <Download size={18} /> {exporting ? 'Exporting...' : 'Export'}
          </button>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Add Customer
          </button>
        </div>
      </div>

      {showCreate && (
        <div className="card" style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>Add Customer</h3>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 16 }}>
            <div>
              <label className="form-label">Reference Type</label>
              <select
                className="form-input"
                value={refType}
                onChange={(e) => setRefType(e.target.value)}
              >
                <option value="email">Email</option>
                <option value="phone">Phone</option>
                <option value="external_id">External ID</option>
              </select>
            </div>
            <div>
              <label className="form-label">Reference Value</label>
              <input
                type="text"
                className="form-input"
                value={refValue}
                onChange={(e) => setRefValue(e.target.value)}
                placeholder={refType === 'email' ? 'customer@example.com' : refType === 'phone' ? '+91...' : 'External ID'}
              />
            </div>
            <div>
              <label className="form-label">Name (optional)</label>
              <input
                type="text"
                className="form-input"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Customer name"
              />
            </div>
            <div>
              <label className="form-label">Email (optional)</label>
              <input
                type="email"
                className="form-input"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="customer@example.com"
              />
            </div>
            <div>
              <label className="form-label">Phone (optional)</label>
              <input
                type="tel"
                className="form-input"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="+91 98765 43210"
              />
            </div>
          </div>
          <div style={{ display: 'flex', gap: 12, marginTop: 20 }}>
            <button className="btn btn-primary" onClick={createCustomer}>
              Add Customer
            </button>
            <button className="btn btn-secondary" onClick={() => setShowCreate(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="card" style={{ marginBottom: 24 }}>
        <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
          <div style={{ position: 'relative', flex: 1 }}>
            <Search
              size={18}
              style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)', color: 'var(--fp-text-dim)' }}
            />
            <input
              type="text"
              className="form-input"
              style={{ paddingLeft: 40 }}
              placeholder="Search customers..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>
      </div>

      {loading ? (
        <div className="card">Loading...</div>
      ) : filteredCustomers.length === 0 ? (
        <div className="card" style={{ textAlign: 'center', padding: 48 }}>
          <Users size={48} style={{ color: 'var(--fp-text-dim)', marginBottom: 16 }} />
          <h3 style={{ marginBottom: 8 }}>No Customers Yet</h3>
          <p style={{ color: 'var(--fp-text-muted)', marginBottom: 24 }}>
            Add your first customer to start tracking their payment methods
          </p>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>
            <Plus size={18} /> Add Customer
          </button>
        </div>
      ) : (
        <div className="card">
          <table className="data-table">
            <thead>
              <tr>
                <th>Customer</th>
                <th>Contact</th>
                <th>Reference</th>
                <th>Created</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredCustomers.map((customer) => (
                <tr key={customer.id}>
                  <td>
                    <div style={{ fontWeight: 500 }}>{customer.name || 'N/A'}</div>
                    <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>
                      ID: {customer.id.slice(0, 8)}...
                    </div>
                  </td>
                  <td>
                    {customer.email && (
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
                        <Mail size={14} style={{ color: 'var(--fp-text-dim)' }} />
                        <span style={{ fontSize: 13 }}>{customer.email}</span>
                      </div>
                    )}
                    {customer.phone && (
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                        <Phone size={14} style={{ color: 'var(--fp-text-dim)' }} />
                        <span style={{ fontSize: 13 }}>{customer.phone}</span>
                      </div>
                    )}
                    {!customer.email && !customer.phone && '-'}
                  </td>
                  <td>
                    <span className="badge">{customer.customer_reference_type}</span>
                    <div style={{ fontSize: 12, marginTop: 4 }}>{customer.customer_reference}</div>
                  </td>
                  <td style={{ fontSize: 13, color: 'var(--fp-text-muted)' }}>
                    {formatDate(customer.created_at)}
                  </td>
                  <td>
                    <div style={{ display: 'flex', gap: 8 }}>
                      <button
                        className="btn btn-sm btn-secondary"
                        onClick={() => {
                          setSelectedCustomer(customer);
                          fetchPaymentMethods(customer.id);
                        }}
                      >
                        <CreditCard size={14} /> Payment Methods
                      </button>
                      <button
                        className="btn btn-sm btn-danger"
                        onClick={() => deleteCustomer(customer.id)}
                      >
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

      {/* Payment Methods Modal */}
      {selectedCustomer && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setSelectedCustomer(null)}
        >
          <div
            className="card"
            style={{ width: 500, maxHeight: '80vh', overflow: 'auto' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 20 }}>
              Payment Methods - {selectedCustomer.name || selectedCustomer.customer_reference}
            </h3>
            {paymentMethods.length === 0 ? (
              <p style={{ color: 'var(--fp-text-muted)', textAlign: 'center', padding: 24 }}>
                No saved payment methods
              </p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                {paymentMethods.map((pm) => (
                  <div
                    key={pm.id}
                    style={{
                      padding: 12,
                      border: '1px solid var(--fp-border)',
                      borderRadius: 8,
                      display: 'flex',
                      alignItems: 'center',
                      gap: 12,
                    }}
                  >
                    <CreditCard size={24} style={{ color: 'var(--fp-primary)' }} />
                    <div style={{ flex: 1 }}>
                      {pm.method_type === 'card' && (
                        <>
                          <div style={{ fontWeight: 500 }}>
                            {pm.card_brand} •••• {pm.card_last4}
                          </div>
                          <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>
                            Expires {pm.card_expiry_month}/{pm.card_expiry_year}
                          </div>
                        </>
                      )}
                      {pm.method_type === 'upi' && (
                        <>
                          <div style={{ fontWeight: 500 }}>UPI</div>
                          <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>{pm.vpa}</div>
                        </>
                      )}
                      {pm.method_type === 'bank_account' && (
                        <>
                          <div style={{ fontWeight: 500 }}>{pm.bank_name}</div>
                          <div style={{ fontSize: 12, color: 'var(--fp-text-muted)' }}>
                            •••• {pm.bank_account_last4}
                          </div>
                        </>
                      )}
                    </div>
                    {pm.is_default && (
                      <span className="badge success">Default</span>
                    )}
                  </div>
                ))}
              </div>
            )}
            <button
              className="btn btn-secondary"
              style={{ marginTop: 16, width: '100%' }}
              onClick={() => setSelectedCustomer(null)}
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}