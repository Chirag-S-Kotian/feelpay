import { useState, FormEvent } from 'react';
import { Link } from 'react-router-dom';
import { apiRequest } from '../hooks/useApi';

export default function ForgotPassword() {
  const [email, setEmail] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setMessage('');

    try {
      const res = await apiRequest<{ message: string }>('POST', '/v1/merchants/forgot-password', { email });
      setMessage(res.message);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Request failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="login-header">
          <h1>FeelPay</h1>
          <p>Reset your password</p>
        </div>

        <form onSubmit={handleSubmit} className="login-form">
          {error && <div className="error-message">{error}</div>}
          {message && <div className="success-message" style={{ color: '#22c55e', marginBottom: '14px', textAlign: 'center', fontSize: '14px' }}>{message}</div>}
          
          <div className="form-group">
            <label>Email Address</label>
            <input 
              type="email" 
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="merchant@business.com"
              required
              disabled={!!message}
            />
          </div>

          {!message && (
            <button type="submit" className="btn-primary btn-full" disabled={loading}>
              {loading ? 'Sending link...' : 'Send Reset Link'}
            </button>
          )}
        </form>

        <div className="login-footer">
          <Link to="/login">Back to Sign In</Link>
        </div>
      </div>
    </div>
  );
}
