/**
 * Show Distances domain — V204.
 *
 * Owns the value types for the contact / intersection / proximity
 * visualization between a restoration and its neighbors.
 */

export type DistanceMode =
    | 'interference-contacts'
    | 'spacing'
    | 'contact-areas'
    | 'distance-to-scan'
    | 'thickness';

export type DynamicChannel = 'protrusive' | 'laterotrusive-l' | 'laterotrusive-r' | 'retrusive';

export interface DistanceStats {
    label: string;
    minMm: number;
    maxMm: number;
    meanMm: number;
    intersectionCount: number;
}

export interface DistanceVisualizationState {
    /** Master toggle for the whole visualization. */
    enabled: boolean;
    /** false-color scale half-range (mm). 0..colorScaleMm = blue→red. */
    colorScaleMm: number;
    showAntagonist: boolean;
    showMesial: boolean;
    showDistal: boolean;
    includeHealthy: boolean;
    mode: DistanceMode;
    dynamicEnabled: boolean;
    dynamicChannels: Set<DynamicChannel>;
    stats: DistanceStats[];
    isBusy: boolean;
    error: string | null;
}

export function defaultDistanceVisualizationState(): DistanceVisualizationState {
    return {
        enabled: false,
        colorScaleMm: 0.5,
        showAntagonist: true,
        showMesial: true,
        showDistal: true,
        includeHealthy: false,
        mode: 'interference-contacts',
        dynamicEnabled: false,
        dynamicChannels: new Set(),
        stats: [],
        isBusy: false,
        error: null,
    };
}

/**
 * Returns the inclusive HSL color stops for the false-color bar at a given
 * scale. Blue (clearance) → green (touch) → red (intersection).
 */
export function colorbarStops(scaleMm: number): Array<{ pct: number; color: string; mm: number }> {
    return [
        { pct: 0, color: '#1d4ed8', mm: -scaleMm },
        { pct: 25, color: '#22d3ee', mm: -scaleMm * 0.5 },
        { pct: 50, color: '#22c55e', mm: 0 },
        { pct: 75, color: '#facc15', mm: scaleMm * 0.5 },
        { pct: 100, color: '#dc2626', mm: scaleMm },
    ];
}
