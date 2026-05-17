/**
 * Single source of truth for the FastAPI backend origin.
 *
 * Bug-fix HIGH#1 — previous adapters duplicated `http://127.0.0.1:17493`
 * across 7+ files; switching ports / hosts required editing each one.
 *
 * Override at build time with `VITE_BACKEND_URL` only for localhost /
 * loopback FastAPI sidecars.
 */

const FALLBACK_BACKEND_ORIGIN = 'http://127.0.0.1:17493';

interface ViteImportMetaEnv {
    readonly VITE_BACKEND_URL?: string;
    readonly DEV?: boolean;
}

interface ViteImportMeta {
    readonly env?: ViteImportMetaEnv;
}

function readEnv(): ViteImportMetaEnv {
    const meta = (import.meta as unknown as ViteImportMeta).env;
    return meta ?? {};
}

export const BACKEND_ORIGIN: string =
    normalizeLocalBackendOrigin(readEnv().VITE_BACKEND_URL) ?? FALLBACK_BACKEND_ORIGIN;

export function backendUrl(path: string): string {
    if (isAbsoluteUrl(path)) {
        throw new Error('Remote backend URLs are disabled in the clinical runtime');
    }
    const prefix = path.startsWith('/') ? '' : '/';
    return `${BACKEND_ORIGIN}${prefix}${path}`;
}

function isAbsoluteUrl(value: string): boolean {
    return /^[a-z][a-z0-9+.-]*:\/\//i.test(value);
}

function normalizeLocalBackendOrigin(value: string | undefined): string | null {
    if (!value) return null;

    try {
        const parsed = new URL(value);
        const isLoopback =
            parsed.hostname === '127.0.0.1' ||
            parsed.hostname === 'localhost' ||
            parsed.hostname === '::1';
        if (!isLoopback) return null;
        if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return null;
        parsed.pathname = '';
        parsed.search = '';
        parsed.hash = '';
        return parsed.toString().replace(/\/$/u, '');
    } catch {
        return null;
    }
}
