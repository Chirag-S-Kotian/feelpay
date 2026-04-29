import { useEffect, useState } from 'react';
import { Copy, Check, RefreshCw, Eye, EyeOff } from 'lucide-react';
import { apiRequest, getEnvironmentMode, setEnvironmentMode, type EnvironmentMode } from '../hooks/useApi';

export default function Developers() {
  const [apiKey, setApiKey] = useState('Loading...');
  const [secretKey] = useState('Hidden for security');
  const [showSecret, setShowSecret] = useState(false);
  const [mode, setMode] = useState<EnvironmentMode>(getEnvironmentMode());
  const [copied, setCopied] = useState<string | null>(null);

  const loadKeys = async () => {
    const data = await apiRequest<{ api_key: string }>('GET', '/v1/merchants/keys');
    setApiKey(data.api_key);
    const merchantRaw = localStorage.getItem('feelpay_merchant');
    if (merchantRaw) {
      const merchant = JSON.parse(merchantRaw) as Record<string, string>;
      merchant[mode === 'live' ? 'api_key_live' : 'api_key_test'] = data.api_key;
      merchant.api_key = data.api_key;
      localStorage.setItem('feelpay_merchant', JSON.stringify(merchant));
    }
  };

  useEffect(() => {
    setEnvironmentMode(mode);
    void loadKeys();
  }, [mode]);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  const regenerateKeys = async () => {
    await apiRequest('POST', '/v1/merchants/keys/roll', {});
    await loadKeys();
  };

  const codeExamples: Record<string, string> = {
    curl: `curl -X POST https://api.feelpay.in/v1/payments \\
  -H "X-Api-Key: ${apiKey}" \\
  -H "Content-Type: application/json" \\
  -d '{
    "order_id": "ord_123",
    "amount": 50000,
    "currency": "INR",
    "payment_method": {
      "type": "card",
      "card": {
        "number": "4111111111111111",
        "expiry_month": 12,
        "expiry_year": 2027,
        "cvv": "123"
      }
    }
  }'`,
    node: `const feelpay = require('feelpay')('${apiKey}', '${secretKey}');

const payment = await feelpay.payments.create({
  order_id: 'ord_123',
  amount: 50000,
  currency: 'INR',
  payment_method: {
    type: 'card',
    card: {
      number: '4111111111111111',
      expiry_month: 12,
      expiry_year: 2027,
      cvv: '123'
    }
  }
});`,
    python: `import feelpay

client = feelpay.Client('${apiKey}', '${secretKey}')

payment = client.payments.create(
    order_id='ord_123',
    amount=50000,
    currency='INR',
    payment_method={
        'type': 'card',
        'card': {
            'number': '4111111111111111',
            'expiry_month': 12,
            'expiry_year': 2027,
            'cvv': '123'
        }
    }
)`
  };

  const [selectedLang, setSelectedLang] = useState('curl');

  return (
    <div className="page">
      <h1>Developer Settings</h1>

      <section className="api-section">
        <h2>API Keys</h2>
        <div className="mode-toggle">
          <button 
            className={mode === 'test' ? 'active' : ''} 
            onClick={() => setMode('test')}
          >
            Test Mode
          </button>
          <button 
            className={mode === 'live' ? 'active' : ''} 
            onClick={() => setMode('live')}
          >
            Live Mode
          </button>
        </div>

        <div className="key-card">
          <label>API Key</label>
          <div className="key-row">
            <code>{apiKey}</code>
            <button onClick={() => copyToClipboard(apiKey, 'key')}>
              {copied === 'key' ? <Check size={16} /> : <Copy size={16} />}
            </button>
          </div>
        </div>

        <div className="key-card">
          <label>Secret Key</label>
          <div className="key-row">
            <code>{showSecret ? secretKey : '••••••••••••••••••••••••'}</code>
            <button onClick={() => setShowSecret(!showSecret)}>
              {showSecret ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
            <button onClick={() => copyToClipboard(secretKey, 'secret')}>
              {copied === 'secret' ? <Check size={16} /> : <Copy size={16} />}
            </button>
          </div>
        </div>

        <button className="btn-secondary" onClick={regenerateKeys}>
          <RefreshCw size={16} /> Regenerate Keys
        </button>
      </section>

      <section className="docs-section">
        <h2>Integration</h2>
        <div className="lang-tabs">
          {['curl', 'node', 'python'].map(lang => (
            <button 
              key={lang}
              className={selectedLang === lang ? 'active' : ''}
              onClick={() => setSelectedLang(lang)}
            >
              {lang === 'node' ? 'Node.js' : lang === 'python' ? 'Python' : 'cURL'}
            </button>
          ))}
        </div>
        <pre className="code-block">
          <code>{codeExamples[selectedLang]}</code>
        </pre>
      </section>

      <section className="webhooks-section">
        <h2>Webhooks</h2>
        <p className="help-text">Configure webhook URL in your merchant settings to receive real-time payment events.</p>
        <div className="webhook-events">
          <div className="event">payment.success</div>
          <div className="event">payment.failed</div>
          <div className="event">payment.pending</div>
          <div className="event">refund.processed</div>
          <div className="event">settlement.completed</div>
        </div>
      </section>
    </div>
  );
}
