import { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  override render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div style={{
          padding: 40,
          textAlign: 'center',
          background: 'var(--fp-bg-card)',
          borderRadius: 14,
          border: '1px solid var(--fp-border)',
        }}>
          <h3 style={{ color: 'var(--fp-danger)', marginBottom: 8 }}>Something went wrong</h3>
          <p style={{ color: 'var(--fp-text-muted)', marginBottom: 16 }}>
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          <button
            onClick={() => this.setState({ hasError: false })}
            style={{
              padding: '8px 16px',
              borderRadius: 8,
              border: 'none',
              background: 'var(--fp-primary)',
              color: '#fff',
              cursor: 'pointer',
            }}
          >
            Try Again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
