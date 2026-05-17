import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type {
    ComputeDistancesInput,
    ComputeDistancesOutput,
    ShowDistancesPort,
} from '../application/show-distances-port';

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

export function createBackendShowDistancesAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): ShowDistancesPort {
    return {
        async compute(input: ComputeDistancesInput): Promise<ComputeDistancesOutput> {
            return callJson<ComputeDistancesOutput>(`${baseUrl}/cad/show-distances/compute`, {
                method: 'POST',
                body: JSON.stringify({
                    restorationPath: input.restorationPath,
                    antagonistPath: input.antagonistPath ?? null,
                    mesialPath: input.mesialPath ?? null,
                    distalPath: input.distalPath ?? null,
                    includeHealthy: input.includeHealthy ?? false,
                    dynamic: input.dynamic ?? false,
                    colorScaleMm: input.colorScaleMm,
                }),
            });
        },
    };
}
