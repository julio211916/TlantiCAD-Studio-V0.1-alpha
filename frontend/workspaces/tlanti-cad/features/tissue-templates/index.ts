/**
 * Tissue templates (V266) — domain + port + adapter + UI helpers.
 *
 * 6 default presets shipped by the backend (`/defaults`) + per-case savable
 * templates. The CBCT volume preview reads `windowCenter`, `windowWidth`,
 * `opacityPoints`, `colorRgb` to drive the volume-rendering material.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export interface TissueOpacityPoint {
    hu: number;
    opacity: number;
}

export interface TissueTemplate {
    id: string;
    label: string;
    windowCenter: number;
    windowWidth: number;
    opacityPoints: TissueOpacityPoint[];
    colorRgb: [number, number, number];
    isDefault?: boolean;
}

export interface TissueTemplatesPort {
    listDefaults(): Promise<TissueTemplate[]>;
    listForCase(caseFolderPath: string): Promise<TissueTemplate[]>;
    save(caseFolderPath: string, template: TissueTemplate): Promise<TissueTemplate>;
}

export function createBackendTissueTemplatesAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): TissueTemplatesPort {
    async function callJson<T>(url: string, init?: RequestInit): Promise<T> {
        const response = await fetch(url, {
            ...init,
            headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
        });
        if (!response.ok) throw new Error(`${init?.method ?? 'GET'} ${url} → ${response.status}`);
        return (await response.json()) as T;
    }
    return {
        async listDefaults() {
            const dto = await callJson<{ templates: TissueTemplate[] }>(
                `${baseUrl}/cad/tissue-templates/defaults`,
            );
            return dto.templates;
        },
        async listForCase(caseFolderPath) {
            const dto = await callJson<{ templates: TissueTemplate[] }>(
                `${baseUrl}/cad/tissue-templates/list`,
                { method: 'POST', body: JSON.stringify({ caseFolderPath }) },
            );
            return dto.templates;
        },
        async save(caseFolderPath, template) {
            return callJson<TissueTemplate>(`${baseUrl}/cad/tissue-templates/save`, {
                method: 'POST',
                body: JSON.stringify({ caseFolderPath, template }),
            });
        },
    };
}

/**
 * Pre-bake a 1-D opacity LUT (size 256) from the template's piece-wise
 * linear opacityPoints; can be uploaded as a Three.js DataTexture.
 */
export function buildOpacityLut(template: TissueTemplate, size = 256): Float32Array {
    const lut = new Float32Array(size);
    if (template.opacityPoints.length === 0) return lut;
    const pts = [...template.opacityPoints].sort((a, b) => a.hu - b.hu);
    const [minHu, maxHu] = [pts[0].hu, pts[pts.length - 1].hu];
    for (let i = 0; i < size; i++) {
        const t = i / (size - 1);
        const hu = minHu + t * (maxHu - minHu);
        // linear interpolate opacity at hu
        let lo = pts[0];
        let hi = pts[pts.length - 1];
        for (let j = 0; j < pts.length - 1; j++) {
            if (hu >= pts[j].hu && hu <= pts[j + 1].hu) {
                lo = pts[j];
                hi = pts[j + 1];
                break;
            }
        }
        const span = hi.hu - lo.hu || 1;
        const alpha = lo.opacity + ((hu - lo.hu) / span) * (hi.opacity - lo.opacity);
        lut[i] = alpha;
    }
    return lut;
}
