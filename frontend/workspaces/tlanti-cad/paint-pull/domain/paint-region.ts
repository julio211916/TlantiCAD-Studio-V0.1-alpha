/**
 * Paint & Pull domain — V231.
 *
 * Three-color region editing for the freeforming Advanced Editing mode:
 *   - moving (green)  → vertices that follow the pull direction 1:1
 *   - elastic (yellow) → falloff band around moving — soft pull
 *   - static (blue)   → vertices that never move
 *
 * Per-vertex weights stored as a Float32Array, one entry per vertex of the
 * active mesh. The vertex index maps to `geometry.attributes.position` of
 * the same mesh; downstream the kernel uses this attribute as the weight in
 * an ARAP / linear-blend deformation.
 */

export type PaintRegion = 'moving' | 'elastic' | 'static';

export interface PaintPullState {
    /** Active brush region. */
    activeRegion: PaintRegion;
    /** Brush radius in mm. */
    brushSizeMm: number;
    /** Pull strength multiplier (0-1). Only used in 'pull' mode. */
    pullStrength: number;
    /**
     * Per-vertex tag — 0 = unassigned (default), 1 = moving, 2 = elastic,
     * 3 = static. Stored as Uint8Array to keep memory tight on large meshes.
     */
    vertexTags: Uint8Array | null;
    /** Vertex count this Uint8Array was allocated for. */
    vertexCount: number;
    /** True when the user is actively dragging the painted area. */
    isPulling: boolean;
}

export const REGION_COLORS: Record<PaintRegion, string> = {
    moving: '#22c55e',
    elastic: '#facc15',
    static: '#3b82f6',
};

const REGION_TO_TAG: Record<PaintRegion, number> = {
    moving: 1,
    elastic: 2,
    static: 3,
};

const TAG_TO_REGION: Record<number, PaintRegion | null> = {
    0: null,
    1: 'moving',
    2: 'elastic',
    3: 'static',
};

export function defaultPaintPullState(vertexCount = 0): PaintPullState {
    return {
        activeRegion: 'moving',
        brushSizeMm: 1.5,
        pullStrength: 0.6,
        vertexTags: vertexCount > 0 ? new Uint8Array(vertexCount) : null,
        vertexCount,
        isPulling: false,
    };
}

export function regionAt(state: PaintPullState, vertexIndex: number): PaintRegion | null {
    if (!state.vertexTags || vertexIndex >= state.vertexTags.length) return null;
    return TAG_TO_REGION[state.vertexTags[vertexIndex]] ?? null;
}

/**
 * Paint a vertex range with the active region. Mutates the buffer in place
 * and returns the updated state object (with new reference) so React can pick
 * the change up.
 */
export function paintVertices(
    state: PaintPullState,
    indices: Int32Array | number[],
    region: PaintRegion = state.activeRegion,
): PaintPullState {
    if (!state.vertexTags) return state;
    const tag = REGION_TO_TAG[region];
    for (let i = 0; i < indices.length; i++) {
        const idx = indices[i];
        if (idx >= 0 && idx < state.vertexTags.length) {
            state.vertexTags[idx] = tag;
        }
    }
    return { ...state, vertexTags: new Uint8Array(state.vertexTags) };
}

/** Clear all painted vertices. */
export function clearPaint(state: PaintPullState): PaintPullState {
    if (!state.vertexTags) return state;
    return {
        ...state,
        vertexTags: new Uint8Array(state.vertexTags.length),
        isPulling: false,
    };
}

/** Invert moving ↔ static (elastic stays). Elastic falloff band needs both extremes painted. */
export function invertPaint(state: PaintPullState): PaintPullState {
    if (!state.vertexTags) return state;
    const next = new Uint8Array(state.vertexTags.length);
    for (let i = 0; i < state.vertexTags.length; i++) {
        const t = state.vertexTags[i];
        if (t === 1) next[i] = 3;
        else if (t === 3) next[i] = 1;
        else next[i] = t;
    }
    return { ...state, vertexTags: next };
}

/** Count vertices per region — for the UI summary. */
export function paintHistogram(state: PaintPullState): Record<PaintRegion | 'unassigned', number> {
    const out: Record<PaintRegion | 'unassigned', number> = {
        moving: 0,
        elastic: 0,
        static: 0,
        unassigned: 0,
    };
    if (!state.vertexTags) {
        out.unassigned = state.vertexCount;
        return out;
    }
    for (let i = 0; i < state.vertexTags.length; i++) {
        const r = TAG_TO_REGION[state.vertexTags[i]];
        if (r) out[r] += 1;
        else out.unassigned += 1;
    }
    return out;
}
