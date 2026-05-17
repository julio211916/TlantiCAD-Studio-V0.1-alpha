import type { ArticulatorConfig, JawFrame } from '../domain/articulator-config';
import type {
    ArticulatorPort,
    ArticulatorSimulateInput,
    ArticulatorSimulateOutput,
} from '../application/articulator-port';

import { BACKEND_ORIGIN } from '../../../lib/backend-config';

const DEFAULT_BASE_URL = BACKEND_ORIGIN;

interface RawJawFrame {
    t: number;
    translation_mm: number[];
    rotation_deg: number[];
}

interface RawSimulate {
    frames: RawJawFrame[];
    backend: string;
}

interface RawConfig {
    condyleInclinationDeg: number;
    bennettAngleDeg: number;
    immediateSideShiftMm: number;
    intercondylarDistanceMm: number;
    incisalGuidanceDeg: number;
}

async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
    const response = await fetch(url, {
        ...init,
        headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    });
    if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → HTTP ${response.status}`);
    return (await response.json()) as T;
}

function toFrame(raw: RawJawFrame): JawFrame {
    return {
        t: raw.t,
        translationMm: [raw.translation_mm[0], raw.translation_mm[1], raw.translation_mm[2]],
        rotationDeg: [raw.rotation_deg[0], raw.rotation_deg[1], raw.rotation_deg[2]],
    };
}

export function createBackendArticulatorAdapter(
    baseUrl: string = DEFAULT_BASE_URL,
): ArticulatorPort {
    return {
        async getConfig(): Promise<ArticulatorConfig> {
            return callJson<ArticulatorConfig>(`${baseUrl}/cad/articulator/config`);
        },
        async simulate(input: ArticulatorSimulateInput): Promise<ArticulatorSimulateOutput> {
            const raw = await callJson<RawSimulate>(`${baseUrl}/cad/articulator/simulate`, {
                method: 'POST',
                body: JSON.stringify({
                    config: input.config,
                    movement: input.movement,
                    frames: input.frames,
                }),
            });
            return { frames: raw.frames.map(toFrame), backend: raw.backend };
        },
        async setInfluencingTeeth(fdis: number[]): Promise<{ ok: boolean; count: number }> {
            return callJson<{ ok: boolean; count: number }>(
                `${baseUrl}/cad/articulator/influencing-teeth`,
                { method: 'POST', body: JSON.stringify({ fdis }) },
            );
        },
    };
}
