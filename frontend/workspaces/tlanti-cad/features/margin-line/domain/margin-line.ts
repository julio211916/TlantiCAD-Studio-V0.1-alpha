/**
 * Pure domain types for the preparation margin-line Wizard step.
 * Reference: exocad DentalCAD "Margin Line Detection".
 */

export interface Vec3 {
    x: number;
    y: number;
    z: number;
}

export type MarginMode = 'supragingival' | 'subgingival';

export type MarginTool = 'detect' | 'correct-draw' | 'repair-draw';

export type DrawMode = 'free' | 'magnetic' | 'edit';

export interface MarginLine {
    /** FDI tooth number (11..48) this margin belongs to. */
    toothFdi: number;
    polyline: Vec3[];
    /** True when the first and last points should be rendered as connected. */
    closed: boolean;
    mode: MarginMode;
    /** Server-reported confidence 0..1. */
    confidence: number;
    /** `trimesh` when the real detector ran; `mock` when a placeholder was used. */
    backend: 'trimesh' | 'mock';
}

export interface MarginDetectInput {
    meshPath: string;
    seed: Vec3;
    mode: MarginMode;
    maxIterations?: number;
}

export interface MarginCorrectInput {
    meshPath: string;
    seedPoints: Vec3[];
    mode: MarginMode;
}

export interface MarginRepairInput {
    meshPath: string;
    polyline: Vec3[];
    dragRadius?: number;
    surfaceSnapDistance?: number;
    repairRegionAroundMargin?: number;
}

/** Perimeter in world units — used by the UI to display a quality hint. */
export function polylinePerimeterMm(polyline: Vec3[]): number {
    if (polyline.length < 2) return 0;
    let total = 0;
    for (let i = 0; i < polyline.length; i += 1) {
        const a = polyline[i];
        const b = polyline[(i + 1) % polyline.length];
        total += Math.hypot(b.x - a.x, b.y - a.y, b.z - a.z);
    }
    return total;
}

/** Smallest-bounding-box diagonal — lets the camera auto-frame the margin. */
export function polylineBoundsDiagonal(polyline: Vec3[]): number {
    if (polyline.length === 0) return 0;
    let minX = Infinity, minY = Infinity, minZ = Infinity;
    let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;
    for (const p of polyline) {
        if (p.x < minX) minX = p.x;
        if (p.y < minY) minY = p.y;
        if (p.z < minZ) minZ = p.z;
        if (p.x > maxX) maxX = p.x;
        if (p.y > maxY) maxY = p.y;
        if (p.z > maxZ) maxZ = p.z;
    }
    return Math.hypot(maxX - minX, maxY - minY, maxZ - minZ);
}
