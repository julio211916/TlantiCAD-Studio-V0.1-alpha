/**
 * AbutmentNextStepDialog — V214.
 *
 * Mirrors exocad's prompt that appears when the abutment design completes:
 * the user picks between saving the abutments only (closes the wizard with
 * just the abutment STLs) or continuing to design the suprastructure
 * (Freeforming → Crown bottoms → Shrinking → Connectors).
 */

import React from 'react';

export type AbutmentNextStep = 'save-only' | 'continue-suprastructure';

export interface AbutmentNextStepDialogProps {
    open: boolean;
    onClose: () => void;
    onChoose: (step: AbutmentNextStep) => void;
    /** Number of abutments designed — surfaced in the dialog copy. */
    abutmentCount?: number;
}

export function AbutmentNextStepDialog({
    open,
    onClose,
    onChoose,
    abutmentCount,
}: AbutmentNextStepDialogProps) {
    if (!open) return null;
    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label="Abutment design — next step"
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex w-full max-w-md flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        Abutment design complete
                    </h2>
                    <p className="text-[11px] text-text-secondary">
                        {abutmentCount && abutmentCount > 0
                            ? `${abutmentCount} abutment${abutmentCount > 1 ? 's' : ''} ready.`
                            : 'Abutments ready.'}{' '}
                        Pick how to proceed.
                    </p>
                </header>
                <div className="flex flex-col gap-2 px-4 py-3">
                    <Choice
                        title="Save abutments only"
                        body="Close the wizard with the abutment STL + .constructionInfo. The crown / coping is fabricated separately."
                        onClick={() => onChoose('save-only')}
                    />
                    <Choice
                        title="Continue to suprastructure"
                        body="Run Freeforming → Crown bottoms → Shrinking → Connectors with the abutment as the new base. The merge step writes a single union STL."
                        accent
                        onClick={() => onChoose('continue-suprastructure')}
                    />
                </div>
                <footer className="border-t border-border bg-surface-sunken/40 px-4 py-3">
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        Cancel
                    </button>
                </footer>
            </div>
        </div>
    );
}

function Choice({
    title,
    body,
    onClick,
    accent,
}: {
    title: string;
    body: string;
    onClick: () => void;
    accent?: boolean;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'flex flex-col items-start gap-1 rounded-md border px-3 py-3 text-left transition',
                accent
                    ? 'border-orange-400 bg-orange-500/10 text-orange-100 hover:bg-orange-500/15'
                    : 'border-border bg-surface-sunken text-text-primary hover:border-sky-400',
            ].join(' ')}
        >
            <span className="text-sm font-semibold">{title}</span>
            <span className="text-[11px] leading-snug text-text-secondary">{body}</span>
        </button>
    );
}
