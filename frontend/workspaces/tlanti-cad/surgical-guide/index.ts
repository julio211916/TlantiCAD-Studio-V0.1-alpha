/**
 * Surgical guide STL export (V272).
 *
 * Lofts a cylindrical sleeve around each planned implant axis + boolean-
 * unions with the base model. The backend writes the STL to
 * `<case>/exports/surgical-guide.stl` ready for 3D printing.
 */

import { BACKEND_ORIGIN } from '../../lib/backend-config';

export interface SurgicalGuideImplant {
    fdi: number;
    apex: [number, number, number];
    coronal: [number, number, number];
    diameterMm: number;
    sleeveDiameterMm: number;
    sleeveHeightMm: number;
}

export interface SurgicalGuideExportInput {
    caseFolderPath: string;
    baseModelPath: string;
    implants: SurgicalGuideImplant[];
    wallThicknessMm?: number;
}

export interface SurgicalGuideSleeve {
    fdi: number;
    diameterMm: number;
    heightMm: number;
    anchorPosition: [number, number, number];
}

export interface SurgicalGuideExportOutput {
    outputPath: string;
    sleeves: SurgicalGuideSleeve[];
    triangleCount: number;
    backend: string;
}

export interface SurgicalGuidePort {
    export(input: SurgicalGuideExportInput): Promise<SurgicalGuideExportOutput>;
}

export function createBackendSurgicalGuideAdapter(
    baseUrl: string = BACKEND_ORIGIN,
): SurgicalGuidePort {
    return {
        async export(input) {
            const response = await fetch(`${baseUrl}/cad/surgical-guide/export`, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                    caseFolderPath: input.caseFolderPath,
                    baseModelPath: input.baseModelPath,
                    implants: input.implants,
                    wallThicknessMm: input.wallThicknessMm ?? 2.0,
                }),
            });
            if (!response.ok) throw new Error(`POST /cad/surgical-guide/export → ${response.status}`);
            return await response.json();
        },
    };
}

/**
 * Recommended sleeve diameter per implant diameter.
 * Standard manufacturers ship sleeves 0.6 mm wider than the implant
 * platform to leave room for the burr + cooling.
 */
export function recommendedSleeveDiameter(implantDiameterMm: number): number {
    return implantDiameterMm + 0.6;
}
