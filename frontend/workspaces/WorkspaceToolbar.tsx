/**
 * WorkspaceToolbar — V262.
 *
 * RealGUIDE patient toolbar parity. Top-level row of category buttons that
 * filter the assets panel and trigger the right import dialog. Pure
 * presentational; the parent owns the active category + click handlers.
 *
 * Strings i18n-keyed under `dentaldb.toolbar.*`.
 */

import React from 'react';

import { useT } from '../../lib/i18n';

export type WorkspaceToolbarCategory =
    | 'project'
    | 'dicom'
    | 'stl'
    | 'images'
    | 'documents'
    | 'notifications';

export interface WorkspaceToolbarProps {
    active: WorkspaceToolbarCategory;
    onChange: (category: WorkspaceToolbarCategory) => void;
    /** Per-category counters (assets imported, unread notifications). */
    counters?: Partial<Record<WorkspaceToolbarCategory, number>>;
    disabled?: boolean;
}

const ENTRIES: Array<{ id: WorkspaceToolbarCategory; key: string; icon: React.ReactNode }> = [
    { id: 'project', key: 'dentaldb.toolbar.project', icon: <FolderIcon /> },
    { id: 'dicom', key: 'dentaldb.toolbar.dicom', icon: <DicomIcon /> },
    { id: 'stl', key: 'dentaldb.toolbar.stl', icon: <CubeIcon /> },
    { id: 'images', key: 'dentaldb.toolbar.images', icon: <ImageIcon /> },
    { id: 'documents', key: 'dentaldb.toolbar.documents', icon: <DocIcon /> },
    { id: 'notifications', key: 'dentaldb.toolbar.notifications', icon: <BellIcon /> },
];

export function WorkspaceToolbar({ active, onChange, counters, disabled }: WorkspaceToolbarProps) {
    const t = useT();
    return (
        <nav
            role="tablist"
            aria-label="Workspace categories"
            className="flex items-center gap-1 rounded-md border border-border bg-surface-sunken/60 px-1 py-1"
            data-visual-qa="workspace-toolbar"
        >
            {ENTRIES.map((entry) => {
                const isActive = active === entry.id;
                const count = counters?.[entry.id];
                return (
                    <button
                        key={entry.id}
                        role="tab"
                        aria-selected={isActive}
                        type="button"
                        onClick={() => onChange(entry.id)}
                        disabled={disabled}
                        className={[
                            'relative flex items-center gap-1.5 rounded-sm px-2.5 py-1.5 text-[11px] font-semibold uppercase tracking-wider transition',
                            isActive
                                ? 'bg-text-display text-black'
                                : 'text-text-secondary hover:bg-white/5 hover:text-text-primary',
                            disabled ? 'opacity-50' : '',
                        ].join(' ')}
                    >
                        <span className="size-3.5" aria-hidden>
                            {entry.icon}
                        </span>
                        <span>{t(entry.key)}</span>
                        {typeof count === 'number' && count > 0 ? (
                            <span
                                className={[
                                    'ml-0.5 rounded-full px-1.5 py-0.5 font-mono text-[9px]',
                                    isActive
                                        ? 'bg-black/15 text-black'
                                        : 'bg-text-display/15 text-text-display',
                                ].join(' ')}
                            >
                                {count}
                            </span>
                        ) : null}
                    </button>
                );
            })}
        </nav>
    );
}

// Inline icons — single path each, currentColor stroke. Keeps the molecule
// dependency-free (no AppIcon registry call per item).
function FolderIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 7v11a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-7l-2-2H5a2 2 0 0 0-2 2z" />
        </svg>
    );
}
function DicomIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <path d="M9 8h2a3 3 0 0 1 0 6H9z M14 8v6 M14 11h2.5" />
        </svg>
    );
}
function CubeIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 3l8 4v10l-8 4-8-4V7z M12 12l8-4 M12 12l-8-4 M12 12v10" />
        </svg>
    );
}
function ImageIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <rect x="3" y="5" width="18" height="14" rx="2" />
            <circle cx="9" cy="11" r="1.5" />
            <path d="M21 17l-6-6-7 8" />
        </svg>
    );
}
function DocIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M7 3h8l4 4v14a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z M14 3v5h5 M9 13h6 M9 17h6" />
        </svg>
    );
}
function BellIcon() {
    return (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M6 16h12l-1.5-3V9a4.5 4.5 0 0 0-9 0v4z M9 19a3 3 0 0 0 6 0" />
        </svg>
    );
}
