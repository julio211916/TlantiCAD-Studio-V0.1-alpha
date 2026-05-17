/**
 * OpenLocationDialog — V264.
 *
 * Custom dialog matching the RealGUIDE "Open / Location / Cancel" flow:
 *   - Lists recent DICOM/case locations from localStorage
 *   - Open picks a recent entry; Location reveals it in Finder/Explorer
 *   - Cancel discards
 *
 * i18n keys reused from `import.action.*`.
 */

import React from 'react';

import { useT } from '../../lib/i18n';
import { forgetLocation, type RecentLocation } from '../../lib/recent-locations';

export interface OpenLocationDialogProps {
    open: boolean;
    locations: readonly RecentLocation[];
    onClose: () => void;
    onOpen: (location: RecentLocation) => void;
    onReveal: (location: RecentLocation) => void;
    /** Browse for a fresh location. */
    onBrowse: () => void;
    /** Refresh after deletion. */
    onLocationsChange?: () => void;
}

export function OpenLocationDialog({
    open,
    locations,
    onClose,
    onOpen,
    onReveal,
    onBrowse,
    onLocationsChange,
}: OpenLocationDialogProps) {
    const t = useT();
    if (!open) return null;
    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label={t('import.dicom.title')}
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex w-full max-w-lg flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        {t('import.dicom.title')}
                    </h2>
                    <p className="mt-1 text-[11px] text-text-secondary">
                        Open a recent location, reveal it in your file manager, or browse for a new folder/file.
                    </p>
                </header>

                <div className="max-h-[50vh] overflow-y-auto">
                    {locations.length === 0 ? (
                        <p className="px-4 py-6 text-center text-[11px] text-text-secondary">
                            No recent locations yet — Browse to pick the first one.
                        </p>
                    ) : (
                        <ul className="flex flex-col">
                            {locations.map((entry) => (
                                <li
                                    key={entry.path}
                                    className="flex items-center gap-3 border-b border-border px-3 py-2.5 text-sm"
                                >
                                    <div className="min-w-0 flex-1">
                                        <div className="truncate font-semibold text-text-primary">
                                            {entry.label}
                                        </div>
                                        <div className="truncate font-mono text-[10px] text-text-secondary">
                                            {entry.path}
                                            {entry.caseRef ? ` · ${entry.caseRef}` : ''}
                                        </div>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={() => onReveal(entry)}
                                        title="Reveal in file manager"
                                        className="rounded-md border border-border bg-surface-sunken px-2 py-1 text-[10px]"
                                    >
                                        Location
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => onOpen(entry)}
                                        className="rounded-md bg-sky-500 px-3 py-1 text-[11px] font-semibold text-white"
                                    >
                                        Open
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            forgetLocation(entry.path);
                                            onLocationsChange?.();
                                        }}
                                        title="Forget this entry"
                                        className="rounded-md border border-rose-500/30 bg-rose-500/5 px-1.5 py-1 text-[10px] text-rose-300"
                                    >
                                        ✕
                                    </button>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>

                <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-4 py-3">
                    <button
                        type="button"
                        onClick={onBrowse}
                        className="rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        Browse…
                    </button>
                    <button
                        type="button"
                        onClick={onClose}
                        className="ml-auto rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        {t('import.action.close')}
                    </button>
                </footer>
            </div>
        </div>
    );
}
