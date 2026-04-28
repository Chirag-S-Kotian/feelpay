import React from 'react';

export type PaymentButtonProps = {
  label?: string;
  disabled?: boolean;
  onClick: () => void;
};

export function PaymentButton({ label = 'Pay now', disabled, onClick }: PaymentButtonProps) {
  return (
    <button type="button" disabled={disabled} onClick={onClick}>
      {label}
    </button>
  );
}

export type CheckoutFrameProps = {
  url: string;
  title?: string;
};

export function CheckoutFrame({ url, title = 'FeelPay checkout' }: CheckoutFrameProps) {
  return <iframe src={url} title={title} style={{ width: '100%', minHeight: 640, border: 0 }} />;
}
