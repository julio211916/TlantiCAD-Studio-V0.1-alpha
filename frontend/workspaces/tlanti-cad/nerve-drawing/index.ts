/**
 * Nerve drawing (V268) — domain + port + adapter.
 *
 * Per-tooth IAN canal splines (left + right). The R3F overlay (future
 * sprint) renders the splines as `Line2` instances with a glow shader.
 * Persistence is per-case JSON via the backend.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export type NerveSide = 'left' | 'right';

export interface NerveSpline {
    toothFdi: number;
    side: NerveSide;
    points: Array<[number, number, number]>;
    colorRgb: [number, number, number];
}

export interface NervePort {
    save(caseFolderPath: string, spline: NerveSpline): Promise<NerveSpline>;
    list(caseFolderPath: string): Promise<NerveSpline[]>;
}

export function createBackendNerveAdapter(baseUrl: string = BACKEND_ORIGIN): NervePort {
    async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
        const response = await fetch(url, {
            ...init,
            headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
        });
        if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → ${response.status}`);
        return (await response.json()) as T;
    }
    return {
        async save(caseFolderPath, spline) {
            return callJson<NerveSpline>(`${baseUrl}/cad/nerve/save`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath, spline }),
            });
        },
        async list(caseFolderPath) {
            const dto = await callJson<{ splines: NerveSpline[] }>(`${baseUrl}/cad/nerve/list`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath }),
            });
            return dto.splines;
        },
    };
}

/**
 * Compute the polyline length in mm — used for the safety-zone check
 * (V273) and as a UX hint while the user draws.
 */
export function nerveLengthMm(spline: Pick<NerveSpline, 'points'>): number {
    let length = 0;
    for (let i = 1; i < spline.points.length; i++) {
        const [ax, ay, az] = spline.points[i - 1];
        const [bx, by, bz] = spline.points[i];
        length += Math.hypot(bx - ax, by - ay, bz - az);
    }
    return length;
}
