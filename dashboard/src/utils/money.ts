export function formatPaise(amount: number, { symbol = true }: { symbol?: boolean } = {}) {
  const paise = Math.trunc(amount || 0);
  const sign = paise < 0 ? '-' : '';
  const absolute = Math.abs(paise);
  const rupees = Math.trunc(absolute / 100).toLocaleString('en-IN');
  const formatted = `${sign}${rupees}.${String(absolute % 100).padStart(2, '0')}`;
  return symbol ? `₹${formatted}` : formatted;
}

export function parseInrToPaise(value: string) {
  const trimmed = value.trim();
  if (!/^\d+(\.\d{0,2})?$/.test(trimmed)) return null;
  const [rupees, paise = ''] = trimmed.split('.');
  if (rupees === undefined) return null;
  const parsed = BigInt(rupees) * 100n + BigInt(paise.padEnd(2, '0') || '0');
  if (parsed > BigInt(Number.MAX_SAFE_INTEGER)) return null;
  return Number(parsed);
}
