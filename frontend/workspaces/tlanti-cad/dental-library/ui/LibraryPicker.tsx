/**
 * LibraryPicker — pure presentational grid that lists groups within a given
 * category and fires `onSelect` with the chosen group name. The consumer owns
 * the catalog port; this component never imports adapters directly.
 */

import React, { useEffect, useMemo, useState } from 'react';
import { Loader2, FolderOpen } from 'lucide-react';

import type { LibraryCatalogPort } from '../application/library-catalog-port';
import type { LibraryGroup } from '../domain/library-item';

interface LibraryPickerProps {
    port: LibraryCatalogPort;
    category: string;
    selectedGroup?: string | null;
    onSelect: (group: LibraryGroup) => void;
    emptyLabel?: string;
}

export function LibraryPicker({
    port,
    category,
    selectedGroup,
    onSelect,
    emptyLabel = 'No assets found for this category.',
}: LibraryPickerProps) {
    const [groups, setGroups] = useState<LibraryGroup[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;
        setGroups(null);
        setError(null);
        port.listGroups(category)
            .then((g) => {
                if (!cancelled) setGroups(g);
            })
            .catch((err) => {
                if (!cancelled) setError(err instanceof Error ? err.message : String(err));
            });
        return () => {
            cancelled = true;
        };
    }, [port, category]);

    const sorted = useMemo(
        () =>
            groups
                ? [...groups].sort((a, b) => a.name.localeCompare(b.name))
                : null,
        [groups],
    );

    if (error) {
        return (
            <div className="rounded-md border border-red-500/40 bg-red-500/10 p-3 text-xs text-red-100">
                {error}
            </div>
        );
    }

    if (!sorted) {
        return (
            <div className="flex items-center gap-2 text-xs text-text-secondary">
                <Loader2 className="size-4 animate-spin" aria-hidden />
                Loading library…
            </div>
        );
    }

    if (sorted.length === 0) {
        return (
            <div className="flex items-center gap-2 text-xs text-text-secondary">
                <FolderOpen className="size-4" aria-hidden />
                {emptyLabel}
            </div>
        );
    }

    return (
        <ul className="grid grid-cols-2 gap-2 md:grid-cols-3">
            {sorted.map((group) => {
                const active = group.name === selectedGroup;
                return (
                    <li key={group.name}>
                        <button
                            type="button"
                            onClick={() => onSelect(group)}
                            className={[
                                'group flex w-full flex-col gap-2 rounded-lg border p-2 text-left transition',
                                active
                                    ? 'border-sky-400 bg-sky-500/10'
                                    : 'border-border bg-surface-raised hover:border-border-strong hover:bg-surface-raised/80',
                            ].join(' ')}
                        >
                            <div className="aspect-square overflow-hidden rounded-md bg-surface-sunken">
                                {group.previewPath ? (
                                    <img
                                        src={group.previewPath}
                                        alt={group.name}
                                        loading="lazy"
                                        className="h-full w-full object-contain"
                                    />
                                ) : (
                                    <div className="flex h-full items-center justify-center text-[0.6875rem] text-text-secondary/60">
                                        No preview
                                    </div>
                                )}
                            </div>
                            <span className="truncate text-xs font-medium text-text-primary">
                                {group.name}
                            </span>
                            <span className="text-[0.6875rem] text-text-secondary">
                                {group.items.length} files
                            </span>
                        </button>
                    </li>
                );
            })}
        </ul>
    );
}
