import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type { MaterialLibraryPort } from '../application/material-library-port';
import type { MaterialLibraryEntry } from '../domain/material-library';

interface ListDTO {
    libraries: MaterialLibraryEntry[];
}

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

export function createBackendMaterialLibraryAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): MaterialLibraryPort {
    return {
        async listLocal(): Promise<MaterialLibraryEntry[]> {
            const dto = await callJson<ListDTO>(`${baseUrl}/cad/material-libraries/local`);
            return dto.libraries;
        },
        async listPeerLibraries(): Promise<MaterialLibraryEntry[]> {
            const dto = await callJson<ListDTO>(
                `${baseUrl}/cad/material-libraries/tlantishare/peer-libraries`,
            );
            return dto.libraries;
        },
        async forgetPeerLibrary(libraryId: string): Promise<{ ok: boolean }> {
            return callJson<{ ok: boolean }>(
                `${baseUrl}/cad/material-libraries/tlantishare/peer-libraries/${encodeURIComponent(libraryId)}`,
                { method: 'DELETE' },
            );
        },
    };
}
