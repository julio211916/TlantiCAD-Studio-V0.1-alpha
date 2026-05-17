/**
 * Pre-op scan + Waxup workflows — domain (V173-V174).
 *
 * Two distinct copy strategies for predefined tooth shapes (see exocad doc:
 * "Understanding the difference between Pre-op and Waxup feature").
 *
 * - **Pre-op scan**: load an extra scan of the un-prepped teeth, place tooth
 *   models close to it, then iteratively morph tooth models into the scan.
 *   Connectors are NOT copied; they are designed the classical way.
 * - **Waxup**: `digital copy milling` — no tooth models are used. Output is
 *   created directly from the wax scan after closing holes / cropping.
 *   Connectors present in the wax are copied verbatim.
 */

export type CopyStrategy = 'preop' | 'waxup';

/** Per-tooth choice of which strategy applies. */
export interface ToothCopyDecision {
    fdi: number;
    strategy: CopyStrategy | 'none';
}

export type Mat4 = [
    [number, number, number, number],
    [number, number, number, number],
    [number, number, number, number],
    [number, number, number, number],
];

export interface PreopAlignment {
    transformMatrix: Mat4;
    rmsMm: number;
    backend: string;
}

export interface PreopAdaptResult {
    converged: boolean;
    iterationsRun: number;
    rmsMm: number;
    backend: string;
}

export interface WaxupPreparation {
    preparedPath: string;
    holesClosed: number;
    cropped: boolean;
    backend: string;
    warnings: string[];
}

export interface PreopWaxupState {
    activeStrategy: CopyStrategy;
    /** Stop the iterative morph loop when the operator says it looks good. */
    isAdapting: boolean;
    iterations: number;
    alignment: PreopAlignment | null;
    lastAdapt: PreopAdaptResult | null;
    waxup: WaxupPreparation | null;
    error: string | null;
}

export function initialPreopWaxupState(): PreopWaxupState {
    return {
        activeStrategy: 'preop',
        isAdapting: false,
        iterations: 50,
        alignment: null,
        lastAdapt: null,
        waxup: null,
        error: null,
    };
}

/**
 * Validates a waxup mesh path before sending. Mirrors the troubleshooting
 * notes from the exocad doc — empty path, wrong extension, or missing margin
 * data are common failure modes.
 */
export function validateWaxupInput(input: {
    waxupPath: string;
    extensionAllowed?: string[];
}): string | null {
    if (!input.waxupPath.trim()) return 'Waxup path is required';
    const allowed = input.extensionAllowed ?? ['.stl', '.ply', '.obj'];
    const lower = input.waxupPath.toLowerCase();
    if (!allowed.some((ext) => lower.endsWith(ext))) {
        return `Waxup must be one of: ${allowed.join(', ')}`;
    }
    return null;
}
