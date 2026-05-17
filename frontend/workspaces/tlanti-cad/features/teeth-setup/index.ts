/**
 * Teeth setup (V269) — virtual arch overlay on the CBCT volume.
 *
 * The lab tech picks N FDIs to place; the backend matches against a tooth
 * library (`public/library/teeth/<id>/`) and returns per-tooth transform
 * matrices. The R3F overlay (future) renders the placed teeth.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export type Mat4 = [
    [number, number, number, number],
    [number, number, number, number],
    [number, number, number, number],
    [number, number, number, number],
];

export interface TeethSetupMatch {
    fdi: number;
    libraryPath: string;
    transformMatrix: Mat4;
    /** Match confidence 0..1 — colours the overlay (red < 0.5 < amber < 0.8 < green). */
    confidence: number;
}

export interface TeethSetupPort {
    match(input: {
        volumePath: string;
        fdis: readonly number[];
        libraryId?: string;
    }): Promise<{ matches: TeethSetupMatch[]; backend: string }>;
}

export function createBackendTeethSetupAdapter(baseUrl: string = BACKEND_ORIGIN): TeethSetupPort {
    return {
        async match(input) {
            const response = await fetch(`${baseUrl}/cad/teeth-setup/match`, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                    volumePath: input.volumePath,
                    fdis: [...input.fdis],
                    libraryId: input.libraryId ?? 'generic',
                }),
            });
            if (!response.ok) throw new Error(`POST /cad/teeth-setup/match → ${response.status}`);
            return await response.json();
        },
    };
}

/** Confidence → color helper for the overlay glyphs. */
export function confidenceColor(confidence: number): string {
    if (confidence < 0.5) return '#ef4444';
    if (confidence < 0.8) return '#fbbf24';
    return '#22c55e';
}
