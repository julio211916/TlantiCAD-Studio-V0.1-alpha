/**
 * Crown Bottom wizard-step parameters (V30).
 * Reference: exocad "Crown Bottoms" doc — GAP / BORDER / ADVANCED tabs.
 */

import type { DentalMaterialType } from '@/lib/dental-workflow';

export type CrownBottomTab = 'gap' | 'border' | 'advanced';

export type CementBrushKind = 'no-cement' | 'cement' | 'zone-3';

export interface CementZone {
    id: CementBrushKind;
    label: string;
    color: string;
    gapMm: number;
}

export interface CrownBottomGapParams {
    /** The active brush the user paints with. */
    activeBrush: CementBrushKind;
    zones: Record<CementBrushKind, CementZone>;
    distanceFromMarginMm: number;
    axialSpacingMm: number;
    radialSpacingMm: number;
    lockAxialRadial: boolean;
}

export interface CrownBottomBorderParams {
    horizontalMm: number; // 1
    angledMm: number;     // 2
    angleDeg: number;     // 3
    verticalMm: number;   // 4
    belowMarginMm: number; // 5
}

export interface CrownBottomAdvancedParams {
    dontBlockOutUndercuts: boolean;
    blockOutAngleDeg: number;
    protectedZoneSizeMm: number;
    anticipateMilling: boolean;
    toolDiameterMm: number;
    bullnoseTool: boolean;
    toolTipFlatPercent: number;
    showUndercuts: boolean;
}

export interface CrownBottomParams {
    gap: CrownBottomGapParams;
    border: CrownBottomBorderParams;
    advanced: CrownBottomAdvancedParams;
}

/** Per-material safe defaults — sourced from exocad shipped parameter profiles. */
const MATERIAL_DEFAULTS: Partial<Record<DentalMaterialType, Partial<CrownBottomBorderParams> & Partial<CrownBottomAdvancedParams>>> = {
    zirconia: { horizontalMm: 0.2, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.2 },
    'zirconia-multilayer': { horizontalMm: 0.25, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.2 },
    'lithium-disilicate': { horizontalMm: 0.3, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.0 },
    pmma: { horizontalMm: 0.2, angledMm: 0.4, angleDeg: 55, toolDiameterMm: 1.2 },
    wax: { horizontalMm: 0.2, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.0 },
    composite: { horizontalMm: 0.25, angledMm: 0.3, angleDeg: 55, toolDiameterMm: 1.0 },
    'np-metal': { horizontalMm: 0.18, angledMm: 0.3, angleDeg: 65, toolDiameterMm: 1.2 },
    titanium: { horizontalMm: 0.25, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.2 },
    '3d-print': { horizontalMm: 0.2, angledMm: 0.3, angleDeg: 60, toolDiameterMm: 1.0 },
};

export function defaultCrownBottomParams(material: DentalMaterialType | null | undefined): CrownBottomParams {
    const mat = material ? MATERIAL_DEFAULTS[material] ?? {} : {};
    return {
        gap: {
            activeBrush: 'cement',
            zones: {
                'no-cement': { id: 'no-cement', label: 'No cement gap', color: '#4ade80', gapMm: 0 },
                'cement': { id: 'cement', label: 'Gap', color: '#facc15', gapMm: 0.08 },
                'zone-3': { id: 'zone-3', label: 'Zone 3', color: '#f472b6', gapMm: 0.12 },
            },
            distanceFromMarginMm: 1.0,
            axialSpacingMm: 0.02,
            radialSpacingMm: 0.02,
            lockAxialRadial: true,
        },
        border: {
            horizontalMm: mat.horizontalMm ?? 0.2,
            angledMm: mat.angledMm ?? 0.3,
            angleDeg: mat.angleDeg ?? 60,
            verticalMm: 0,
            belowMarginMm: 0,
        },
        advanced: {
            dontBlockOutUndercuts: false,
            blockOutAngleDeg: 0,
            protectedZoneSizeMm: 0.1,
            anticipateMilling: true,
            toolDiameterMm: mat.toolDiameterMm ?? 1.2,
            bullnoseTool: false,
            toolTipFlatPercent: 0,
            showUndercuts: false,
        },
    };
}

/** Clinician warning if param violates material minimum. */
export function validateBorder(params: CrownBottomBorderParams, material: DentalMaterialType | null): string | null {
    if (material === 'zirconia' && params.horizontalMm < 0.2) {
        return 'Zirconia requires a horizontal border ≥ 0.20 mm.';
    }
    if (params.horizontalMm < 0.1) {
        return 'Horizontal border below 0.1 mm is clinically unsafe.';
    }
    return null;
}
