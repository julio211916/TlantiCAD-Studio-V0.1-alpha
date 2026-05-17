import { backendUrl } from '@/lib/backend-config';
import type { AIRuntimePort } from '../application/ai-runtime-port';
import type { AICacheState, AIRuntimeStatus } from '../domain/runtime';

function assertCapability(enabled?: boolean) {
    if (!enabled) {
        throw new Error('Local HTTP AI runtime adapter is disabled. Use Tauri/Python local runtime status for clinical workflows.');
    }
}

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) {
        throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    }
    return (await response.json()) as T;
}

export function createBackendAIRuntimeAdapter(options: { enabled?: boolean } = {}): AIRuntimePort {
    const callLocalJson = <T,>(path: string, init?: RequestInit) => {
        assertCapability(options.enabled);
        return callJson<T>(backendUrl(path), init);
    };

    return {
        status: () => callLocalJson<AIRuntimeStatus>('/ai-runtime/status'),
        verify: (modelId: string) =>
            callLocalJson<{ integrityOk: boolean; lastError: string }>(
                `/ai-runtime/verify/${encodeURIComponent(modelId)}`,
                { method: 'POST' },
            ),
        cache: () => callLocalJson<AICacheState>('/ai-runtime/cache'),
        prune: () =>
            callLocalJson<{ freedBytes: number; totalBytesAfter: number }>(
                '/ai-runtime/cache/prune',
                { method: 'POST' },
            ),
        clearCache: async () => {
            await callLocalJson<{ ok: boolean }>('/ai-runtime/cache', { method: 'DELETE' });
        },
    };
}
