/**
 * HotkeyHelpOverlay — F1 cheat-sheet (V203).
 *
 * Lists every registered hotkey grouped by context. Mounted at App root next
 * to the command palette; toggles open with F1 (global hotkey).
 */

import React, { useEffect, useState } from 'react';

import { hotkeyRegistry } from '../application/hotkey-registry';
import type { HotkeyContext, HotkeyDefinition } from '../domain/hotkey';

export function HotkeyHelpOverlay() {
    const [open, setOpen] = useState(false);
    const [, force] = useState(0);

    useEffect(() => {
        const dispose = hotkeyRegistry.register({
            chord: 'f1',
            label: 'Toggle hotkey help',
            description: 'Show / hide the keyboard shortcuts cheat-sheet',
            context: 'global',
            run: () => setOpen((v) => !v),
            paletteAction: {
                id: 'help.hotkeys.toggle',
                label: 'Show keyboard shortcuts',
                kind: 'navigation',
                keywords: ['hotkeys', 'shortcuts', 'help'],
            },
        });
        return dispose;
    }, []);

    useEffect(() => {
        if (!open) return;
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape') setOpen(false);
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [open]);

    // Force re-render when registry changes (cheap — overlay is closed most of the time).
    useEffect(() => {
        if (!open) return;
        const id = window.setInterval(() => force((n) => n + 1), 1000);
        return () => window.clearInterval(id);
    }, [open]);

    if (!open) return null;

    const grouped = groupByContext(hotkeyRegistry.list());

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Hotkey help"
            className="fixed inset-0 z-[150] flex items-start justify-center bg-black/60 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) setOpen(false);
            }}
        >
            <div className="flex max-h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-center justify-between border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        Keyboard shortcuts
                    </h2>
                    <kbd className="rounded border border-border bg-surface-sunken px-2 py-0.5 font-mono text-[10px] text-text-secondary">
                        Esc
                    </kbd>
                </header>
                <div className="grid flex-1 grid-cols-2 gap-x-6 gap-y-4 overflow-y-auto px-4 py-3">
                    {Object.entries(grouped).map(([context, defs]) => (
                        <section key={context} className="text-[12px]">
                            <h3 className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-text-secondary">
                                {humanContext(context as HotkeyContext)}
                            </h3>
                            <ul className="flex flex-col gap-1">
                                {defs.map((d) => (
                                    <li
                                        key={`${d.context}-${d.chord}-${d.label}`}
                                        className="flex items-center justify-between gap-2 rounded px-2 py-1 hover:bg-surface-sunken"
                                    >
                                        <span className="truncate text-text-primary">{d.label}</span>
                                        <kbd className="shrink-0 rounded border border-border bg-surface-sunken px-1.5 py-0.5 font-mono text-[10px] text-text-secondary">
                                            {humanizeChord(d.chord)}
                                        </kbd>
                                    </li>
                                ))}
                            </ul>
                        </section>
                    ))}
                </div>
                <footer className="flex items-center gap-3 border-t border-border bg-surface-sunken/40 px-4 py-2 text-[10px] text-text-secondary">
                    <span>{hotkeyRegistry.list().length} shortcuts registered</span>
                    <span className="ml-auto">
                        Press <kbd className="rounded border border-border bg-surface-sunken px-1 font-mono">F1</kbd>{' '}
                        anytime to reopen
                    </span>
                </footer>
            </div>
        </div>
    );
}

function groupByContext(defs: HotkeyDefinition[]): Record<HotkeyContext, HotkeyDefinition[]> {
    const out: Record<HotkeyContext, HotkeyDefinition[]> = {
        global: [],
        wizard: [],
        visibility: [],
        'crown-bottom': [],
        'free-form': [],
        'free-form-cut': [],
        margin: [],
        'mesh-editor': [],
        'design-control': [],
    };
    for (const d of defs) out[d.context].push(d);
    return out;
}

function humanContext(c: HotkeyContext): string {
    switch (c) {
        case 'global':
            return 'Global';
        case 'wizard':
            return 'Wizard';
        case 'visibility':
            return 'Show / Hide groups';
        case 'crown-bottom':
            return 'Crown bottoms';
        case 'free-form':
            return 'Free-forming';
        case 'free-form-cut':
            return 'Free-forming · Cut intersections';
        case 'margin':
            return 'Margin detection';
        case 'mesh-editor':
            return 'Mesh editor';
        case 'design-control':
            return 'Design controls';
    }
}

function humanizeChord(chord: string): string {
    return chord
        .split('+')
        .map((p) =>
            p === 'ctrl'
                ? 'Ctrl'
                : p === 'shift'
                  ? 'Shift'
                  : p === 'alt'
                    ? 'Alt'
                    : p === 'meta'
                      ? '⌘'
                      : p === 'space'
                        ? 'Space'
                        : p.length === 1
                          ? p.toUpperCase()
                          : p.charAt(0).toUpperCase() + p.slice(1),
        )
        .join(' + ');
}
