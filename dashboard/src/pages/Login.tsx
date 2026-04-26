import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';

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

    // Simulated login - in production this would call the API
    setTimeout(() => {
      if (email && password) {
        localStorage.setItem('feelpay_token', 'mock_jwt_token');
        localStorage.setItem('feelpay_merchant', JSON.stringify({
          id: 'merch_001',
          name: 'Demo Merchant',
          email: email,
          tier: 'growth'
        }));
        navigate('/');
      } else {
        setError('Invalid credentials');
        setLoading(false);
      }
    }, 1000);
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
