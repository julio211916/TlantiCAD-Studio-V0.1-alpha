/**
 * Viewer layout modes. The shell renders a different viewport tree per mode;
 * `application/` use cases decide which mode is valid for a given study.
 */

export type ViewerMode =
    | 'review'     // 2D stack scroll + W/L + measurements
    | 'mpr'        // Triple orthogonal MPR (axial + coronal + sagittal)
    | 'volume'     // 3D volume rendering with preset
    | 'panoramic'  // Curved reconstruction dental panorama
    | 'ai-report'; // AI segmentation + findings overlay

export interface ViewerModeCapability {
    mode: ViewerMode;
    /** Is this mode valid for a series with given shape? */
    canApply(params: {
        sliceCount: number;
        isVolumetric: boolean;
        modality: string;
    }): boolean;
    /** User-facing reason when it can't apply. */
    disabledReason(params: {
        sliceCount: number;
        isVolumetric: boolean;
        modality: string;
    }): string | null;
}

export const VIEWER_MODE_CAPABILITIES: ViewerModeCapability[] = [
    {
        mode: 'review',
        canApply: () => true,
        disabledReason: () => null,
    },
    {
        mode: 'mpr',
        canApply: ({ sliceCount, isVolumetric }) => sliceCount >= 8 && isVolumetric,
        disabledReason: ({ sliceCount, isVolumetric }) => {
            if (sliceCount < 8) return 'MPR requires at least 8 slices.';
            if (!isVolumetric) return 'MPR requires a volumetric series with consistent spacing.';
            return null;
        },
    },
    {
        mode: 'volume',
        canApply: ({ sliceCount, isVolumetric }) => sliceCount >= 16 && isVolumetric,
        disabledReason: ({ sliceCount, isVolumetric }) => {
            if (sliceCount < 16) return 'Volume rendering requires at least 16 slices.';
            if (!isVolumetric) return 'Volume rendering requires volumetric data.';
            return null;
        },
    },
    {
        mode: 'panoramic',
        canApply: ({ sliceCount, modality }) =>
            sliceCount >= 32 && (modality === 'CT' || modality === 'CBCT'),
        disabledReason: ({ sliceCount, modality }) => {
            if (modality !== 'CT' && modality !== 'CBCT')
                return 'Panoramic reconstruction needs a CT/CBCT series.';
            if (sliceCount < 32) return 'Panoramic reconstruction needs at least 32 axial slices.';
            return null;
        },
    },
    {
        mode: 'ai-report',
        canApply: ({ sliceCount, modality }) =>
            sliceCount >= 16 && (modality === 'CT' || modality === 'CBCT' || modality === 'MR'),
        disabledReason: ({ sliceCount, modality }) => {
            if (sliceCount < 16) return 'AI report requires a volumetric series (≥16 slices).';
            if (!['CT', 'CBCT', 'MR'].includes(modality))
                return `AI report not available for modality ${modality}.`;
            return null;
        },
    },
];

export function resolveAvailableModes(params: {
    sliceCount: number;
    isVolumetric: boolean;
    modality: string;
}): Array<{ mode: ViewerMode; available: boolean; disabledReason: string | null }> {
    return VIEWER_MODE_CAPABILITIES.map((cap) => ({
        mode: cap.mode,
        available: cap.canApply(params),
        disabledReason: cap.disabledReason(params),
    }));
}
