import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type {
    ToothStlPort,
    ToothStlWriteInput,
    ToothStlWriteOutput,
} from '../application/tooth-stl-port';
import type { ToothStlEntry } from '../domain/tooth-stl';

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

interface ListDTO {
    teeth: ToothStlEntry[];
}

export function createBackendToothStlAdapter(baseUrl: string = BACKEND_ORIGIN): ToothStlPort {
    return {
        async write(input: ToothStlWriteInput): Promise<ToothStlWriteOutput> {
            return callJson<ToothStlWriteOutput>(`${baseUrl}/cad/tooth-stl/write`, {
                method: 'POST',
                body: JSON.stringify({
                    caseFolderPath: input.caseFolderPath,
                    toothFdi: input.toothFdi,
                    positions: input.buffer?.positions ?? [],
                    indices: input.buffer?.indices ?? [],
                    placeholderSizeMm: input.placeholderSizeMm ?? 8.0,
                    prefix: input.prefix ?? 'tooth',
                }),
            });
        },

        async list(caseFolderPath: string): Promise<ToothStlEntry[]> {
            const dto = await callJson<ListDTO>(`${baseUrl}/cad/tooth-stl/list`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath }),
            });
            return dto.teeth;
        },

        async clear(caseFolderPath: string): Promise<number> {
            const dto = await callJson<{ removed: number }>(`${baseUrl}/cad/tooth-stl/clear`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath }),
            });
            return dto.removed;
        },
    };
}
