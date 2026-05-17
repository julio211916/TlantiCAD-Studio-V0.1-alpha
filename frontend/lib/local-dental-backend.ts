const DEFAULT_DENTAL_BACKEND_URL = 'http://127.0.0.1:17493';
const DENTAL_BACKEND_STORAGE_KEY = 'tlanticad.dentalBackend.baseUrl';

function normalizeLoopbackBaseUrl(value: string | null | undefined) {
  const candidate = (value || DEFAULT_DENTAL_BACKEND_URL).replace(/\/$/, '');

  try {
    const url = new URL(candidate);
    const isLoopback = url.hostname === '127.0.0.1' || url.hostname === 'localhost';
    if ((url.protocol === 'http:' || url.protocol === 'https:') && isLoopback) {
      return url.toString().replace(/\/$/, '');
    }
  } catch {
    // Fall back to the embedded local dental backend.
  }

  return DEFAULT_DENTAL_BACKEND_URL;
}

export function getDentalBackendBaseUrl() {
  if (typeof window === 'undefined') {
    return DEFAULT_DENTAL_BACKEND_URL;
  }

  return normalizeLoopbackBaseUrl(window.localStorage.getItem(DENTAL_BACKEND_STORAGE_KEY));
}

export function setDentalBackendBaseUrl(value: string) {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.setItem(DENTAL_BACKEND_STORAGE_KEY, normalizeLoopbackBaseUrl(value));
}

export function buildDentalBackendUrl(path: string) {
  return `${getDentalBackendBaseUrl()}${path.startsWith('/') ? path : `/${path}`}`;
}
