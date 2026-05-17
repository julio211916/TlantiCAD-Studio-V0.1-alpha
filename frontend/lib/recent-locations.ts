/**
 * Recent locations registry — V264.
 *
 * Persists up to 8 recently-opened DICOM/case folders in localStorage. Used
 * by the Open / Location / Cancel dialog to populate the list.
 */

const STORAGE_KEY = 'tlanticad.recent-locations';
const MAX_ENTRIES = 8;

export interface RecentLocation {
    /** Absolute filesystem path. */
    path: string;
    /** Display label — usually the folder name. */
    label: string;
    /** Last-opened timestamp (ms). */
    openedAt: number;
    /** Optional case number / patient name displayed alongside the path. */
    caseRef?: string;
}

function readRaw(): RecentLocation[] {
    if (typeof window === 'undefined') return [];
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];
        return parsed.filter(
            (entry): entry is RecentLocation =>
                entry &&
                typeof entry.path === 'string' &&
                typeof entry.label === 'string' &&
                typeof entry.openedAt === 'number',
        );
    } catch {
        return [];
    }
}

function writeRaw(entries: RecentLocation[]): void {
    if (typeof window === 'undefined') return;
    try {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
    } catch {
        // ignore quota / private mode
    }
}

export function listRecentLocations(): RecentLocation[] {
    return readRaw().sort((a, b) => b.openedAt - a.openedAt);
}

export function rememberLocation(entry: Omit<RecentLocation, 'openedAt'>): void {
    const existing = readRaw().filter((e) => e.path !== entry.path);
    const next = [{ ...entry, openedAt: Date.now() }, ...existing].slice(0, MAX_ENTRIES);
    writeRaw(next);
}

export function forgetLocation(path: string): void {
    const next = readRaw().filter((e) => e.path !== path);
    writeRaw(next);
}

export function clearRecentLocations(): void {
    writeRaw([]);
}
