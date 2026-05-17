/**
 * Pure domain for the Insertion Direction wizard step (V27–V29).
 */

export interface Vec3 {
    x: number;
    y: number;
    z: number;
}

export interface InsertionAxis {
    toothFdi: number;
    vector: Vec3;
    undercutVolumeMm3: number;
    undercutPeakMm: number;
    confidence: number;
    backend: 'trimesh-pca' | 'mock';
    /** Set when the tooth is part of a bridge with a shared axis. */
    uniqueForBridge?: boolean;
}

export interface UndercutStats {
    volumeMm3: number;
    peakMm: number;
}

export const UNDERCUT_LEGEND: Array<{ value: number; label: string; color: string }> = [
    { value: 0.0, label: '0', color: '#0044cc' },
    { value: 0.12, label: '0.12', color: '#00c2ff' },
    { value: 0.25, label: '0.25', color: '#46d66b' },
    { value: 0.38, label: '0.38', color: '#ffcb33' },
    { value: 0.5, label: '0.5', color: '#ff4d2d' },
];

/** Dot product between two axes; used to judge how divergent they are. */
export function axisDot(a: Vec3, b: Vec3): number {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

export function axisLength(v: Vec3): number {
    return Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
}

export function normaliseAxis(v: Vec3): Vec3 {
    const n = axisLength(v) || 1;
    return { x: v.x / n, y: v.y / n, z: v.z / n };
}

/** Angle in degrees between two axes. */
export function axisAngleDeg(a: Vec3, b: Vec3): number {
    const na = normaliseAxis(a);
    const nb = normaliseAxis(b);
    const d = Math.max(-1, Math.min(1, axisDot(na, nb)));
    return (Math.acos(d) * 180) / Math.PI;
}
