/**
 * Implant library catalog (V271).
 *
 * Domain + port + adapter for the implant SKU picker. The picker UI is a
 * sibling component (see `ImplantLibraryPicker.tsx`).
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export type ImplantConnection = 'internal-hex' | 'external-hex' | 'morse-taper' | 'tri-channel';

export interface ImplantSku {
    sku: string;
    manufacturer: string;
    family: string;
    label: string;
    diameterMm: number;
    lengthMm: number;
    connection: ImplantConnection;
    platformMm: number;
    stlPath: string;
    abutmentSkus: string[];
}

export interface ImplantLibraryFilters {
    manufacturer?: string;
    diameterMin?: number;
    diameterMax?: number;
    lengthMin?: number;
    lengthMax?: number;
    connection?: ImplantConnection;
}

export interface ImplantFileAvailability {
    sku: string;
    libraryPath: string;
    fileExists: boolean;
    fileSizeBytes: number;
}

export interface ImplantLibraryPort {
    list(filters: ImplantLibraryFilters): Promise<{ items: ImplantSku[]; total: number }>;
    listManufacturers(): Promise<string[]>;
    get(sku: string): Promise<ImplantSku>;
    /**
     * Local-first availability check — does the STL exist on disk?
     * Replaces the earlier cloud-download stub. The UI greys out SKUs
     * whose mesh is missing.
     */
    availability(sku: string): Promise<ImplantFileAvailability>;
}

export function createBackendImplantLibraryAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): ImplantLibraryPort {
    async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
        const response = await fetch(url, {
            ...init,
            headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
        });
        if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → ${response.status}`);
        return (await response.json()) as T;
    }
    return {
        async list(filters) {
            const qs = new URLSearchParams();
            if (filters.manufacturer) qs.set('manufacturer', filters.manufacturer);
            if (filters.diameterMin !== undefined) qs.set('diameterMin', String(filters.diameterMin));
            if (filters.diameterMax !== undefined) qs.set('diameterMax', String(filters.diameterMax));
            if (filters.lengthMin !== undefined) qs.set('lengthMin', String(filters.lengthMin));
            if (filters.lengthMax !== undefined) qs.set('lengthMax', String(filters.lengthMax));
            if (filters.connection) qs.set('connection', filters.connection);
            return callJson(`${baseUrl}/cad/implant-library/list?${qs}`);
        },
        async listManufacturers() {
            const dto = await callJson<{ manufacturers: string[] }>(
                `${baseUrl}/cad/implant-library/manufacturers`,
            );
            return dto.manufacturers;
        },
        async get(sku) {
            return callJson<ImplantSku>(`${baseUrl}/cad/implant-library/${encodeURIComponent(sku)}`);
        },
        async availability(sku) {
            return callJson<ImplantFileAvailability>(
                `${baseUrl}/cad/implant-library/availability/${encodeURIComponent(sku)}`,
            );
        },
    };
}

/** Connection display label. */
export function connectionLabel(c: ImplantConnection): string {
    switch (c) {
        case 'internal-hex':
            return 'Internal Hex';
        case 'external-hex':
            return 'External Hex';
        case 'morse-taper':
            return 'Morse Taper';
        case 'tri-channel':
            return 'Tri-Channel';
    }
}
