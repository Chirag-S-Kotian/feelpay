const API_BASE = import.meta.env.VITE_API_BASE_URL || '/api';

export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

function getToken() {
  return localStorage.getItem('feelpay_token');
}

function getApiKey() {
  const mode = getEnvironmentMode();
  const raw = localStorage.getItem('feelpay_merchant');
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as {
      api_key?: string;
      api_key_test?: string;
      api_key_live?: string;
    };
    if (mode === 'live') return parsed.api_key_live || parsed.api_key || null;
    return parsed.api_key_test || parsed.api_key || null;
  } catch {
    return null;
  }
}

export type EnvironmentMode = 'test' | 'live';

const ENV_KEY = 'feelpay_environment_mode';

export function getEnvironmentMode(): EnvironmentMode {
  const value = localStorage.getItem(ENV_KEY);
  return value === 'live' ? 'live' : 'test';
}

export function setEnvironmentMode(mode: EnvironmentMode) {
  localStorage.setItem(ENV_KEY, mode);
}

export async function apiRequest<T = unknown>(
  method: ApiMethod,
  path: string,
  body?: unknown,
  customToken?: string | null,
  opts?: { useApiKey?: boolean; idempotencyKey?: string },
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  const token = customToken || getToken();
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

  headers['x-feelpay-environment'] = getEnvironmentMode();

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
    throw new Error(
      data?.error?.message || data?.message || `API request failed (${res.status})`,
    );
  }

  return data as T;
}
