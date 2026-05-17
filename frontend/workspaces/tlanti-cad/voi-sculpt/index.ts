/**
 * VOI sculpting (V265) — domain + port + adapter + panel.
 *
 * Polyline-lasso volume carve. The user draws a closed polygon on the
 * canvas; we send the screen-space polyline to the backend together with
 * the active volume path; the backend returns the count of voxels
 * removed + a path to the carved volume artifact.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

// ─── domain ────────────────────────────────────────────────────────────

export interface VoiSculptInput {
    volumePath: string;
    polylineScreen: Array<[number, number]>;
    invert?: boolean;
    featherVoxels?: number;
}

export interface VoiSculptOutput {
    voxelsRemoved: number;
    outputPath: string;
    backend: string;
}

// ─── application port ──────────────────────────────────────────────────

export interface VoiSculptPort {
    sculpt(input: VoiSculptInput): Promise<VoiSculptOutput>;
}

// ─── infrastructure ────────────────────────────────────────────────────

export function createBackendVoiSculptAdapter(baseUrl: string = BACKEND_ORIGIN): VoiSculptPort {
    return {
        async sculpt(input) {
            const response = await fetch(`${baseUrl}/cad/voi/sculpt`, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                    volumePath: input.volumePath,
                    polylineScreen: input.polylineScreen,
                    invert: input.invert ?? false,
                    featherVoxels: input.featherVoxels ?? 2,
                }),
            });
            if (!response.ok) throw new Error(`POST /cad/voi/sculpt → ${response.status}`);
            return (await response.json()) as VoiSculptOutput;
        },
    };
}
