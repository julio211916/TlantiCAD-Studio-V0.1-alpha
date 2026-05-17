/**
 * useHotkey — convenience React hook for feature panels.
 *
 * Mounts a single hotkey definition and disposes on unmount.
 */

import { useEffect } from 'react';

import { hotkeyRegistry } from '../application/hotkey-registry';
import type { HotkeyDefinition } from '../domain/hotkey';

export function useHotkey(def: HotkeyDefinition | HotkeyDefinition[]): void {
    useEffect(() => {
        const defs = Array.isArray(def) ? def : [def];
        const disposers = defs.map((d) => hotkeyRegistry.register(d));
        return () => {
            for (const dispose of disposers) dispose();
        };
    }, [def]);
}
