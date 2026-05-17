/**
 * HTTP adapter that fulfils `MarginDetectionPort` by calling the FastAPI
 * `/cad/margin/*` routes. Keeps a single fetcher reference so tests can
 * inject a fake.
 */

import type {
    MarginCorrectInput,
    MarginDetectInput,
    MarginLine,
    MarginMode,
    MarginRepairInput,
    Vec3,
} from '../domain/margin-line';
import type { MarginDetectionPort } from '../application/margin-detection-port';
import { BACKEND_ORIGIN } from '../../../lib/backend-config';

const DEFAULT_BACKEND_ORIGIN = BACKEND_ORIGIN;

interface Vec3DTO {
    x: number;
    y: number;
    z: number;
}

interface MarginResponseDTO {
    polyline: Vec3DTO[];
    closed: boolean;
    confidence: number;
    mode: MarginMode;
    backend: 'trimesh' | 'mock';
}

function vec3In(v: Vec3): Vec3DTO {
    return { x: v.x, y: v.y, z: v.z };
}

function dtoToMarginLine(dto: MarginResponseDTO, toothFdi: number): MarginLine {
    return {
        toothFdi,
        polyline: dto.polyline.map((p) => ({ x: p.x, y: p.y, z: p.z })),
        closed: dto.closed,
        mode: dto.mode,
        confidence: dto.confidence,
        backend: dto.backend,
    };
}

export function createBackendMarginAdapter(options?: {
    origin?: string;
    fetcher?: typeof fetch;
    /** Used to stamp the resulting margin with its FDI number. */
    resolveToothFdi?: () => number;
}): MarginDetectionPort {
    const origin = options?.origin ?? DEFAULT_BACKEND_ORIGIN;
    const fetcher = options?.fetcher ?? fetch;
    const toothFdi = options?.resolveToothFdi ?? (() => 0);

    async function post<T>(path: string, body: unknown): Promise<T> {
        const res = await fetcher(`${origin}${path}`, {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            const detail = await res.text().catch(() => res.statusText);
            throw new Error(`POST ${path} failed: ${res.status} ${detail}`);
        }
        return (await res.json()) as T;
    }

    return {
        async detect(input: MarginDetectInput) {
            const dto = await post<MarginResponseDTO>('/cad/margin/detect', {
                mesh_path: input.meshPath,
                seed: vec3In(input.seed),
                mode: input.mode,
                max_iterations: input.maxIterations ?? 128,
            });
            return dtoToMarginLine(dto, toothFdi());
        },

        async correct(input: MarginCorrectInput) {
            const dto = await post<MarginResponseDTO>('/cad/margin/correct', {
                mesh_path: input.meshPath,
                seed_points: input.seedPoints.map(vec3In),
                mode: input.mode,
            });
            return dtoToMarginLine(dto, toothFdi());
        },

        async repair(input: MarginRepairInput) {
            const dto = await post<MarginResponseDTO>('/cad/margin/repair', {
                mesh_path: input.meshPath,
                polyline: input.polyline.map(vec3In),
                drag_radius: input.dragRadius ?? 1.0,
                surface_snap_distance: input.surfaceSnapDistance ?? 0.1,
                repair_region_around_margin: input.repairRegionAroundMargin ?? 1.0,
            });
            return dtoToMarginLine(dto, toothFdi());
        },
    };
}
