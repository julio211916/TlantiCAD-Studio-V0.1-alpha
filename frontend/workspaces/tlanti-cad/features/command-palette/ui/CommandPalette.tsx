/**
 * CommandPalette — Cmd/Ctrl+K overlay. Searches every registered action
 * across features (toolbar, wizard steps, tools, settings).
 */

import React, { useEffect, useMemo, useRef, useState } from 'react';

import type { CommandAction, CommandMatch } from '../domain/command-action';
import { useCommandPalette } from './useCommandPalette';

export function CommandPalette() {
    const { open, query, setQuery, results, recents, closePalette, runAction } = useCommandPalette();
    const [index, setIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement | null>(null);

    const items = useMemo<CommandMatch[]>(() => {
        if (query.trim().length === 0) {
            return recents.map((action, i) => ({ action, score: 1000 - i, matchedIndices: [] }));
        }
        return results;
    }, [query, recents, results]);

    useEffect(() => {
        setIndex(0);
    }, [query, open]);

    useEffect(() => {
        if (open) {
            // Defer focus so the overlay is mounted before focusing.
            const id = window.setTimeout(() => inputRef.current?.focus(), 30);
            return () => window.clearTimeout(id);
        }
        return undefined;
    }, [open]);

    if (!open) return null;

    const activate = async (target: CommandMatch | undefined) => {
        if (!target) return;
        await runAction(target.action);
    };

    const onKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            setIndex((value) => Math.min(items.length - 1, value + 1));
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            setIndex((value) => Math.max(0, value - 1));
        } else if (event.key === 'Enter') {
            event.preventDefault();
            void activate(items[index]);
        } else if (event.key === 'Escape') {
            event.preventDefault();
            closePalette();
        }
    };

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Command palette"
            className="fixed inset-0 z-[140] flex items-start justify-center bg-black/55 p-4 pt-24 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) closePalette();
            }}
        >
            <div className="flex w-full max-w-xl flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="1.8"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        className="text-text-secondary"
                    >
                        <circle cx="11" cy="11" r="7" />
                        <path d="M20 20l-3.5-3.5" />
                    </svg>
                    <input
                        ref={inputRef}
                        value={query}
                        onChange={(e) => setQuery(e.currentTarget.value)}
                        onKeyDown={onKeyDown}
                        placeholder="Search actions, tools, cases, settings…"
                        className="flex-1 bg-transparent text-sm text-text-primary placeholder:text-text-secondary focus:outline-none"
                    />
                    <kbd className="rounded border border-border bg-surface-sunken px-1.5 py-0.5 font-mono text-[10px] text-text-secondary">
                        Esc
                    </kbd>
                </div>

                {items.length === 0 ? (
                    <div className="px-3 py-6 text-center text-xs text-text-secondary">
                        No actions match <span className="font-mono text-text-primary">{query}</span>.
                    </div>
                ) : (
                    <ul role="listbox" className="max-h-[360px] overflow-y-auto py-1">
                        {items.map((item, i) => (
                            <PaletteItem
                                key={item.action.id}
                                match={item}
                                active={i === index}
                                onMouseEnter={() => setIndex(i)}
                                onClick={() => void activate(item)}
                                showMatch={query.trim().length > 0}
                            />
                        ))}
                    </ul>
                )}

                <footer className="flex items-center gap-3 border-t border-border bg-surface-sunken/40 px-3 py-2 text-[10px] text-text-secondary">
                    <span>
                        <kbd className="rounded border border-border bg-surface-sunken px-1 font-mono">↑ ↓</kbd>{' '}
                        navigate
                    </span>
                    <span>
                        <kbd className="rounded border border-border bg-surface-sunken px-1 font-mono">↵</kbd>{' '}
                        run
                    </span>
                    <span>
                        <kbd className="rounded border border-border bg-surface-sunken px-1 font-mono">⌘K</kbd>{' '}
                        toggle
                    </span>
                    <span className="ml-auto font-mono">{items.length} results</span>
                </footer>
            </div>
        </div>
    );
}

function PaletteItem({
    match,
    active,
    onMouseEnter,
    onClick,
    showMatch,
}: {
    match: CommandMatch;
    active: boolean;
    onMouseEnter: () => void;
    onClick: () => void;
    showMatch: boolean;
}) {
    return (
        <li
            role="option"
            aria-selected={active}
            onMouseEnter={onMouseEnter}
            onMouseDown={(e) => e.preventDefault()}
            onClick={onClick}
            className={[
                'flex cursor-pointer items-center gap-3 px-3 py-2 text-sm',
                active
                    ? 'bg-sky-500/15 text-text-primary'
                    : 'text-text-primary hover:bg-surface-sunken/80',
            ].join(' ')}
        >
            <KindBadge kind={match.action.kind} />
            <div className="min-w-0 flex-1">
                <div className="truncate font-medium">
                    {showMatch ? (
                        <HighlightedLabel label={match.action.label} indices={match.matchedIndices} />
                    ) : (
                        match.action.label
                    )}
                </div>
                {match.action.description ? (
                    <div className="truncate text-[11px] text-text-secondary">
                        {match.action.description}
                    </div>
                ) : null}
            </div>
            {match.action.hotkey ? (
                <kbd className="rounded border border-border bg-surface-sunken px-1.5 py-0.5 font-mono text-[10px] text-text-secondary">
                    {match.action.hotkey}
                </kbd>
            ) : null}
        </li>
    );
}

function HighlightedLabel({
    label,
    indices,
}: {
    label: string;
    indices: readonly number[];
}) {
    if (indices.length === 0) return <>{label}</>;
    const marks = new Set(indices);
    return (
        <>
            {Array.from(label).map((ch, i) => (
                <span key={i} className={marks.has(i) ? 'font-bold text-sky-400' : undefined}>
                    {ch}
                </span>
            ))}
        </>
    );
}

function KindBadge({ kind }: { kind: CommandAction['kind'] }) {
    const colorMap: Record<CommandAction['kind'], string> = {
        navigation: 'bg-sky-500/20 text-sky-300',
        'wizard-step': 'bg-purple-500/20 text-purple-300',
        tool: 'bg-emerald-500/20 text-emerald-300',
        toggle: 'bg-amber-500/20 text-amber-300',
        case: 'bg-rose-500/20 text-rose-300',
        asset: 'bg-cyan-500/20 text-cyan-300',
        setting: 'bg-slate-500/20 text-slate-300',
        custom: 'bg-violet-500/20 text-violet-300',
    };
    return (
        <span
            className={[
                'shrink-0 rounded px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-wider',
                colorMap[kind],
            ].join(' ')}
        >
            {kind.replace('-', ' ')}
        </span>
    );
}
