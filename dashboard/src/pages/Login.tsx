import { useState, FormEvent, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Eye, EyeOff } from 'lucide-react';
import { apiRequest, setSessionExpiryFromSeconds } from '../hooks/useApi';

export default function Login() {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const impersonateToken = params.get('token');
    const reason = params.get('reason');

    if (reason === 'expired' || reason === 'invalid') setError('Session expired. Please sign in again.');
    if (reason === 'logout') setError('You have been logged out.');

    if (impersonateToken) {
      const handleImpersonate = async () => {
        setLoading(true);
        setError('');
        try {
          // Admin server already generated a token, but we need merchant metadata
          // We can fetch profile with this token
          const res = await apiRequest<{
            id: string;
            name: string;
            email: string;
            api_key: string;
          }>('GET', '/v1/merchants/profile', null, impersonateToken);

          localStorage.setItem('feelpay_token', impersonateToken);
          setSessionExpiryFromSeconds(86_400);
          localStorage.setItem(
            'feelpay_merchant',
            JSON.stringify({
              id: res.id,
              name: res.name,
              email: res.email,
              api_key: res.api_key,
            }),
          );

          navigate('/');
        } catch (err) {
          setError('Impersonation failed: Session invalid or expired');
        } finally {
          setLoading(false);
        }
      };
      handleImpersonate();
    }
  }, [navigate]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const res = await apiRequest<{
        token: string;
        merchant_id: string;
        name: string;
        email?: string;
        api_key: string;
        expires_in?: number;
      }>('POST', '/v1/merchants/login', { email, password });

      localStorage.setItem('feelpay_token', res.token);
      setSessionExpiryFromSeconds(res.expires_in || 86_400);
      localStorage.setItem(
        'feelpay_merchant',
        JSON.stringify({
          id: res.merchant_id,
          name: res.name,
          email: res.email || email,
          api_key: res.api_key,
        }),
      );

      navigate('/');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="login-page">
      <div className="login-card">
        <div className="auth-shell">
          <section className="auth-hero">
            <div>
              <img src="/feelpay.png" alt="FeelPay" className="auth-logo" />
              <p className="auth-kicker">Developer Platform</p>
              <h1 className="auth-title">Build, test, ship payments fast.</h1>
              <p className="auth-subtitle">
                Move from sandbox to live with one flow. Same endpoints, clean keys, reliable webhooks.
              </p>
            </div>
            <div className="auth-pills">
              <span className="auth-pill">Hosted Checkout</span>
              <span className="auth-pill">Smart Retries</span>
              <span className="auth-pill">Disputes + Settlements</span>
            </div>
          </section>
          <section className="auth-form-wrap">
            <div className="login-header">
              <img src="/feelpay.png" alt="FeelPay" className="auth-logo small" />
              <h1>FeelPay</h1>
              <p>Sign in to your merchant dashboard</p>
            </div>

            <form onSubmit={handleSubmit} className="login-form">
              {error && <div className="error-message">{error}</div>}
              
              <div className="form-group">
                <label htmlFor="login-email">Email</label>
                <input 
                  id="login-email"
                  name="email"
                  autoComplete="username"
                  type="email" 
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="merchant@business.com"
                  required
                />
              </div>

              <div className="form-group">
                <label htmlFor="login-password">Password</label>
                <div className="password-input-wrapper">
                  <input 
                    id="login-password"
                    name="password"
                    autoComplete="current-password"
                    type={showPassword ? "text" : "password"} 
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="••••••••"
                    required
                  />
                  <button 
                    type="button" 
                    className="password-toggle"
                    onClick={() => setShowPassword(!showPassword)}
                    tabIndex={-1}
                    aria-label={showPassword ? 'Hide password' : 'Show password'}
                  >
                    {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
                  </button>
                </div>
              </div>

              <button type="submit" className="btn-primary btn-full" disabled={loading}>
                {loading ? 'Signing in...' : 'Sign In'}
              </button>
            </form>

            <div className="login-footer">
              <Link to="/forgot-password">Forgot password?</Link>
              <span>•</span>
              <Link to="/signup">Create account</Link>
              <span>•</span>
              <Link to="/landing">Landing page</Link>
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}
