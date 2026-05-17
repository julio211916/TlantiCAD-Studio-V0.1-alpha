/**
 * Viewer presets for window/level + transfer function.
 *
 * CT-based presets use HU (Hounsfield Units) directly.
 * MRI presets use the intensity range of the volume since HU doesn't apply.
 *
 * These are values, not rendering code. `ui/presets/` translates them to
 * Cornerstone3D VOI LUT + opacity transfer functions.
 */

export type PresetKind = 'ct' | 'mri';

export interface CtPreset {
    id: string;
    kind: 'ct';
    label: string;
    /** Window width (HU). */
    ww: number;
    /** Window center (HU). */
    wc: number;
    /**
     * Opacity transfer function points [intensity, opacity 0..1].
     * Used by the 3D volume viewport only (VR preset).
     */
    opacityPoints: Array<[number, number]>;
    /** Cornerstone3D built-in VR preset name (optional). */
    volumePreset?: string;
}

export interface MriPreset {
    id: string;
    kind: 'mri';
    label: string;
    /** Window width in arbitrary intensity units. */
    ww: number;
    /** Window center in arbitrary intensity units. */
    wc: number;
}

export type VolumePreset = CtPreset | MriPreset;

/** CT/CBCT presets — cover the 4 most common dental workflows. */
export const CT_PRESETS: CtPreset[] = [
    {
        id: 'bone',
        kind: 'ct',
        label: 'Bone',
        ww: 2000,
        wc: 500,
        volumePreset: 'CT-Bone',
        opacityPoints: [
            [-1000, 0],
            [100, 0.0],
            [300, 0.6],
            [2000, 1.0],
        ],
    },
    {
        id: 'soft-tissue',
        kind: 'ct',
        label: 'Soft Tissue',
        ww: 400,
        wc: 40,
        volumePreset: 'CT-Soft-Tissue',
        opacityPoints: [
            [-1000, 0],
            [-60, 0.0],
            [80, 0.8],
            [400, 1.0],
        ],
    },
    {
        id: 'teeth',
        kind: 'ct',
        label: 'Teeth',
        ww: 3000,
        wc: 1500,
        volumePreset: 'CT-Bone',
        opacityPoints: [
            [-1000, 0],
            [500, 0.0],
            [1500, 0.9],
            [3500, 1.0],
        ],
    },
    {
        id: 'sinus',
        kind: 'ct',
        label: 'Sinus',
        ww: 2500,
        wc: -300,
        volumePreset: 'CT-Air',
        opacityPoints: [
            [-1000, 0],
            [-900, 0.1],
            [-300, 0.4],
            [500, 1.0],
        ],
    },
];

/** MRI presets — relative intensity; auto-applied when modality = MR. */
export const MRI_PRESETS: MriPreset[] = [
    { id: 't1', kind: 'mri', label: 'T1', ww: 600, wc: 300 },
    { id: 't2', kind: 'mri', label: 'T2', ww: 1200, wc: 600 },
    { id: 'stir', kind: 'mri', label: 'STIR', ww: 800, wc: 400 },
    { id: 'flair', kind: 'mri', label: 'FLAIR', ww: 1000, wc: 500 },
];

export function getPresetsForModality(modality: string | null): VolumePreset[] {
    const upper = (modality ?? 'CT').toUpperCase();
    if (upper === 'MR') return MRI_PRESETS;
    return CT_PRESETS;
}

export function findPresetById(presets: VolumePreset[], id: string): VolumePreset | null {
    return presets.find((preset) => preset.id === id) ?? null;
}
