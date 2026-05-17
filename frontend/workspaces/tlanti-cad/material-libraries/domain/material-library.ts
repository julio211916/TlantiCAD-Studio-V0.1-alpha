/**
 * Material Library — domain (V206 + tlantishare rename).
 *
 * Surfaces local + tlantishare-peer material configs to the user. The
 * selected library drives subsequent material pickers everywhere in the app.
 * `tlantishare` libraries arrive over P2P from LAN peers, never from a
 * central cloud.
 */

export type MaterialLibrarySource = 'local' | 'tlantishare';

export interface MaterialLibraryEntry {
    id: string;
    label: string;
    vendor: string;
    materials: string[];
    lastUpdated: string;
    source: MaterialLibrarySource;
}

export interface MaterialLibraryState {
    activeLibraryId: string | null;
    local: MaterialLibraryEntry[];
    /** Libraries received via TlantiShare P2P from peers on the same LAN. */
    peerLibraries: MaterialLibraryEntry[];
    isLoading: boolean;
    error: string | null;
}

export function defaultMaterialLibraryState(): MaterialLibraryState {
    return {
        activeLibraryId: 'exocad-default',
        local: [],
        peerLibraries: [],
        isLoading: false,
        error: null,
    };
}

export function findLibrary(
    state: MaterialLibraryState,
    id: string,
): MaterialLibraryEntry | null {
    return (
        state.local.find((l) => l.id === id) ??
        state.peerLibraries.find((l) => l.id === id) ??
        null
    );
}
