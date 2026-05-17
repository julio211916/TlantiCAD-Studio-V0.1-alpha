/**
 * FDI tooth numbering (ISO 3950) — two-digit codes where the first digit
 * is the quadrant (1..4 permanent, 5..8 primary) and the second digit is
 * the tooth position counted from the midline.
 *
 *   Upper right: 18 17 16 15 14 13 12 11 │ 21 22 23 24 25 26 27 28  Upper left
 *   Lower right: 48 47 46 45 44 43 42 41 │ 31 32 33 34 35 36 37 38  Lower left
 *
 * Each entry carries an anatomical label used for tooltips and for the
 * automatic-segmentation result mapping.
 */

export type JawKind = 'maxilla' | 'mandible';
export type ToothCategory = 'incisor' | 'canine' | 'premolar' | 'molar';

export interface ToothDefinition {
    fdi: number;
    quadrant: 1 | 2 | 3 | 4;
    position: number; // 1..8 from midline
    jaw: JawKind;
    category: ToothCategory;
    label: string;
}

function make(
    fdi: number,
    quadrant: 1 | 2 | 3 | 4,
    position: number,
    category: ToothCategory,
    label: string,
): ToothDefinition {
    const jaw: JawKind = quadrant === 1 || quadrant === 2 ? 'maxilla' : 'mandible';
    return { fdi, quadrant, position, jaw, category, label };
}

/** Permanent dentition (32 teeth). */
export const PERMANENT_TEETH: ToothDefinition[] = [
    // Upper right (quadrant 1) — 18..11
    make(18, 1, 8, 'molar', 'Third molar'),
    make(17, 1, 7, 'molar', 'Second molar'),
    make(16, 1, 6, 'molar', 'First molar'),
    make(15, 1, 5, 'premolar', 'Second premolar'),
    make(14, 1, 4, 'premolar', 'First premolar'),
    make(13, 1, 3, 'canine', 'Canine'),
    make(12, 1, 2, 'incisor', 'Lateral incisor'),
    make(11, 1, 1, 'incisor', 'Central incisor'),
    // Upper left (quadrant 2) — 21..28
    make(21, 2, 1, 'incisor', 'Central incisor'),
    make(22, 2, 2, 'incisor', 'Lateral incisor'),
    make(23, 2, 3, 'canine', 'Canine'),
    make(24, 2, 4, 'premolar', 'First premolar'),
    make(25, 2, 5, 'premolar', 'Second premolar'),
    make(26, 2, 6, 'molar', 'First molar'),
    make(27, 2, 7, 'molar', 'Second molar'),
    make(28, 2, 8, 'molar', 'Third molar'),
    // Lower left (quadrant 3) — 38..31
    make(38, 3, 8, 'molar', 'Third molar'),
    make(37, 3, 7, 'molar', 'Second molar'),
    make(36, 3, 6, 'molar', 'First molar'),
    make(35, 3, 5, 'premolar', 'Second premolar'),
    make(34, 3, 4, 'premolar', 'First premolar'),
    make(33, 3, 3, 'canine', 'Canine'),
    make(32, 3, 2, 'incisor', 'Lateral incisor'),
    make(31, 3, 1, 'incisor', 'Central incisor'),
    // Lower right (quadrant 4) — 41..48
    make(41, 4, 1, 'incisor', 'Central incisor'),
    make(42, 4, 2, 'incisor', 'Lateral incisor'),
    make(43, 4, 3, 'canine', 'Canine'),
    make(44, 4, 4, 'premolar', 'First premolar'),
    make(45, 4, 5, 'premolar', 'Second premolar'),
    make(46, 4, 6, 'molar', 'First molar'),
    make(47, 4, 7, 'molar', 'Second molar'),
    make(48, 4, 8, 'molar', 'Third molar'),
];

export function teethOfJaw(jaw: JawKind): ToothDefinition[] {
    return PERMANENT_TEETH.filter((t) => t.jaw === jaw);
}

export function findTooth(fdi: number): ToothDefinition | null {
    return PERMANENT_TEETH.find((t) => t.fdi === fdi) ?? null;
}

/**
 * Logical state for a given tooth in the segmentation UI.
 *
 *   unsegmented — visible slot, not yet detected
 *   segmented   — AI produced a mask (paint with `color`)
 *   missing     — clinician marked as absent (greyed out)
 *   locked      — user pinned the crown, do not touch on re-run
 */
export type ToothStatus = 'unsegmented' | 'segmented' | 'missing' | 'locked';

export interface ToothState {
    fdi: number;
    status: ToothStatus;
    /** Hex colour for the segmentation overlay; null when unsegmented. */
    color: string | null;
}

/** Palette used by RealGUIDE — pastel per-tooth for easy visual separation. */
export const DEFAULT_TOOTH_PALETTE: string[] = [
    '#F4B8C4', '#FFE3A6', '#CFE9A6', '#A6E3D4', '#A6C9E3',
    '#C2A6E3', '#E3A6D0', '#E3B5A6', '#B8E3A6', '#A6DCE3',
    '#D0A6E3', '#E3A6B5', '#E3CBA6', '#A6E3B8', '#A6B5E3',
    '#E3A6C5', '#F4D4A6', '#C5E3A6', '#A6E3C5', '#A6B8E3',
    '#D9A6E3', '#E3A6B8', '#E3D9A6', '#A6E3D9', '#A6C5E3',
    '#C5A6E3', '#E3A6B8', '#E3B8A6', '#B8E3A6', '#A6E3E3',
    '#C5B8E3', '#F0C2A6',
];

export function defaultColorFor(fdi: number): string {
    const idx = PERMANENT_TEETH.findIndex((t) => t.fdi === fdi);
    if (idx < 0) return DEFAULT_TOOTH_PALETTE[0];
    return DEFAULT_TOOTH_PALETTE[idx % DEFAULT_TOOTH_PALETTE.length];
}
