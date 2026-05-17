/**
 * Articulator library — domain (V208).
 *
 * Catalogs the 19 physical articulator vendors bundled in
 * `public/library/articulator/`. Selecting a vendor in the picker overrides
 * the Bonwill defaults of the active ArticulatorConfig.
 */

import type { ArticulatorConfig } from '../../articulator';

export interface ArticulatorVendorEntry {
    id: string;
    label: string;
    vendor: string;
    adjustable: boolean;
    intercondylarDistanceMm?: number | null;
    anteriorPosteriorDistanceMm?: number | null;
    rotationPointShiftMm?: number | null;
    hasMeshes?: boolean;
    folderName?: string;
}

export interface ArticulatorPreset {
    id: string;
    label: string;
    vendor: string;
    config: ArticulatorConfig;
    backend: string;
}

export interface ArticulatorLibraryState {
    activeVendorId: string | null;
    vendors: ArticulatorVendorEntry[];
    activePreset: ArticulatorPreset | null;
    isLoading: boolean;
    error: string | null;
    backend: 'filesystem' | 'mock' | null;
}

export function defaultArticulatorLibraryState(): ArticulatorLibraryState {
    return {
        activeVendorId: null,
        vendors: [],
        activePreset: null,
        isLoading: false,
        error: null,
        backend: null,
    };
}

/** Group vendors by manufacturer for the UI list. */
export function groupVendors(
    vendors: readonly ArticulatorVendorEntry[],
): Record<string, ArticulatorVendorEntry[]> {
    const out: Record<string, ArticulatorVendorEntry[]> = {};
    for (const v of vendors) {
        const key = v.vendor || 'Other';
        (out[key] ??= []).push(v);
    }
    for (const key of Object.keys(out)) {
        out[key].sort((a, b) => a.label.localeCompare(b.label));
    }
    return out;
}
