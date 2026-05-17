/**
 * Pure domain types for clipping planes.
 *
 * A clipping plane cuts a volume along one of the anatomical axes; offset
 * is expressed in world-units relative to the volume centre and negated
 * when `inverted` is true.
 */

export type ClipAxis = 'axial' | 'sagittal' | 'coronal';

export interface ClippingPlane {
    id: string;
    axis: ClipAxis;
    enabled: boolean;
    inverted: boolean;
    /** Normalised offset in the range [-1, 1]; 0 = through the volume centre. */
    offset: number;
}

export const DEFAULT_CLIPPING_PLANES: ClippingPlane[] = [
    { id: 'axial', axis: 'axial', enabled: false, inverted: false, offset: 0 },
    { id: 'sagittal', axis: 'sagittal', enabled: false, inverted: false, offset: 0 },
    { id: 'coronal', axis: 'coronal', enabled: false, inverted: false, offset: 0 },
];

export function togglePlane(planes: ClippingPlane[], id: string): ClippingPlane[] {
    return planes.map((p) => (p.id === id ? { ...p, enabled: !p.enabled } : p));
}

export function invertPlane(planes: ClippingPlane[], id: string): ClippingPlane[] {
    return planes.map((p) => (p.id === id ? { ...p, inverted: !p.inverted } : p));
}

export function offsetPlane(
    planes: ClippingPlane[],
    id: string,
    offset: number,
): ClippingPlane[] {
    const clamped = Math.min(1, Math.max(-1, offset));
    return planes.map((p) => (p.id === id ? { ...p, offset: clamped } : p));
}

export function resetPlanes(planes: ClippingPlane[]): ClippingPlane[] {
    return planes.map((p) => ({ ...p, enabled: false, inverted: false, offset: 0 }));
}
