/**
 * ArticulatorLibraryPicker — V208 + V209.
 *
 * Modal vendor list (grouped by manufacturer) with adjustable badge,
 * intercondylar distance, and a "Select" action that fetches the preset and
 * notifies the parent. Pure presentational; the parent owns state and fetches.
 */

import React, { useMemo } from 'react';

import type {
    ArticulatorVendorEntry,
    ArticulatorLibraryState,
} from '../domain/articulator-vendor';
import { groupVendors } from '../domain/articulator-vendor';

export interface ArticulatorLibraryPickerProps {
    open: boolean;
    state: ArticulatorLibraryState;
    onClose: () => void;
    onSelect: (vendorId: string) => void;
}

export function ArticulatorLibraryPicker({
    open,
    state,
    onClose,
    onSelect,
}: ArticulatorLibraryPickerProps) {
    const groups = useMemo(() => groupVendors(state.vendors), [state.vendors]);

    if (!open) return null;

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Articulator library picker"
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex max-h-[80vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-center justify-between border-b border-border px-4 py-3">
                    <div>
                        <h2 className="text-sm font-semibold text-text-primary">
                            Articulator library
                        </h2>
                        <p className="text-[10px] uppercase tracking-wider text-text-secondary">
                            {state.vendors.length} vendors · backend: {state.backend ?? '—'}
                        </p>
                    </div>
                    <kbd
                        onClick={onClose}
                        className="cursor-pointer rounded border border-border bg-surface-sunken px-2 py-0.5 font-mono text-[10px] text-text-secondary hover:bg-surface-raised"
                    >
                        Esc
                    </kbd>
                </header>

                <div className="flex-1 overflow-y-auto px-3 py-2">
                    {state.isLoading ? (
                        <div className="px-4 py-6 text-center text-xs text-text-secondary">
                            Loading library…
                        </div>
                    ) : state.vendors.length === 0 ? (
                        <div className="px-4 py-6 text-center text-xs text-text-secondary">
                            No vendors found in <code>public/library/articulator/</code>.
                        </div>
                    ) : (
                        Object.entries(groups).map(([vendor, list]) => (
                            <section key={vendor} className="mb-3">
                                <h3 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-text-secondary">
                                    {vendor}
                                </h3>
                                <ul role="listbox" className="flex flex-col gap-1">
                                    {list.map((entry) => (
                                        <VendorRow
                                            key={entry.id}
                                            entry={entry}
                                            active={state.activeVendorId === entry.id}
                                            onSelect={() => onSelect(entry.id)}
                                        />
                                    ))}
                                </ul>
                            </section>
                        ))
                    )}
                </div>

                {state.error ? (
                    <p className="border-t border-rose-500/40 bg-rose-500/10 px-3 py-1.5 text-[11px] text-rose-200">
                        {state.error}
                    </p>
                ) : null}

                <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-4 py-3 text-[10px] text-text-secondary">
                    <span>
                        Adjustable vendors expose condyle inclination + Bennett sliders that
                        override the default Bonwill triangle.
                    </span>
                    <button
                        type="button"
                        onClick={onClose}
                        className="ml-auto rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        Cancel
                    </button>
                </footer>
            </div>
        </div>
    );
}

function VendorRow({
    entry,
    active,
    onSelect,
}: {
    entry: ArticulatorVendorEntry;
    active: boolean;
    onSelect: () => void;
}) {
    return (
        <li
            role="option"
            aria-selected={active}
            onClick={onSelect}
            className={[
                'flex cursor-pointer items-center gap-3 rounded border px-3 py-2 text-sm transition',
                active
                    ? 'border-orange-400 bg-orange-500/15 text-orange-200'
                    : 'border-border text-text-primary hover:border-sky-400 hover:bg-surface-sunken/80',
            ].join(' ')}
        >
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                    <span className="truncate font-semibold">{entry.label}</span>
                    {entry.adjustable ? (
                        <span className="rounded bg-emerald-500/20 px-1.5 py-0.5 font-mono text-[9px] uppercase text-emerald-300">
                            adjustable
                        </span>
                    ) : null}
                </div>
                <div className="text-[10px] text-text-secondary">
                    {entry.intercondylarDistanceMm
                        ? `Intercondylar ${entry.intercondylarDistanceMm.toFixed(2)} mm`
                        : '—'}
                    {entry.anteriorPosteriorDistanceMm
                        ? ` · A-P ${entry.anteriorPosteriorDistanceMm.toFixed(1)} mm`
                        : ''}
                </div>
            </div>
            <button
                type="button"
                onClick={(e) => {
                    e.stopPropagation();
                    onSelect();
                }}
                className="rounded-md bg-sky-500 px-3 py-1 text-[11px] font-semibold text-white"
            >
                Select
            </button>
        </li>
    );
}
