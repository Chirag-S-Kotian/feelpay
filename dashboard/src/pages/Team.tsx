import { useEffect, useState } from 'react';
import { ShieldCheck, UserPlus } from 'lucide-react';
import { apiRequest, MerchantAuthError } from '../hooks/useApi';

type TeamMember = {
  id: string;
  email: string;
  full_name?: string | null;
  status: string;
  created_at: string;
  role: string;
};

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

function roleLabel(value: string) {
  return MERCHANT_ROLES.find((role) => role.value === value)?.label || value.replace(/_/g, ' ');
}

function SkeletonTable() {
  return (
    <div className="card team-table-card">
      <table className="data-table">
        <tbody>
          {[...Array(4)].map((_, i) => (
            <tr key={i}>
              <td><div className="skeleton" style={{ width: 140, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 180, height: 16 }} /></td>
              <td><div className="skeleton" style={{ width: 120, height: 32 }} /></td>
              <td><div className="skeleton" style={{ width: 80, height: 16 }} /></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function Team() {
  const [team, setTeam] = useState<TeamMember[]>([]);
  const [inviteName, setInviteName] = useState('');
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState('merchant_support');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');

  const loadTeam = async () => {
    setLoading(true);
    setError('');
    try {
      const response = await apiRequest<TeamMember[]>('GET', '/v1/merchants/team', undefined, undefined, { cacheMs: 0 });
      setTeam(response || []);
    } catch (err) {
      if (err instanceof MerchantAuthError) return;
      setError(err instanceof Error ? err.message : 'Unable to load team members');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadTeam();
  }, []);

  const inviteTeamMember = async () => {
    setSaving(true);
    setNotice('');
    setError('');
    try {
      await apiRequest('POST', '/v1/merchants/team/invite', {
        email: inviteEmail.trim(),
        full_name: inviteName.trim(),
        role: inviteRole,
      });
      setNotice('Invitation created.');
      setInviteName('');
      setInviteEmail('');
      setInviteRole('merchant_support');
      await loadTeam();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to invite team member');
    } finally {
      setSaving(false);
    }
  };

  const updateRole = async (userId: string, role: string) => {
    await apiRequest('PUT', `/v1/merchants/team/${userId}/role`, { role });
    setNotice('Team role updated.');
    await loadTeam();
  };

  const removeTeamMember = async (userId: string) => {
    await apiRequest('DELETE', `/v1/merchants/team/${userId}`);
    setNotice('Team member removed.');
    await loadTeam();
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1>Team</h1>
          <p>Invite users and assign merchant access roles</p>
        </div>
      </div>

      {notice && <div className="success-banner">{notice}</div>}
      {error && <div className="error-banner" role="alert">{error}</div>}

      <section className="card">
        <h3 className="section-title"><UserPlus size={17} /> Invite Member</h3>
        <div className="team-form-grid">
          <label className="input-group">
            <span>Member Name</span>
            <input value={inviteName} onChange={(event) => setInviteName(event.target.value)} placeholder="Jane Doe" />
          </label>
          <label className="input-group">
            <span>Member Email</span>
            <input value={inviteEmail} onChange={(event) => setInviteEmail(event.target.value)} placeholder="jane@company.com" />
          </label>
          <label className="input-group">
            <span>Role</span>
            <select value={inviteRole} onChange={(event) => setInviteRole(event.target.value)}>
              {MERCHANT_ROLES.map((role) => <option key={role.value} value={role.value}>{role.label}</option>)}
            </select>
          </label>
          <button className="btn-primary team-submit" type="button" onClick={() => void inviteTeamMember()} disabled={saving || !inviteEmail.trim() || !inviteName.trim()}>
            Invite
          </button>
        </div>
      </section>

      <section className="card team-table-section">
        <h3 className="section-title"><ShieldCheck size={17} /> Access Control</h3>
        {loading ? <SkeletonTable /> : (
          <div className="team-table-card">
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
                      <select aria-label={`Role for ${member.email}`} value={member.role} onChange={(event) => void updateRole(member.id, event.target.value)}>
                        {MERCHANT_ROLES.map((role) => <option key={role.value} value={role.value}>{role.label}</option>)}
                      </select>
                    </td>
                    <td>{member.status}</td>
                    <td><button className="btn-secondary" type="button" onClick={() => void removeTeamMember(member.id)}>Remove</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
            {team.length === 0 && (
              <div className="empty-state">
                <ShieldCheck size={28} aria-hidden={true} />
                <strong>No team members yet</strong>
                <p>Invite finance, support, developer, and read-only users as needed.</p>
              </div>
            )}
          </div>
        )}
        <div className="team-role-note">
          {MERCHANT_ROLES.slice(0, 6).map((role) => <span key={role.value}>{role.label}: {roleLabel(role.value)}</span>)}
        </div>
      </section>

      <style>{`
        .team-form-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
          gap: 14px;
          align-items: end;
        }
        .team-submit {
          min-height: 40px;
        }
        .team-table-section {
          margin-top: 16px;
        }
        .team-table-card {
          overflow-x: auto;
          -webkit-overflow-scrolling: touch;
        }
        .team-role-note {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-top: 14px;
          color: var(--fp-text-muted);
          font-size: 12px;
        }
        .team-role-note span {
          border: 1px solid var(--fp-border);
          border-radius: 999px;
          padding: 4px 8px;
          background: var(--fp-bg-elevated);
        }
      `}</style>
    </div>
  );
}
