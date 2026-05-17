/**
 * MaterialLibraryDialog — V206 (tlantishare rename).
 *
 * Modal "Material Configuration Selection" replicating the exocad dialog
 * (image #07): two tabs (Local / TlantiShare peer libraries), OK/Cancel
 * footer. Pure presentational; the parent owns state, async loading, and
 * the P2P listener that fills the peer-libraries inbox.
 */

import React, { useState } from 'react';

import type {
    MaterialLibraryEntry,
    MaterialLibraryState,
} from '../domain/material-library';
import { findLibrary } from '../domain/material-library';

export interface MaterialLibraryDialogProps {
    open: boolean;
    state: MaterialLibraryState;
    onClose: () => void;
    onSelect: (libraryId: string) => void;
    onRefreshLocal: () => void;
    onRefreshPeerLibraries: () => void;
    onForgetPeerLibrary: (libraryId: string) => void;
}

type Tab = 'local' | 'peers';

export function MaterialLibraryDialog({
    open,
    state,
    onClose,
    onSelect,
    onRefreshLocal,
    onRefreshPeerLibraries,
    onForgetPeerLibrary,
}: MaterialLibraryDialogProps) {
    const [tab, setTab] = useState<Tab>('local');
    const [pendingId, setPendingId] = useState<string | null>(state.activeLibraryId);

    if (!open) return null;

    const visible = tab === 'local' ? state.local : state.peerLibraries;
    const active = pendingId ? findLibrary(state, pendingId) : null;

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Material configuration selection"
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex max-h-[80vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-center justify-between border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        Material Configuration Selection
                    </h2>
                    <kbd
                        onClick={onClose}
                        className="cursor-pointer rounded border border-border bg-surface-sunken px-2 py-0.5 font-mono text-[10px] text-text-secondary hover:bg-surface-raised"
                    >
                        Esc
                    </kbd>
                </header>

                <nav className="flex border-b border-border bg-surface-sunken/40 text-[11px] uppercase tracking-wider">
                    <TabButton active={tab === 'local'} onClick={() => setTab('local')}>
                        Local libraries
                    </TabButton>
                    <TabButton active={tab === 'peers'} onClick={() => setTab('peers')}>
                        TlantiShare peers
                    </TabButton>
                    <button
                        type="button"
                        className="ml-auto px-3 py-2 text-text-secondary hover:text-text-primary"
                        onClick={tab === 'local' ? onRefreshLocal : onRefreshPeerLibraries}
                    >
                        ↻ Refresh
                    </button>
                </nav>

                <div className="grid flex-1 grid-cols-[2fr_1fr] gap-0 overflow-hidden">
                    <div className="overflow-y-auto border-r border-border">
                        {state.isLoading ? (
                            <div className="px-4 py-6 text-center text-xs text-text-secondary">
                                Loading…
                            </div>
                        ) : visible.length === 0 ? (
                            <div className="px-4 py-6 text-center text-xs text-text-secondary">
                                {tab === 'peers'
                                    ? 'No peer libraries received yet. When a colleague shares a library via TlantiShare on the same Wi-Fi, it will appear here.'
                                    : 'No local libraries found.'}
                            </div>
                        ) : (
                            <ul role="listbox" className="flex flex-col">
                                {visible.map((entry) => (
                                    <LibraryRow
                                        key={entry.id}
                                        entry={entry}
                                        active={pendingId === entry.id}
                                        onClick={() => setPendingId(entry.id)}
                                        onForget={
                                            tab === 'peers'
                                                ? () => onForgetPeerLibrary(entry.id)
                                                : undefined
                                        }
                                    />
                                ))}
                            </ul>
                        )}
                    </div>

                    <aside className="overflow-y-auto bg-surface-sunken/30 px-4 py-3">
                        {active ? (
                            <div className="flex flex-col gap-1.5 text-[11px]">
                                <p className="text-[10px] uppercase tracking-wider text-text-secondary">
                                    Library details
                                </p>
                                <p className="text-sm font-semibold text-text-primary">
                                    {active.label}
                                </p>
                                <p className="text-text-secondary">{active.vendor}</p>
                                <p className="font-mono text-[10px] text-text-secondary">
                                    Last updated · {active.lastUpdated}
                                </p>
                                <p className="mt-2 text-[10px] uppercase tracking-wider text-text-secondary">
                                    Materials ({active.materials.length})
                                </p>
                                <ul className="flex flex-wrap gap-1">
                                    {active.materials.map((m) => (
                                        <li
                                            key={m}
                                            className="rounded border border-border bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-primary"
                                        >
                                            {m}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        ) : (
                            <p className="text-[11px] text-text-secondary">
                                Select a library to see its materials.
                            </p>
                        )}
                    </aside>
                </div>

                {state.error ? (
                    <p className="border-t border-rose-500/40 bg-rose-500/10 px-3 py-1.5 text-[11px] text-rose-200">
                        {state.error}
                    </p>
                ) : null}

                <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-4 py-3">
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={() => {
                            if (pendingId) {
                                onSelect(pendingId);
                                onClose();
                            }
                        }}
                        disabled={!pendingId}
                        className="ml-auto rounded-md bg-sky-500 px-4 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
                    >
                        OK
                    </button>
                </footer>
            </div>
        </div>
    );
}

function TabButton({
    active,
    onClick,
    children,
}: {
    active: boolean;
    onClick: () => void;
    children: React.ReactNode;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'px-4 py-2 transition',
                active
                    ? 'border-b-2 border-sky-400 text-text-primary'
                    : 'text-text-secondary hover:text-text-primary',
            ].join(' ')}
        >
            {children}
        </button>
    );
}

function LibraryRow({
    entry,
    active,
    onClick,
    onForget,
}: {
    entry: MaterialLibraryEntry;
    active: boolean;
    onClick: () => void;
    onForget?: () => void;
}) {
    return (
        <li
            role="option"
            aria-selected={active}
            onClick={onClick}
            onMouseDown={(e) => e.preventDefault()}
            className={[
                'flex cursor-pointer items-center gap-3 border-b border-border px-3 py-2 text-sm',
                active
                    ? 'bg-sky-500/15 text-text-primary'
                    : 'text-text-primary hover:bg-surface-sunken/80',
            ].join(' ')}
        >
            <div className="min-w-0 flex-1">
                <div className="truncate font-semibold">{entry.label}</div>
                <div className="truncate text-[11px] text-text-secondary">
                    {entry.vendor} · {entry.materials.length} materials
                </div>
            </div>
            <span
                className={[
                    'shrink-0 rounded px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-wider',
                    entry.source === 'local'
                        ? 'bg-emerald-500/20 text-emerald-300'
                        : 'bg-sky-500/20 text-sky-300',
                ].join(' ')}
            >
                {entry.source}
            </span>
            {onForget ? (
                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        onForget();
                    }}
                    aria-label="Forget peer library"
                    className="shrink-0 rounded px-1.5 py-0.5 text-[10px] text-text-secondary hover:bg-rose-500/20 hover:text-rose-300"
                >
                    ✕
                </button>
            ) : null}
        </li>
    );
}
