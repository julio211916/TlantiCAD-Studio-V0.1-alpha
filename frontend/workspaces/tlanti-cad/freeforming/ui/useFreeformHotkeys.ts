/**
 * useFreeformHotkeys — V228 + V229.
 *
 * Registers exocad-faithful shortcuts for the freeforming step:
 *   1 → add-remove        2 → smooth-flatten   3 → adapt
 *   4 → cusps             5 → tooth-parts      6 → entire-tooth   7 → ridge
 *   Ctrl+T → hold cusp tips    Ctrl+Q → hold equator    Ctrl+M → occlusal-only
 *   Ctrl+X → cut all       Ctrl+G → cut basal
 *   Ctrl+N → cut adjacents Ctrl+A → cut antagonist
 *
 * Active only while the parent `enabled` flag is true (typically the
 * freeforming wizard step is current).
 */

import { useEffect } from 'react';

import { hotkeyRegistry } from '../../hotkeys';
import type {
    AnatomicPreset,
    FreeformMode,
    FreeformState,
    MovementRestriction,
} from '../domain/freeform-brush';

export type CutKind = 'all' | 'basal' | 'adjacents' | 'antagonist';

export interface UseFreeformHotkeysProps {
    enabled: boolean;
    state: FreeformState;
    onChange: (next: FreeformState) => void;
    onCut: (kind: CutKind) => void;
}

export function useFreeformHotkeys({
    enabled,
    state,
    onChange,
    onCut,
}: UseFreeformHotkeysProps): void {
    useEffect(() => {
        if (!enabled) return;
        const setMode = (mode: FreeformMode) =>
            onChange({ ...state, brush: { ...state.brush, mode } });
        const setPreset = (preset: AnatomicPreset) =>
            onChange({ ...state, anatomic: { ...state.anatomic, preset } });
        const toggleRestriction = (r: MovementRestriction) => {
            const next = new Set(state.anatomic.restrictions);
            if (next.has(r)) next.delete(r);
            else next.add(r);
            onChange({ ...state, anatomic: { ...state.anatomic, restrictions: next } });
        };

        const disposers = [
            // V228 — mode shortcuts (1–3)
            hotkeyRegistry.register({
                chord: '1',
                label: 'Add / Remove',
                context: 'free-form',
                run: () => setMode('add-remove'),
            }),
            hotkeyRegistry.register({
                chord: '2',
                label: 'Smooth / Flatten',
                context: 'free-form',
                run: () => setMode('smooth-flatten'),
            }),
            hotkeyRegistry.register({
                chord: '3',
                label: 'Adapt',
                context: 'free-form',
                run: () => setMode('adapt'),
            }),
            // V228 — anatomic preset shortcuts (4–7)
            hotkeyRegistry.register({
                chord: '4',
                label: 'Move cusps',
                context: 'free-form',
                run: () => setPreset('cusps'),
            }),
            hotkeyRegistry.register({
                chord: '5',
                label: 'Move tooth parts',
                context: 'free-form',
                run: () => setPreset('tooth-parts'),
            }),
            hotkeyRegistry.register({
                chord: '6',
                label: 'Move entire tooth',
                context: 'free-form',
                run: () => setPreset('entire-tooth'),
            }),
            hotkeyRegistry.register({
                chord: '7',
                label: 'Move ridge',
                context: 'free-form',
                run: () => setPreset('ridge'),
            }),
            // V228 — restrictions
            hotkeyRegistry.register({
                chord: 'ctrl+t',
                label: 'Toggle hold cusp tips',
                context: 'free-form',
                run: () => toggleRestriction('lock-cusp-tips'),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+q',
                label: 'Toggle hold equator',
                context: 'free-form',
                run: () => toggleRestriction('lock-equator'),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+m',
                label: 'Toggle occlusal movement only',
                context: 'free-form',
                run: () => toggleRestriction('occlusal-only'),
            }),
            // V229 — cut intersections
            hotkeyRegistry.register({
                chord: 'ctrl+x',
                label: 'Cut all intersections',
                context: 'free-form-cut',
                run: () => onCut('all'),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+g',
                label: 'Cut basal only',
                context: 'free-form-cut',
                run: () => onCut('basal'),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+n',
                label: 'Cut adjacents only',
                context: 'free-form-cut',
                run: () => onCut('adjacents'),
            }),
            hotkeyRegistry.register({
                chord: 'ctrl+a',
                label: 'Cut antagonist only',
                context: 'free-form-cut',
                run: () => onCut('antagonist'),
            }),
        ];
        return () => {
            for (const d of disposers) d();
        };
    }, [enabled, state, onChange, onCut]);
}
