/**
 * Screw Holes domain (V92–V93 + V205).
 *
 * Per-tooth state mirrors exocad's 3-mode toggle:
 *   - 'anatomy' (white) → channel referenced to the anatomy
 *   - 'framework' (green) → channel referenced to the framework
 *   - 'none' (yellow) → no screw channel
 *
 * Plus global thickness / height knobs + per-tooth control-point overrides
 * captured under `customMm`.
 */

export type ScrewChannelMode = 'anatomy' | 'framework' | 'none';

export interface ScrewHolesParams {
    /** Master switch — "Cut screw hole into parts above abutments". */
    cutHoles: boolean;
    /** Additional radial offset (mm) for the cut. Range −0.3..0.5. */
    offsetMm: number;
    /**
     * Minimum diameter (mm) we try to preserve for the hole. exocad notes
     * this is a heuristic, recommends using values significantly larger than
     * strictly required. Range 0.5..3.
     */
    minDiameterMm: number;
    /** Wall thickness around the channel (mm). Per-tooth overrides via customMm. */
    thicknessMm: number;
    /**
     * Channel height delta above (positive) or below (negative) the
     * anatomy. Per-tooth overrides via customMm.
     */
    heightMm: number;
    /** All control points unlocked from the anatomy (Snap All Off). */
    snapAllOff: boolean;
}

export interface ScrewHoleToothState {
    fdi: string;
    /** Three-mode selection. */
    mode: ScrewChannelMode;
    /** Per-tooth override of thickness (mm). undefined → inherit global. */
    customThicknessMm?: number;
    /** Per-tooth override of height (mm). undefined → inherit global. */
    customHeightMm?: number;
    /** Whether the control points for this tooth are locked to the anatomy. */
    locked: boolean;
}

export function defaultScrewHolesParams(): ScrewHolesParams {
    return {
        cutHoles: true,
        offsetMm: 0,
        minDiameterMm: 1,
        thicknessMm: 0.5,
        heightMm: 0,
        snapAllOff: false,
    };
}

export function defaultToothState(fdi: string): ScrewHoleToothState {
    return { fdi, mode: 'anatomy', locked: true };
}

export function clampOffsetMm(value: number): number {
    if (Number.isNaN(value)) return 0;
    return Math.min(0.5, Math.max(-0.3, value));
}

export function clampMinDiameterMm(value: number): number {
    if (Number.isNaN(value)) return 1;
    return Math.min(3, Math.max(0.5, value));
}

export function clampThicknessMm(value: number): number {
    if (Number.isNaN(value)) return 0.5;
    return Math.min(2, Math.max(0.2, value));
}

export function clampHeightMm(value: number): number {
    if (Number.isNaN(value)) return 0;
    return Math.min(5, Math.max(-5, value));
}

export function summarizeScrewHoles(
    teeth: readonly ScrewHoleToothState[],
): { anatomy: number; framework: number; none: number } {
    let anatomy = 0;
    let framework = 0;
    let none = 0;
    for (const tooth of teeth) {
        if (tooth.mode === 'anatomy') anatomy += 1;
        else if (tooth.mode === 'framework') framework += 1;
        else none += 1;
    }
    return { anatomy, framework, none };
}

/** Cycle through the 3 modes when clicking a tooth: anatomy → framework → none → anatomy. */
export function cycleScrewMode(current: ScrewChannelMode): ScrewChannelMode {
    if (current === 'anatomy') return 'framework';
    if (current === 'framework') return 'none';
    return 'anatomy';
}

/** Apply a global default to all teeth (used by the Anatomy / Framework buttons). */
export function applyGlobalMode(
    teeth: readonly ScrewHoleToothState[],
    mode: ScrewChannelMode,
): ScrewHoleToothState[] {
    return teeth.map((t) => ({ ...t, mode }));
}

/**
 * V221 — Flatten the top of a channel: pin its custom height to the lowest
 * legal value (max of `params.heightMm` and the channel's locked min). This
 * mirrors exocad's "CTRL + click big green control point" gesture.
 */
export function flattenChannelTop(
    teeth: readonly ScrewHoleToothState[],
    fdi: string,
    minHeightMm: number = 0,
): ScrewHoleToothState[] {
    return teeth.map((t) =>
        t.fdi === fdi ? { ...t, customHeightMm: minHeightMm, locked: true } : t,
    );
}

/**
 * V219 — Update per-tooth thickness from a 3D widget drag. Clamps to a sane
 * minimum (≥ 0.2mm) so the channel never collapses below printable thickness.
 */
export function setChannelThickness(
    teeth: readonly ScrewHoleToothState[],
    fdi: string,
    thicknessMm: number,
): ScrewHoleToothState[] {
    const clamped = Math.max(0.2, Math.min(2.0, thicknessMm));
    return teeth.map((t) =>
        t.fdi === fdi ? { ...t, customThicknessMm: clamped } : t,
    );
}

/** Toggle lock on a single tooth, or all teeth when `all=true` (Ctrl+click). */
export function toggleLocked(
    teeth: readonly ScrewHoleToothState[],
    fdi: string,
    options: { all?: boolean } = {},
): ScrewHoleToothState[] {
    const target = teeth.find((t) => t.fdi === fdi);
    if (!target) return [...teeth];
    const nextLocked = !target.locked;
    if (options.all) {
        return teeth.map((t) => ({ ...t, locked: nextLocked }));
    }
    return teeth.map((t) => (t.fdi === fdi ? { ...t, locked: nextLocked } : t));
}
