const API_BASE = import.meta.env.VITE_ADMIN_API_BASE_URL || 'http://127.0.0.1:8081';

export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

function getToken() {
  return localStorage.getItem('feelpay_token');
}

function getApiKey() {
  const raw = localStorage.getItem('feelpay_merchant');
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as { api_key?: string };
    return parsed.api_key || null;
  } catch {
    return null;
  }
}

export async function apiRequest<T = unknown>(
  method: ApiMethod,
  path: string,
  body?: unknown,
  opts?: { useApiKey?: boolean; idempotencyKey?: string },
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  const token = getToken();
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  if (opts?.useApiKey) {
    const apiKey = getApiKey();
    if (apiKey) {
      headers['x-api-key'] = apiKey;
    }
  }

  if (opts?.idempotencyKey) {
    headers['idempotency-key'] = opts.idempotencyKey;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  const text = await res.text();
  const data = text ? JSON.parse(text) : null;

  if (!res.ok) {
    if (res.status === 401) {
      localStorage.removeItem('feelpay_token');
    }
    throw new Error(data?.message || `API request failed (${res.status})`);
  }

  return data as T;
}
