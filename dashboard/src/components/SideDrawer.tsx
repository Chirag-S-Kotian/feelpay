import React, { useEffect } from 'react';
import { X } from 'lucide-react';

interface SideDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  children: React.ReactNode;
}

export function SideDrawer({ isOpen, onClose, title, subtitle, children }: SideDrawerProps) {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }
    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <>
      <div className="drawer-overlay" onClick={onClose} />
      <div className={`drawer-content ${isOpen ? 'open' : ''}`}>
        <div className="drawer-header">
          <div>
            <h3>{title}</h3>
            {subtitle && <p>{subtitle}</p>}
          </div>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>
        <div className="drawer-body">
          {children}
        </div>
      </div>
      <style>{`
        .drawer-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.4);
          backdrop-filter: blur(4px);
          z-index: 1000;
          animation: fadeIn 0.2s ease-out;
        }

        .drawer-content {
          position: fixed;
          top: 0;
          right: 0;
          bottom: 0;
          width: 500px;
          max-width: 90vw;
          background: var(--fp-bg);
          border-left: 1px solid var(--fp-border);
          z-index: 1001;
          display: flex;
          flex-direction: column;
          animation: slideIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
          box-shadow: -10px 0 30px rgba(0,0,0,0.2);
        }

        .drawer-header {
          padding: 24px;
          border-bottom: 1px solid var(--fp-border);
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .drawer-header h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
        }

        .drawer-header p {
          margin: 4px 0 0;
          font-size: 13px;
          color: var(--fp-text-muted);
        }

        .drawer-body {
          flex: 1;
          overflow-y: auto;
          padding: 24px;
        }

        .close-btn {
          background: none;
          border: none;
          color: var(--fp-text-muted);
          cursor: pointer;
          padding: 8px;
          border-radius: 8px;
          transition: all 0.2s;
        }

        .close-btn:hover {
          background: var(--fp-bg-elevated);
          color: var(--fp-text);
        }

        @keyframes fadeIn {
          from { opacity: 0; }
          to { opacity: 1; }
        }

        @keyframes slideIn {
          from { transform: translateX(100%); }
          to { transform: translateX(0); }
        }

        .detail-section {
          margin-bottom: 32px;
        }

        .detail-section h4 {
          font-size: 12px;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: var(--fp-text-muted);
          margin-bottom: 16px;
        }

        .detail-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
        }

        .detail-item label {
          display: block;
          font-size: 12px;
          color: var(--fp-text-muted);
          margin-bottom: 4px;
        }

        .detail-item span {
          font-size: 14px;
          font-weight: 500;
          word-break: break-all;
        }

        .timeline {
          position: relative;
          padding-left: 32px;
        }

        .timeline::before {
          content: '';
          position: absolute;
          left: 7px;
          top: 0;
          bottom: 0;
          width: 2px;
          background: var(--fp-border);
        }

        .timeline-item {
          position: relative;
          padding-bottom: 24px;
        }

        .timeline-dot {
          position: absolute;
          left: -32px;
          width: 16px;
          height: 16px;
          border-radius: 50%;
          background: var(--fp-bg);
          border: 2px solid var(--fp-primary);
          z-index: 1;
        }

        .timeline-content h5 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
        }

        .timeline-content p {
          margin: 4px 0 0;
          font-size: 12px;
          color: var(--fp-text-muted);
        }

        .fee-row {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          font-size: 14px;
        }

        .fee-row.total {
          border-top: 1px solid var(--fp-border);
          margin-top: 8px;
          padding-top: 16px;
          font-weight: 600;
          font-size: 16px;
        }
      `}</style>
    </>
  );
}
