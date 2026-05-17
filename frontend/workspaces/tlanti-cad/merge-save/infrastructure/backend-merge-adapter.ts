/**
 * HTTP adapter for the merge & save port.
 */

import type {
    MergeCompleteOutput,
    MergePort,
    MergeProgressOutput,
    MergeStartInput,
    MergeStartOutput,
} from '../application/merge-port';

import { BACKEND_ORIGIN } from '../../../lib/backend-config';

const DEFAULT_BASE_URL = BACKEND_ORIGIN;

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

export function createBackendMergeAdapter(baseUrl: string = DEFAULT_BASE_URL): MergePort {
    return {
        async start(input: MergeStartInput): Promise<MergeStartOutput> {
            return callJson<MergeStartOutput>(`${baseUrl}/cad/merge/start`, {
                method: 'POST',
                body: JSON.stringify({
                    caseId: input.caseId,
                    caseFolderPath: input.caseFolderPath,
                    teeth: input.teeth,
                    optimizeFor3dPrint: input.optimizeFor3dPrint,
                }),
            });
        },
        async progress(jobId: string): Promise<MergeProgressOutput> {
            return callJson<MergeProgressOutput>(`${baseUrl}/cad/merge/progress/${jobId}`, {
                method: 'POST',
            });
        },
        async cancel(jobId: string): Promise<void> {
            await callJson(`${baseUrl}/cad/merge/cancel/${jobId}`, { method: 'POST' });
        },
        async finalize(jobId: string): Promise<MergeCompleteOutput> {
            return callJson<MergeCompleteOutput>(`${baseUrl}/cad/merge/finalize/${jobId}`, {
                method: 'POST',
            });
        },
        async remove(caseFolderPath: string): Promise<number> {
            const result = await callJson<{ removed: number }>(`${baseUrl}/cad/merge/remove`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath }),
            });
            return result.removed;
        },
    };
}
