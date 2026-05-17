/**
 * HTTP adapter for the insertion-direction port.
 */

import type {
    InsertionDetectInput,
    InsertionDetectionPort,
} from '../application/insertion-direction-port';
import type { InsertionAxis, Vec3 } from '../domain/insertion-axis';
import { BACKEND_ORIGIN } from '../../../lib/backend-config';

const DEFAULT_BACKEND_ORIGIN = BACKEND_ORIGIN;

interface DetectDTO {
    axis: Vec3;
    undercut_volume_mm3: number;
    undercut_peak_mm: number;
    confidence: number;
    backend: string;
}

interface UnifyDTO {
    axis: Vec3;
    max_deviation_degrees: number;
}

export function createBackendInsertionAdapter(options?: {
    origin?: string;
    fetcher?: typeof fetch;
}): InsertionDetectionPort {
    const origin = options?.origin ?? DEFAULT_BACKEND_ORIGIN;
    const fetcher = options?.fetcher ?? fetch;

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
        async detect(input: InsertionDetectInput): Promise<InsertionAxis> {
            const dto = await post<DetectDTO>('/cad/insertion/detect', {
                mesh_path: input.meshPath,
                tooth_fdi: input.toothFdi,
                margin_polyline: input.marginPolyline,
            });
            const backend: InsertionAxis['backend'] =
                dto.backend === 'trimesh-pca' ? 'trimesh-pca' : 'mock';
            return {
                toothFdi: input.toothFdi,
                vector: dto.axis,
                undercutVolumeMm3: dto.undercut_volume_mm3,
                undercutPeakMm: dto.undercut_peak_mm,
                confidence: dto.confidence,
                backend,
            };
        },

        async unifyBridge(axes, weights) {
            const dto = await post<UnifyDTO>('/cad/insertion/unify-bridge', {
                axes,
                weights,
            });
            return {
                axis: dto.axis,
                maxDeviationDegrees: dto.max_deviation_degrees,
            };
        },
    };
}
