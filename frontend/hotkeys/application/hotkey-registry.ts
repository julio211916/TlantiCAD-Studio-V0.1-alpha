/**
 * Global hotkey registry — V203.
 *
 * Mounts a single window-level keydown listener. Each feature contributes
 * `HotkeyDefinition`s on mount and removes them on unmount. The first
 * definition matching the current chord whose `available()` returns true is
 * fired; subsequent matches are ignored so feature panels can shadow global
 * defaults while the panel is open.
 *
 * Definitions also opt-in to the command palette via `paletteAction`.
 */

import { commandRegistry } from '../../command-palette';
import type { HotkeyDefinition } from '../domain/hotkey';
import { chordFromEvent, normalizeChord } from '../domain/hotkey';

class HotkeyRegistryImpl {
    private definitions = new Map<string, HotkeyDefinition[]>();
    private listenersAttached = false;

    register(def: HotkeyDefinition): () => void {
        const chord = normalizeChord(def.chord);
        const existing = this.definitions.get(chord) ?? [];
        existing.push(def);
        this.definitions.set(chord, existing);

        const disposePalette = def.paletteAction
            ? commandRegistry.register({
                  id: def.paletteAction.id,
                  label: def.paletteAction.label,
                  kind: def.paletteAction.kind,
                  keywords: def.paletteAction.keywords,
                  hotkey: humanizeChord(chord),
                  run: () => def.run(synthesizeEvent(chord)),
                  available: def.available,
              })
            : null;

        this.attachListenerOnce();

        return () => {
            const list = this.definitions.get(chord);
            if (list) {
                const next = list.filter((d) => d !== def);
                if (next.length === 0) this.definitions.delete(chord);
                else this.definitions.set(chord, next);
            }
            disposePalette?.();
        };
    }

    list(): HotkeyDefinition[] {
        return Array.from(this.definitions.values()).flat();
    }

    private attachListenerOnce(): void {
        if (this.listenersAttached || typeof window === 'undefined') return;
        this.listenersAttached = true;
        window.addEventListener('keydown', this.handleKeydown);
    }

    private handleKeydown = (event: KeyboardEvent): void => {
        // Don't intercept when the user is typing in an input or contenteditable.
        const target = event.target as HTMLElement | null;
        if (
            target &&
            (target.tagName === 'INPUT' ||
                target.tagName === 'TEXTAREA' ||
                target.tagName === 'SELECT' ||
                target.isContentEditable)
        ) {
            return;
        }
        const chord = chordFromEvent(event);
        const candidates = this.definitions.get(chord);
        if (!candidates || candidates.length === 0) return;
        for (const def of candidates) {
            if (def.available && !def.available()) continue;
            event.preventDefault();
            def.run(event);
            return;
        }
    };
}

function humanizeChord(canonical: string): string {
    return canonical
        .split('+')
        .map((p) =>
            p === 'ctrl'
                ? '⌃'
                : p === 'shift'
                  ? '⇧'
                  : p === 'alt'
                    ? '⌥'
                    : p === 'meta'
                      ? '⌘'
                      : p.toUpperCase(),
        )
        .join('');
}

function synthesizeEvent(chord: string): KeyboardEvent {
    const parts = chord.split('+');
    const key = parts[parts.length - 1];
    return new KeyboardEvent('keydown', {
        key,
        ctrlKey: parts.includes('ctrl'),
        shiftKey: parts.includes('shift'),
        altKey: parts.includes('alt'),
        metaKey: parts.includes('meta'),
    });
}

export const hotkeyRegistry = new HotkeyRegistryImpl();
