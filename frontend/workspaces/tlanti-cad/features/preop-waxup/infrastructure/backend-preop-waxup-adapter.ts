import { BACKEND_ORIGIN } from '../../../lib/backend-config';
import type {
    PreopAdaptInput,
    PreopAlignInput,
    PreopWaxupPort,
    WaxupPrepareInput,
} from '../application/preop-waxup-port';
import type {
    Mat4,
    PreopAdaptResult,
    PreopAlignment,
    WaxupPreparation,
} from '../domain/preop-waxup';

interface AlignDTO {
    transformMatrix: number[][];
    rmsMm: number;
    backend: string;
}

interface AdaptDTO {
    converged: boolean;
    iterationsRun: number;
    rmsMm: number;
    backend: string;
}

interface WaxupDTO {
    preparedPath: string;
    holesClosed: number;
    cropped: boolean;
    backend: string;
    warnings: string[];
}

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

function toMat4(rows: number[][]): Mat4 {
    return [
        [rows[0][0], rows[0][1], rows[0][2], rows[0][3]],
        [rows[1][0], rows[1][1], rows[1][2], rows[1][3]],
        [rows[2][0], rows[2][1], rows[2][2], rows[2][3]],
        [rows[3][0], rows[3][1], rows[3][2], rows[3][3]],
    ];
}

export function createBackendPreopWaxupAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): PreopWaxupPort {
    return {
        async alignPreop(input: PreopAlignInput): Promise<PreopAlignment> {
            const dto = await callJson<AlignDTO>(`${baseUrl}/cad/preop/align`, {
                method: 'POST',
                body: JSON.stringify({
                    preopPath: input.preopPath,
                    modelPath: input.modelPath,
                    initialTranslationMm: input.initialTranslationMm ?? [0, 0, 0],
                }),
            });
            return { transformMatrix: toMat4(dto.transformMatrix), rmsMm: dto.rmsMm, backend: dto.backend };
        },

        async adaptToPreop(input: PreopAdaptInput): Promise<PreopAdaptResult> {
            const dto = await callJson<AdaptDTO>(`${baseUrl}/cad/preop/adapt`, {
                method: 'POST',
                body: JSON.stringify({
                    preopPath: input.preopPath,
                    toothPaths: input.toothPaths,
                    iterations: input.iterations,
                }),
            });
            return {
                converged: dto.converged,
                iterationsRun: dto.iterationsRun,
                rmsMm: dto.rmsMm,
                backend: dto.backend,
            };
        },

        async prepareWaxup(input: WaxupPrepareInput): Promise<WaxupPreparation> {
            const dto = await callJson<WaxupDTO>(`${baseUrl}/cad/waxup/prepare`, {
                method: 'POST',
                body: JSON.stringify({
                    waxupPath: input.waxupPath,
                    marginPolylinePerTooth: input.marginPolylinePerTooth,
                    cropAboveMargin: input.cropAboveMargin,
                    closeHoles: input.closeHoles,
                }),
            });
            return {
                preparedPath: dto.preparedPath,
                holesClosed: dto.holesClosed,
                cropped: dto.cropped,
                backend: dto.backend,
                warnings: dto.warnings,
            };
        },
    };
}
