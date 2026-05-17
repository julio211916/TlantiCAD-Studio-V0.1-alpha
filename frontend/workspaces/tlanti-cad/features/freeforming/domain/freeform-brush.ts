/**
 * Freeforming brush state (V40).
 * Scaffold types shared by the three tabs: Free (wax knife), Anatomic, Attachment.
 */

export type FreeformTab = 'free' | 'anatomic' | 'attachment';
export type FreeformMode = 'add-remove' | 'smooth-flatten' | 'adapt';
export type FreeformBrushType = 'round-ball' | 'pointed-knife' | 'flat-cylinder';

export interface FreeformBrushState {
    mode: FreeformMode;
    brushType: FreeformBrushType;
    /** 0..1 — maps to force/speed server-side. */
    strength: number;
    /** Brush radius in mm. */
    sizeMm: number;
}

export type AnatomicPreset = 'cusps' | 'tooth-parts' | 'entire-tooth' | 'ridge';
export type MovementRestriction = 'occlusal-only' | 'lock-cusp-tips' | 'lock-equator';

export interface AnatomicState {
    preset: AnatomicPreset;
    restrictions: Set<MovementRestriction>;
    /** Advanced paint-and-pull mode active. */
    advancedPaintPull: boolean;
}

export type AttachmentMode = 'add' | 'subtract';
export type InsertionDirectionSource = 'top' | 'view' | 'surface';

export interface AttachmentState {
    mode: AttachmentMode;
    library: string;
    type: string;
    insertionDirection: InsertionDirectionSource;
    cutOnGingiva: boolean;
    cutDistanceMm: number;
    isText: boolean;
    textValue: string;
    textThicknessMm: number;
    textSizeMm: number;
    textEmboss: boolean; // emboss (raised) vs deboss (engraved)
}

export interface FreeformState {
    tab: FreeformTab;
    brush: FreeformBrushState;
    anatomic: AnatomicState;
    attachment: AttachmentState;
    /** Ring buffer of past states for undo. */
    undoDepth: number;
    redoDepth: number;
}

export function defaultFreeformState(): FreeformState {
    return {
        tab: 'free',
        brush: {
            mode: 'add-remove',
            brushType: 'round-ball',
            strength: 0.5,
            sizeMm: 1.2,
        },
        anatomic: {
            preset: 'entire-tooth',
            restrictions: new Set(),
            advancedPaintPull: false,
        },
        attachment: {
            mode: 'add',
            library: 'generic',
            type: 'Default.sdfa',
            insertionDirection: 'top',
            cutOnGingiva: false,
            cutDistanceMm: 0,
            isText: false,
            textValue: '',
            textThicknessMm: 0.5,
            textSizeMm: 4,
            textEmboss: true,
        },
        undoDepth: 0,
        redoDepth: 0,
    };
}
