/**
 * CPR — Curved Planar Reformation (V267).
 *
 * Domain + port + adapter for the panoramic-style straightened slab. The
 * spline is sampled in world coordinates; the backend returns an image path
 * the UI mounts as a 2D texture.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export interface CprBuildInput {
    volumePath: string;
    splineWorld: Array<[number, number, number]>;
    slabThicknessMm?: number;
    samplesAlongCurve?: number;
}

export interface CprBuildOutput {
    outputImagePath: string;
    width: number;
    height: number;
    splineLengthMm: number;
    backend: string;
}

export interface CprPort {
    build(input: CprBuildInput): Promise<CprBuildOutput>;
}

export function createBackendCprAdapter(baseUrl: string = BACKEND_ORIGIN): CprPort {
    return {
        async build(input) {
            const response = await fetch(`${baseUrl}/cad/cpr/build`, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                    volumePath: input.volumePath,
                    splineWorld: input.splineWorld,
                    slabThicknessMm: input.slabThicknessMm ?? 2.0,
                    samplesAlongCurve: input.samplesAlongCurve ?? 256,
                }),
            });
            if (!response.ok) throw new Error(`POST /cad/cpr/build → ${response.status}`);
            return (await response.json()) as CprBuildOutput;
        },
    };
}

/**
 * Catmull-Rom-ish helper to upsample a control polyline before sending. The
 * backend takes the spline as control points; this convenience helps if
 * the UI only has 4-5 anchor points but wants smoother sampling.
 */
export function densifySpline(
    spline: ReadonlyArray<readonly [number, number, number]>,
    steps = 16,
): Array<[number, number, number]> {
    if (spline.length < 2) return spline.map((p) => [...p] as [number, number, number]);
    const out: Array<[number, number, number]> = [];
    for (let i = 0; i < spline.length - 1; i++) {
        const a = spline[i];
        const b = spline[i + 1];
        for (let s = 0; s < steps; s++) {
            const t = s / steps;
            out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]);
        }
    }
    out.push([...spline[spline.length - 1]] as [number, number, number]);
    return out;
}
