const API_BASE = import.meta.env.VITE_ADMIN_API_BASE_URL || 'http://127.0.0.1:8081';

export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

function buildUrl(path: string) {
  if (path.startsWith('http://') || path.startsWith('https://')) return path;
  return `${API_BASE}${path.startsWith('/') ? path : `/${path}`}`;
}

export function setToken(token: string) {
  localStorage.setItem('feelpay_token', token);
}

export function clearToken() {
  localStorage.removeItem('feelpay_token');
}

export function getToken() {
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

  let res: Response;
  try {
    res = await fetch(buildUrl(path), {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });
  } catch (err) {
    throw new Error(`Network/CORS failure: ${err instanceof Error ? err.message : 'request blocked'}`);
  }

  const text = await res.text();
  let parsed: unknown = null;
  try {
    parsed = text ? JSON.parse(text) : null;
  } catch {
    parsed = { message: text };
  }
  const errorData = parsed && typeof parsed === 'object' && !Array.isArray(parsed)
    ? parsed as { message?: string; request_id?: string }
    : null;

  if (!res.ok) {
    if (res.status === 401) {
      clearToken();
    }
    const label =
      res.status === 401 ? 'Unauthenticated' :
      res.status === 403 ? 'Not authorized' :
      res.status === 404 ? 'Route missing' :
      res.status >= 500 ? 'Backend error' :
      'API request failed';
    const requestId = errorData?.request_id || res.headers.get('x-request-id');
    throw new Error(`${label} (${res.status}): ${errorData?.message || res.statusText}${requestId ? ` [request_id: ${requestId}]` : ''}`);
  }

  return parsed as T;
}

export async function apiDownload(path: string): Promise<{ blob: Blob; fileName: string }> {
  const headers: Record<string, string> = {};
  const token = getToken();
  if (token) headers.Authorization = `Bearer ${token}`;

  const res = await fetch(buildUrl(path), { method: 'GET', headers });
  if (!res.ok) {
    const text = await res.text();
    let message = `API request failed (${res.status})`;
    try {
      const data = text ? JSON.parse(text) as { message?: string } : null;
      message = data?.message || message;
    } catch {
      message = text || message;
    }
    if (res.status === 401) clearToken();
    throw new Error(message);
  }

  const disposition = res.headers.get('content-disposition') || '';
  const match = /filename="?([^"]+)"?/i.exec(disposition);
  return {
    blob: await res.blob(),
    fileName: match?.[1] || 'report-export',
  };
}
