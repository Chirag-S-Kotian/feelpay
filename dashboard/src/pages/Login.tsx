import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';

export default function Login() {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

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
            <input 
              type="password" 
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••"
              required
            />
          </div>

          <button type="submit" className="btn-primary btn-full" disabled={loading}>
            {loading ? 'Signing in...' : 'Sign In'}
          </button>
        </form>

        <div className="login-footer">
          <a href="#">Forgot password?</a>
          <span>•</span>
          <a href="#">Create account</a>
        </div>
      </div>
    </div>
  );
}
