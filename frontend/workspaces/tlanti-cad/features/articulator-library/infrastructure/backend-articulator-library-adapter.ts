import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type { ArticulatorLibraryPort } from '../application/articulator-library-port';
import type {
    ArticulatorPreset,
    ArticulatorVendorEntry,
} from '../domain/articulator-vendor';

interface ListDTO {
    backend: 'filesystem' | 'mock';
    vendors: ArticulatorVendorEntry[];
    libraryPath: string | null;
}

interface PresetDTO {
    id: string;
    label: string;
    vendor: string;
    config: {
        condyleInclinationDeg: number;
        bennettAngleDeg: number;
        immediateSideShiftMm: number;
        intercondylarDistanceMm: number;
        incisalGuidanceDeg: number;
    };
    backend: string;
}

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

export function createBackendArticulatorLibraryAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): ArticulatorLibraryPort {
    return {
        async list() {
            const dto = await callJson<ListDTO>(`${baseUrl}/cad/articulator-library`);
            return { vendors: dto.vendors, backend: dto.backend };
        },
        async getPreset(vendorId: string): Promise<ArticulatorPreset> {
            const dto = await callJson<PresetDTO>(
                `${baseUrl}/cad/articulator-library/preset/${encodeURIComponent(vendorId)}`,
            );
            return {
                id: dto.id,
                label: dto.label,
                vendor: dto.vendor,
                config: dto.config,
                backend: dto.backend,
            };
        },
    };
}
