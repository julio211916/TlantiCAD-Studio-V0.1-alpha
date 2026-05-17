/**
 * Abutment design (V143) — domain.
 *
 * Reference: local notes for exocad-style abutment editing behavior.
 */

export type AbutmentStyle = 'cylindrical' | 'angular' | 'standard' | 'legacy';

export interface AbutmentStyleProfile {
    id: AbutmentStyle;
    label: string;
    description: string;
    /** Default shoulder/wall thickness (mm). */
    shoulderSizeMm: number;
    /** Default roundness (0=sharp, 1=fully rounded). */
    roundness: number;
    /** Minimum cone angle (deg) — avoids undercuts. */
    minimumAngleDeg: number;
}

export const ABUTMENT_STYLES: AbutmentStyleProfile[] = [
    {
        id: 'cylindrical',
        label: 'Cylindrical',
        description: 'Small shoulder, reduced angularity, smooth rounded edges',
        shoulderSizeMm: 0.4,
        roundness: 0.7,
        minimumAngleDeg: 4,
    },
    {
        id: 'angular',
        label: 'Angular',
        description: 'Moderate shoulder, pronounced angularity',
        shoulderSizeMm: 0.6,
        roundness: 0.25,
        minimumAngleDeg: 6,
    },
    {
        id: 'standard',
        label: 'Standard',
        description: 'Average shoulder, moderate angularity',
        shoulderSizeMm: 0.5,
        roundness: 0.5,
        minimumAngleDeg: 5,
    },
    {
        id: 'legacy',
        label: 'Legacy (3.2)',
        description: 'Replicates 3.2 software behavior, no advanced shape customization',
        shoulderSizeMm: 0.5,
        roundness: 0.5,
        minimumAngleDeg: 5,
    },
];

export type AbutmentTab = 'top' | 'bottom' | 'advanced';

export interface AbutmentTopParams {
    style: AbutmentStyle;
    /** Locks shoulder size when adjusting other parameters. */
    keepConstantShoulderSize: boolean;
    shoulderSizeMm: number;
    roundness: number;
    minimumAngleDeg: number;
    connectFissureControls: boolean;
    keepDesignWithinAnatomy: boolean;
    /** mm — distance to occlusal surface. */
    spacingMm: number;
    autoAdaptSpacing: boolean;
    /** Visualize distance to anatomy on the abutment surface. */
    showDistanceOnAbutment: boolean;
    /** Visualize distance on the anatomy (crown bottom). */
    showDistanceOnAnatomy: boolean;
}

/**
 * Shape preset for the emergence profile — left buttons in exocad doc:
 *   - 'concave-dished'  → concave (dished) shape, recommended for thin gingiva
 *   - 'standard'        → straight emergence
 *   - 'convex'          → convex shape, recommended for thick gingiva
 */
export type AbutmentBottomShape = 'concave-dished' | 'standard' | 'convex';

/** Pink (stick to gingiva) vs green (free) per-control-point state (V213). */
export type AbutmentControlPointStick = 'stick' | 'free';

export interface AbutmentBottomControlPoint {
    /** Index along the emergence margin polyline. */
    index: number;
    stick: AbutmentControlPointStick;
    /** Optional override of the upper/lower shape sliders. */
    customUpperMm?: number;
    customLowerMm?: number;
}

export interface AbutmentBottomParams {
    /** Same as Crown Bottoms — emergence profile parameters. */
    emergenceHeightMm: number;
    emergenceAngleDeg: number;
    contactPressureMm: number;
    /** V212 — shape preset shared by all abutments unless free-form active. */
    shape: AbutmentBottomShape;
    /** Upper-half slider (controls the upper part of the emergence). */
    upperShapeMm: number;
    /** Lower-half slider (controls the lower part). */
    lowerShapeMm: number;
    /** Free-form bottom mode — clicks add/remove material per abutment. */
    freeFormBottom: boolean;
    /** Distance-to-gingiva limit (mm) when visualization is active. */
    distanceLimitMm: number;
    /** Intersection-with-gingiva limit (mm). */
    intersectionLimitMm: number;
    /** Show distance-to-gingiva visualization (blue). */
    visualizeDistance: boolean;
    /** Show intersection-with-gingiva visualization (red). */
    visualizeIntersection: boolean;
    /** Per-control-point overrides (V213). */
    controlPoints: AbutmentBottomControlPoint[];
}

export type ScrewChannelMode = 'straight' | 'angulated-clickable' | 'angulated-draggable';

export interface AbutmentAdvancedParams {
    profileBorderHeightMm: number;
    profileBorderRadiusMm: number;
    /** Radians from the implant axis (0 = straight, max ~25°). */
    angulatedScrewChannelDeg: number;
    screwChannelMode: ScrewChannelMode;
    minThicknessMm: number;
    abutmentToolDiameterMm: number;
    superstructureToolDiameterMm: number;
    screwChannelDistanceMm: number;
    marginAngleDeg: number;
}

export interface AbutmentDesign {
    top: AbutmentTopParams;
    bottom: AbutmentBottomParams;
    advanced: AbutmentAdvancedParams;
}

export function defaultAbutmentDesign(style: AbutmentStyle = 'standard'): AbutmentDesign {
    const profile =
        ABUTMENT_STYLES.find((s) => s.id === style) ?? ABUTMENT_STYLES[2];
    return {
        top: {
            style: profile.id,
            keepConstantShoulderSize: false,
            shoulderSizeMm: profile.shoulderSizeMm,
            roundness: profile.roundness,
            minimumAngleDeg: profile.minimumAngleDeg,
            connectFissureControls: true,
            keepDesignWithinAnatomy: true,
            spacingMm: 0.6,
            autoAdaptSpacing: true,
            showDistanceOnAbutment: false,
            showDistanceOnAnatomy: false,
        },
        bottom: {
            emergenceHeightMm: 0.8,
            emergenceAngleDeg: 30,
            contactPressureMm: 0.05,
            shape: 'standard',
            upperShapeMm: 0.5,
            lowerShapeMm: 0.5,
            freeFormBottom: false,
            distanceLimitMm: 0.1,
            intersectionLimitMm: 0.1,
            visualizeDistance: false,
            visualizeIntersection: false,
            controlPoints: [],
        },
        advanced: {
            profileBorderHeightMm: 0.4,
            profileBorderRadiusMm: 0.15,
            angulatedScrewChannelDeg: 0,
            screwChannelMode: 'straight',
            minThicknessMm: 0.5,
            abutmentToolDiameterMm: 1.2,
            superstructureToolDiameterMm: 1.0,
            screwChannelDistanceMm: 0.8,
            marginAngleDeg: 60,
        },
    };
}

/**
 * V213 — toggle a single emergence control point between stick (pink) and
 * free (green). With `all=true`, the action mirrors exocad's Ctrl+click
 * shortcut (toggle every point at once).
 */
export function toggleControlPointStick(
    points: readonly AbutmentBottomControlPoint[],
    index: number,
    options: { all?: boolean } = {},
): AbutmentBottomControlPoint[] {
    const target = points.find((p) => p.index === index);
    if (!target) {
        // Auto-create a control point at the requested index, default stick.
        return [...points, { index, stick: 'stick' }];
    }
    const nextStick: AbutmentControlPointStick = target.stick === 'stick' ? 'free' : 'stick';
    if (options.all) {
        return points.map((p) => ({ ...p, stick: nextStick }));
    }
    return points.map((p) => (p.index === index ? { ...p, stick: nextStick } : p));
}

/**
 * Force every control point to a single stick mode — used by exocad's
 * "Unstick all" / "Restick all" buttons in the Advanced section.
 */
export function setAllControlPointsStick(
    points: readonly AbutmentBottomControlPoint[],
    stick: AbutmentControlPointStick,
): AbutmentBottomControlPoint[] {
    return points.map((p) => ({ ...p, stick }));
}

/**
 * Validates angulated screw channel angle. Most implant libraries support up
 * to 25 degrees; over that the export rejects the channel.
 */
export function validateScrewChannelAngle(deg: number): string | null {
    if (deg < 0) return 'Angle cannot be negative';
    if (deg > 25) return 'Most implant libraries cap angulated channels at 25°';
    if (deg > 20) return 'Above 20° → verify your implant library supports it';
    return null;
}
