/**
 * HTTP adapter for the screw-holes port.
 */

import type {
    ScrewHolesApplyInput,
    ScrewHolesApplyOutput,
    ScrewHolesPort,
} from '../application/screw-holes-port';

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

export function createBackendScrewHolesAdapter(baseUrl: string = DEFAULT_BASE_URL): ScrewHolesPort {
    return {
        async apply(input: ScrewHolesApplyInput): Promise<ScrewHolesApplyOutput> {
            return callJson<ScrewHolesApplyOutput>(`${baseUrl}/cad/screw-holes/apply`, {
                method: 'POST',
                body: JSON.stringify({
                    caseFolderPath: input.caseFolderPath,
                    offsetMm: input.offsetMm,
                    minDiameterMm: input.minDiameterMm,
                    toothMask: input.toothMask,
                }),
            });
        },
        async clear(caseFolderPath: string): Promise<boolean> {
            const result = await callJson<{ cleared: boolean }>(`${baseUrl}/cad/screw-holes/clear`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath }),
            });
            return result.cleared;
        },
    };
}
