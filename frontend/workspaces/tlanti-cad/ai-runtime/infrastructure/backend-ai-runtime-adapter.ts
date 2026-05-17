import type { AIRuntimePort } from '../application/ai-runtime-port';
import type { AICacheState, AIRuntimeStatus } from '../domain/runtime';

const DEFAULT_LOCAL_RUNTIME_ORIGIN = 'http://127.0.0.1:17493';

export interface BackendAIRuntimeAdapterOptions {
    baseUrl?: string;
    fetcher?: typeof fetch;
    /**
     * Must be true before this adapter performs HTTP. The default is
     * intentionally unavailable so the offline desktop does not silently
     * depend on a generic web backend.
     */
    enabled?: boolean;
}

function createUnavailableAdapterError(): Error {
    return new Error(
        'AI runtime HTTP adapter is unavailable. Use the local Tauri/Python healthcheck path or explicitly enable the loopback adapter for a supervised local sidecar.',
    );
}

function assertLoopbackUrl(rawUrl: string): void {
    const url = new URL(rawUrl);
    if (url.protocol !== 'http:') {
        throw new Error(`AI runtime adapter only permits local http loopback, received ${url.protocol}`);
    }
    if (!['127.0.0.1', 'localhost', '[::1]', '::1'].includes(url.hostname)) {
        throw new Error(`AI runtime adapter blocked non-loopback host: ${url.hostname}`);
    }
}

async function callJson<T>(fetcher: typeof fetch, url: string, init?: RequestInit): Promise<T> {
    const response = await fetcher(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) {
        throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    }
    return (await response.json()) as T;
}

export function createBackendAIRuntimeAdapter(
    options: string | BackendAIRuntimeAdapterOptions = {},
): AIRuntimePort {
    const resolvedOptions =
        typeof options === 'string' ? { baseUrl: options, enabled: false } : options;
    const baseUrl = resolvedOptions.baseUrl ?? DEFAULT_LOCAL_RUNTIME_ORIGIN;
    const fetcher = resolvedOptions.fetcher ?? fetch;

    function assertCapability() {
        if (resolvedOptions.enabled !== true) {
            throw createUnavailableAdapterError();
        }
        assertLoopbackUrl(baseUrl);
    }

    async function gatedCallJson<T>(url: string, init?: RequestInit): Promise<T> {
        assertCapability();
        return callJson<T>(fetcher, url, init);
    }

    return {
        status: () => gatedCallJson<AIRuntimeStatus>(`${baseUrl}/ai-runtime/status`),
        verify: (modelId: string) =>
            gatedCallJson<{ integrityOk: boolean; lastError: string }>(
                `${baseUrl}/ai-runtime/verify/${encodeURIComponent(modelId)}`,
                { method: 'POST' },
            ),
        cache: () => gatedCallJson<AICacheState>(`${baseUrl}/ai-runtime/cache`),
        prune: () =>
            gatedCallJson<{ freedBytes: number; totalBytesAfter: number }>(
                `${baseUrl}/ai-runtime/cache/prune`,
                { method: 'POST' },
            ),
        clearCache: async () => {
            await gatedCallJson<{ ok: boolean }>(`${baseUrl}/ai-runtime/cache`, { method: 'DELETE' });
        },
    };
}
