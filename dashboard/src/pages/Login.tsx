import { useState, FormEvent, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { Eye, EyeOff } from 'lucide-react';
import { apiRequest } from '../hooks/useApi';

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
        api_key: string;
      }>('POST', '/v1/merchants/login', { email, password });

      localStorage.setItem('feelpay_token', res.token);
      localStorage.setItem(
        'feelpay_merchant',
        JSON.stringify({
          id: res.merchant_id,
          name: res.name,
          email,
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
    <div className="login-page">
      <div className="login-card">
        <div className="login-header">
          <h1>FeelPay</h1>
          <p>Payment Gateway</p>
        </div>

        <form onSubmit={handleSubmit} className="login-form">
          {error && <div className="error-message">{error}</div>}
          
          <div className="form-group">
            <label>Email</label>
            <input 
              type="email" 
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="merchant@business.com"
              required
            />
          </div>

          <div className="form-group">
            <label>Password</label>
            <div className="password-input-wrapper">
              <input 
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
        </div>
      </div>
    </div>
  );
}
