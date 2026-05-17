/**
 * Sectional view domain — V224 + V226.
 *
 * A single global clipping plane shared between the main R3F canvas and the
 * cut-view companion. Position+normal in world space (mm). Drag handles on
 * the canvas mutate this value through the parent state.
 */

export interface SectionalPlane {
    enabled: boolean;
    /** Normal vector (world space). Should be unit-length. */
    normal: [number, number, number];
    /** Plane offset along the normal (mm). */
    offsetMm: number;
}

export function defaultSectionalPlane(): SectionalPlane {
    return {
        enabled: false,
        normal: [0, 1, 0], // top→down section by default
        offsetMm: 0,
    };
}

/** Snap the normal to the closest axis (X / Y / Z) — useful UX shortcut. */
export function snapNormalToAxis(
    normal: readonly [number, number, number],
): [number, number, number] {
    const [nx, ny, nz] = normal;
    const ax = Math.abs(nx);
    const ay = Math.abs(ny);
    const az = Math.abs(nz);
    if (ax >= ay && ax >= az) return [Math.sign(nx) || 1, 0, 0];
    if (ay >= az) return [0, Math.sign(ny) || 1, 0];
    return [0, 0, Math.sign(nz) || 1];
}
