const API_BASE = import.meta.env.VITE_API_BASE_URL || '/api';

export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
export const MERCHANT_SESSION_INVALID_EVENT = 'feelpay:merchant-session-invalid';

type ApiErrorBody = {
  error?: { message?: string };
  message?: string;
};

export class MerchantAuthError extends Error {
  status: number;

  constructor(message = 'Session expired. Please sign in again.', status = 401) {
    super(message);
    this.name = 'MerchantAuthError';
    this.status = status;
  }
}

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
export const SESSION_EXPIRES_AT_KEY = 'feelpay_session_expires_at';
const DEFAULT_GET_CACHE_MS = 15_000;
const getCache = new Map<string, { expiresAt: number; data: unknown }>();
const inFlight = new Map<string, Promise<unknown>>();

export function clearMerchantSession() {
  localStorage.removeItem('feelpay_token');
  localStorage.removeItem('feelpay_merchant');
  localStorage.removeItem(SESSION_EXPIRES_AT_KEY);
  getCache.clear();
  inFlight.clear();
}

function notifyInvalidSession(reason: 'expired' | 'unauthorized' = 'unauthorized') {
  clearMerchantSession();
  window.dispatchEvent(
    new CustomEvent(MERCHANT_SESSION_INVALID_EVENT, {
      detail: { reason },
    }),
  );
}

export function setSessionExpiryFromSeconds(seconds = 86_400) {
  localStorage.setItem(SESSION_EXPIRES_AT_KEY, String(Date.now() + seconds * 1000));
}

export function isSessionExpired() {
  const expiresAt = Number(localStorage.getItem(SESSION_EXPIRES_AT_KEY) || '0');
  return Boolean(expiresAt && Date.now() >= expiresAt);
}

export function getEnvironmentMode(): EnvironmentMode {
  const value = localStorage.getItem(ENV_KEY);
  return value === 'live' ? 'live' : 'test';
}

export function setEnvironmentMode(mode: EnvironmentMode) {
  localStorage.setItem(ENV_KEY, mode);
}

function isPublicPath(path: string) {
  return (
    path.startsWith('/v1/public/') ||
    path === '/v1/merchants/login' ||
    path === '/v1/merchants/register' ||
    path === '/v1/merchants/forgot-password' ||
    path.startsWith('/v1/checkout/public/')
  );
}

function messageFromBody(data: unknown, fallback: string) {
  if (data && typeof data === 'object') {
    const body = data as ApiErrorBody;
    return body.error?.message || body.message || fallback;
  }
  return fallback;
}

export async function apiRequest<T = unknown>(
  method: ApiMethod,
  path: string,
  body?: unknown,
  customToken?: string | null,
  opts?: { useApiKey?: boolean; idempotencyKey?: string; cacheMs?: number },
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  const token = customToken || getToken();
  const apiKey = opts?.useApiKey ? getApiKey() : null;
  const protectedPath = !isPublicPath(path);

  if (protectedPath && !customToken && isSessionExpired()) {
    notifyInvalidSession('expired');
    throw new MerchantAuthError();
  }

  if (protectedPath && !token && !apiKey) {
    notifyInvalidSession('unauthorized');
    throw new MerchantAuthError();
  }

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  if (apiKey) {
    headers['x-api-key'] = apiKey;
  }

  if (opts?.idempotencyKey) {
    headers['idempotency-key'] = opts.idempotencyKey;
  }

  const envMode = getEnvironmentMode();
  headers['x-feelpay-environment'] = envMode;
  const isGet = method === 'GET';
  const cacheKey =
    isGet
      ? `${envMode}:${opts?.useApiKey ? 'key' : 'auth'}:${path}`
      : '';
  const cacheMs = opts?.cacheMs ?? DEFAULT_GET_CACHE_MS;

  if (isGet && cacheMs > 0) {
    const now = Date.now();
    const cached = getCache.get(cacheKey);
    if (cached && cached.expiresAt > now) {
      return cached.data as T;
    }
    const pending = inFlight.get(cacheKey);
    if (pending) {
      return pending as Promise<T>;
    }
  }

  const doFetch = () =>
    fetch(`${API_BASE}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

  const requestPromise = (async () => {
    let res = await doFetch();
    // Smooth rate-limit spikes in dev/refresh storms.
    if (res.status === 429 && method === 'GET') {
      await new Promise((resolve) => setTimeout(resolve, 350));
      res = await doFetch();
    }
    if (res.status === 429 && method === 'GET') {
      await new Promise((resolve) => setTimeout(resolve, 700));
      res = await doFetch();
    }

    const text = await res.text();
    let data: unknown = null;
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = { message: text };
      }
    }

    if (!res.ok) {
      if (res.status === 401 || res.status === 403) {
        notifyInvalidSession('unauthorized');
        throw new MerchantAuthError(messageFromBody(data, 'Session expired. Please sign in again.'), res.status);
      }
      if (isGet) {
        const stale = getCache.get(cacheKey);
        if (stale?.data) return stale.data as T;
      }
      throw new Error(
        messageFromBody(data, `API request failed (${res.status})`),
      );
    }

    if (!isGet) {
      getCache.clear();
      inFlight.clear();
    }

    if (isGet && cacheMs > 0) {
      getCache.set(cacheKey, { expiresAt: Date.now() + cacheMs, data });
    }

    return data as T;
  })();

  if (isGet && cacheMs > 0) {
    inFlight.set(cacheKey, requestPromise);
  }

  try {
    return await requestPromise;
  } finally {
    if (isGet && cacheMs > 0) {
      inFlight.delete(cacheKey);
    }
  }
}
