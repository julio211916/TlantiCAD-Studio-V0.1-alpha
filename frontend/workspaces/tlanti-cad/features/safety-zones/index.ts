/**
 * Safety-zones (V273).
 *
 * Computes per-implant clearance to nerve splines / sinus mesh / cortical
 * bone. The kernel is local-only (point-to-polyline distance for nerves;
 * point-to-mesh distance for sinus/cortical). Surfaces a severity flag
 * the UI can render as a green/amber/red dot next to the implant.
 *
 * Thresholds are clinical defaults; teams can override via
 * `setSafetyThresholds(...)` for protocol-specific cases.
 */

export type SafetySeverity = 'ok' | 'warning' | 'error';

export interface SafetyThresholds {
    /** Distance < error → 'error'. Default 1.5 mm. */
    nerveErrorMm: number;
    /** Distance < warning → 'warning'. Default 3.0 mm. */
    nerveWarningMm: number;
    sinusErrorMm: number;
    sinusWarningMm: number;
    corticalErrorMm: number;
    corticalWarningMm: number;
}

let thresholds: SafetyThresholds = {
    nerveErrorMm: 1.5,
    nerveWarningMm: 3.0,
    sinusErrorMm: 1.0,
    sinusWarningMm: 2.0,
    corticalErrorMm: 1.0,
    corticalWarningMm: 2.0,
};

export function setSafetyThresholds(next: Partial<SafetyThresholds>): void {
    thresholds = { ...thresholds, ...next };
}

export function getSafetyThresholds(): SafetyThresholds {
    return thresholds;
}

export interface SafetyAssessment {
    fdi: number;
    nearestNerveMm: number | null;
    nearestSinusMm: number | null;
    nearestCorticalMm: number | null;
    severity: SafetySeverity;
    notes: string[];
}

export type Vec3 = [number, number, number];

function distancePointToSegment(p: Vec3, a: Vec3, b: Vec3): number {
    const abx = b[0] - a[0];
    const aby = b[1] - a[1];
    const abz = b[2] - a[2];
    const apx = p[0] - a[0];
    const apy = p[1] - a[1];
    const apz = p[2] - a[2];
    const lenSq = abx * abx + aby * aby + abz * abz || 1;
    let t = (apx * abx + apy * aby + apz * abz) / lenSq;
    t = Math.max(0, Math.min(1, t));
    const cx = a[0] + abx * t;
    const cy = a[1] + aby * t;
    const cz = a[2] + abz * t;
    return Math.hypot(p[0] - cx, p[1] - cy, p[2] - cz);
}

/** Minimum distance from a point to any segment of a polyline. */
export function distancePointToPolyline(p: Vec3, polyline: ReadonlyArray<Vec3>): number {
    if (polyline.length < 2) return Number.POSITIVE_INFINITY;
    let min = Number.POSITIVE_INFINITY;
    for (let i = 0; i < polyline.length - 1; i++) {
        const d = distancePointToSegment(p, polyline[i], polyline[i + 1]);
        if (d < min) min = d;
    }
    return min;
}

/** Compute severity from raw distances + active thresholds. */
export function severityFor(
    distances: { nerveMm?: number | null; sinusMm?: number | null; corticalMm?: number | null },
): SafetySeverity {
    const t = thresholds;
    const checks: Array<[number | null | undefined, number, number]> = [
        [distances.nerveMm, t.nerveErrorMm, t.nerveWarningMm],
        [distances.sinusMm, t.sinusErrorMm, t.sinusWarningMm],
        [distances.corticalMm, t.corticalErrorMm, t.corticalWarningMm],
    ];
    let worst: SafetySeverity = 'ok';
    for (const [value, errorThr, warningThr] of checks) {
        if (value === null || value === undefined) continue;
        if (value < errorThr) return 'error';
        if (value < warningThr) worst = 'warning';
    }
    return worst;
}

export interface SafetyAssessmentInput {
    fdi: number;
    /** Sample points along the implant axis (apex + coronal at minimum). */
    samplePoints: ReadonlyArray<Vec3>;
    nervePolylines?: ReadonlyArray<ReadonlyArray<Vec3>>;
    sinusSurfaceSamples?: ReadonlyArray<Vec3>;
    corticalSurfaceSamples?: ReadonlyArray<Vec3>;
}

export function assessImplantSafety(input: SafetyAssessmentInput): SafetyAssessment {
    const minOver = (
        candidates: ReadonlyArray<ReadonlyArray<Vec3>> | undefined,
        per: (samples: ReadonlyArray<Vec3>) => number,
    ): number | null => {
        if (!candidates || candidates.length === 0) return null;
        let best = Number.POSITIVE_INFINITY;
        for (const samples of candidates) {
            const d = per(samples);
            if (d < best) best = d;
        }
        return Number.isFinite(best) ? best : null;
    };

    const nerveDist = minOver(input.nervePolylines, (poly) => {
        let best = Number.POSITIVE_INFINITY;
        for (const p of input.samplePoints) {
            const d = distancePointToPolyline(p, poly);
            if (d < best) best = d;
        }
        return best;
    });

    const surfaceMin = (samples: ReadonlyArray<Vec3>): number | null => {
        if (samples.length === 0) return null;
        let best = Number.POSITIVE_INFINITY;
        for (const p of input.samplePoints) {
            for (const s of samples) {
                const d = Math.hypot(p[0] - s[0], p[1] - s[1], p[2] - s[2]);
                if (d < best) best = d;
            }
        }
        return Number.isFinite(best) ? best : null;
    };

    const sinusDist = input.sinusSurfaceSamples ? surfaceMin(input.sinusSurfaceSamples) : null;
    const corticalDist = input.corticalSurfaceSamples
        ? surfaceMin(input.corticalSurfaceSamples)
        : null;

    const severity = severityFor({
        nerveMm: nerveDist,
        sinusMm: sinusDist,
        corticalMm: corticalDist,
    });

    const notes: string[] = [];
    const t = thresholds;
    if (nerveDist !== null && nerveDist < t.nerveWarningMm) {
        notes.push(`Nerve clearance ${nerveDist.toFixed(2)} mm`);
    }
    if (sinusDist !== null && sinusDist < t.sinusWarningMm) {
        notes.push(`Sinus clearance ${sinusDist.toFixed(2)} mm`);
    }
    if (corticalDist !== null && corticalDist < t.corticalWarningMm) {
        notes.push(`Cortical clearance ${corticalDist.toFixed(2)} mm`);
    }

    return {
        fdi: input.fdi,
        nearestNerveMm: nerveDist,
        nearestSinusMm: sinusDist,
        nearestCorticalMm: corticalDist,
        severity,
        notes,
    };
}

/** UI ramp color helper. */
export function severityColor(s: SafetySeverity): string {
    if (s === 'ok') return '#22c55e';
    if (s === 'warning') return '#fbbf24';
    return '#ef4444';
}
