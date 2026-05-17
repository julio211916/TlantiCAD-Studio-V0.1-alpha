/**
 * InfluencingTeethDialog — V211.
 *
 * Modal con 32 teeth FDI (upper + lower jaw) para que el clínico marque qué
 * dientes influyen en la articulación. Mirrors el doc de exocad
 * "Choose which teeth influence articulator movement".
 */

import React, { useEffect, useState } from 'react';

import { PERMANENT_TEETH } from '../../tooth-segmentation/domain/fdi-chart';

export interface InfluencingTeethDialogProps {
    open: boolean;
    initialFdis: readonly number[];
    onClose: () => void;
    onConfirm: (fdis: number[]) => void;
}

export function InfluencingTeethDialog({
    open,
    initialFdis,
    onClose,
    onConfirm,
}: InfluencingTeethDialogProps) {
    const [selected, setSelected] = useState<Set<number>>(new Set(initialFdis));

    useEffect(() => {
        if (open) setSelected(new Set(initialFdis));
    }, [open, initialFdis]);

    if (!open) return null;

    const toggle = (fdi: number) => {
        setSelected((prev) => {
            const next = new Set(prev);
            if (next.has(fdi)) next.delete(fdi);
            else next.add(fdi);
            return next;
        });
    };

    const upper = PERMANENT_TEETH.filter((t) => t.jaw === 'maxilla').sort(
        (a, b) => a.fdi - b.fdi,
    );
    const lower = PERMANENT_TEETH.filter((t) => t.jaw === 'mandible').sort(
        (a, b) => a.fdi - b.fdi,
    );

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Choose teeth that influence articulator movement"
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex w-full max-w-lg flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-center justify-between border-b border-border px-4 py-3">
                    <div>
                        <h2 className="text-sm font-semibold text-text-primary">
                            Influencing teeth
                        </h2>
                        <p className="text-[10px] uppercase tracking-wider text-text-secondary">
                            {selected.size} / 32 selected
                        </p>
                    </div>
                    <button
                        type="button"
                        onClick={() => setSelected(new Set(PERMANENT_TEETH.map((t) => t.fdi)))}
                        className="rounded border border-border bg-surface-sunken px-2 py-1 text-[10px] text-text-secondary hover:text-text-primary"
                    >
                        Select all
                    </button>
                    <button
                        type="button"
                        onClick={() => setSelected(new Set())}
                        className="rounded border border-border bg-surface-sunken px-2 py-1 text-[10px] text-text-secondary hover:text-text-primary"
                    >
                        Clear
                    </button>
                </header>

                <div className="flex flex-col gap-3 px-4 py-3">
                    <ToothJaw label="Upper jaw" teeth={upper} selected={selected} onToggle={toggle} />
                    <ToothJaw label="Lower jaw" teeth={lower} selected={selected} onToggle={toggle} />
                </div>

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
                            onConfirm(Array.from(selected).sort((a, b) => a - b));
                            onClose();
                        }}
                        className="ml-auto rounded-md bg-sky-500 px-4 py-1.5 text-xs font-semibold text-white"
                    >
                        Apply
                    </button>
                </footer>
            </div>
        </div>
    );
}

function ToothJaw({
    label,
    teeth,
    selected,
    onToggle,
}: {
    label: string;
    teeth: readonly { fdi: number }[];
    selected: Set<number>;
    onToggle: (fdi: number) => void;
}) {
    return (
        <div>
            <p className="mb-1 text-[10px] uppercase tracking-wider text-text-secondary">
                {label}
            </p>
            <div className="grid grid-cols-8 gap-1">
                {teeth.map((t) => {
                    const active = selected.has(t.fdi);
                    return (
                        <button
                            key={t.fdi}
                            type="button"
                            onClick={() => onToggle(t.fdi)}
                            className={[
                                'rounded border px-1 py-1.5 text-center text-[10px] font-semibold transition',
                                active
                                    ? 'border-sky-400 bg-sky-500/20 text-sky-200'
                                    : 'border-border bg-surface-sunken text-text-secondary hover:border-sky-400 hover:text-text-primary',
                            ].join(' ')}
                        >
                            {t.fdi}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
