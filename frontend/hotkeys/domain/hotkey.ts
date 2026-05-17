/**
 * Hotkey registry — domain (V203).
 *
 * Mirrors the exocad-faithful shortcuts catalogued in the local workflow
 * notes. Keys are normalized
 * lowercase; modifiers are sorted alphabetically (alt → ctrl → meta → shift)
 * to make the parser deterministic.
 */

export type HotkeyContext =
    | 'global'
    | 'wizard'
    | 'visibility'
    | 'crown-bottom'
    | 'free-form'
    | 'free-form-cut'
    | 'margin'
    | 'mesh-editor'
    | 'design-control';

export type HotkeyCategory =
    | 'file'
    | 'edit'
    | 'view'
    | 'selection'
    | 'tools'
    | 'navigation'
    | 'modelling'
    | 'system'
    | string;

export interface HotkeyDefinition {
    /** Canonical chord — modifiers + key, e.g. `ctrl+r` or `1`. */
    chord: string;
    /** Short label shown in `?`/F1 help overlay. */
    label: string;
    /** Long-form description. */
    description?: string;
    /** Where the hotkey is active. */
    context: HotkeyContext;
    /** The action to run. */
    run: (event: KeyboardEvent) => void;
    /** Optional predicate evaluated at fire-time. */
    available?: () => boolean;
    /** Whether the hotkey should be exposed to the command palette as well. */
    paletteAction?: {
        id: string;
        label: string;
        kind: 'tool' | 'navigation' | 'toggle' | 'wizard-step' | 'custom';
        keywords?: string[];
    };
}

/** Normalize KeyboardEvent → canonical chord string. */
export function chordFromEvent(event: KeyboardEvent): string {
    const parts: string[] = [];
    if (event.altKey) parts.push('alt');
    if (event.ctrlKey || event.metaKey) parts.push('ctrl');
    if (event.shiftKey) parts.push('shift');
    const key = event.key.toLowerCase();
    // Map browser key names to exocad-style names.
    const mapped =
        key === ' '
            ? 'space'
            : key === 'arrowup'
              ? 'up'
              : key === 'arrowdown'
                ? 'down'
                : key === 'arrowleft'
                  ? 'left'
                  : key === 'arrowright'
                    ? 'right'
                    : key === 'pageup'
                      ? 'pageup'
                      : key === 'pagedown'
                        ? 'pagedown'
                        : key === 'home'
                          ? 'home'
                          : key === 'escape'
                            ? 'esc'
                            : key === 'backspace'
                              ? 'backspace'
                              : key;
    parts.push(mapped);
    return parts.join('+');
}

/** Parse user-friendly chord ("Ctrl+R") into the canonical form. */
export function normalizeChord(input: string): string {
    return input
        .toLowerCase()
        .split('+')
        .map((s) => s.trim())
        .sort((a, b) => {
            // alt < ctrl < meta < shift < key
            const order: Record<string, number> = {
                alt: 0,
                ctrl: 1,
                meta: 2,
                shift: 3,
            };
            const ra = order[a] ?? 99;
            const rb = order[b] ?? 99;
            return ra - rb;
        })
        .join('+');
}
