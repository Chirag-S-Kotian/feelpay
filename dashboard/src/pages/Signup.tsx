import { useState, FormEvent } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Eye, EyeOff } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

export default function Signup() {
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    password: '',
    business_name: '',
    business_type: 'individual',
  });
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      await apiRequest('POST', '/v1/merchants/register', formData);
      setSuccess(true);
      setTimeout(() => navigate('/login'), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Registration failed');
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <div className="login-page">
        <div className="login-card">
          <div className="login-header">
            <h1>Account Created!</h1>
            <p>Welcome to FeelPay. Redirecting to login...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="auth-shell">
          <section className="auth-hero">
            <div>
              <p className="auth-kicker">Merchant Onboarding</p>
              <h1 className="auth-title">Launch checkout in minutes.</h1>
              <p className="auth-subtitle">
                Create your account, complete onboarding, and start testing with isolated credentials.
              </p>
            </div>
            <div className="auth-pills">
              <span className="auth-pill">Multi-Env Keys</span>
              <span className="auth-pill">Dispute Workflow</span>
              <span className="auth-pill">Bank Settlements</span>
            </div>
          </section>
          <section className="auth-form-wrap">
            <div className="login-header">
              <h1>FeelPay</h1>
              <p>Create your merchant account</p>
            </div>

            <form onSubmit={handleSubmit} className="login-form">
              {error && <div className="error-message">{error}</div>}
              
              <div className="form-group">
                <label>Full Name</label>
                <input 
                  type="text" 
                  value={formData.name}
                  onChange={(e) => setFormData({...formData, name: e.target.value})}
                  placeholder="John Doe"
                  required
                />
              </div>

              <div className="form-group">
                <label>Email Address</label>
                <input 
                  type="email" 
                  value={formData.email}
                  onChange={(e) => setFormData({...formData, email: e.target.value})}
                  placeholder="merchant@business.com"
                  required
                />
              </div>

              <div className="form-group">
                <label>Business Name</label>
                <input 
                  type="text" 
                  value={formData.business_name}
                  onChange={(e) => setFormData({...formData, business_name: e.target.value})}
                  placeholder="My Awesome Shop"
                  required
                />
              </div>

              <div className="form-group">
                <label>Password</label>
                <div className="password-input-wrapper">
                  <input 
                    type={showPassword ? "text" : "password"} 
                    value={formData.password}
                    onChange={(e) => setFormData({...formData, password: e.target.value})}
                    placeholder="••••••••"
                    required
                  />
                  <button 
                    type="button" 
                    className="password-toggle"
                    onClick={() => setShowPassword(!showPassword)}
                    tabIndex={-1}
                  >
                    {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
                  </button>
                </div>
              </div>

              <button type="submit" className="btn-primary btn-full" disabled={loading}>
                {loading ? 'Creating account...' : 'Get Started'}
              </button>
            </form>

            <div className="login-footer">
              <span>Already have an account?</span>
              <Link to="/login">Sign In</Link>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
