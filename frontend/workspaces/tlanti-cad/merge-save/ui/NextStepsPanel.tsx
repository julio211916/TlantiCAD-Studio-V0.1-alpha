/**
 * NextStepsPanel — post-merge navigation. Replicates exocad image #10:
 * I'm done / Design model / Proceed to production / Free-form / Expert mode.
 */

import React from 'react';

import { Button } from '@/components/ui/button';

export type NextStepChoice =
    | 'done'
    | 'design-model'
    | 'proceed-production'
    | 'free-form'
    | 'expert-mode';

export interface NextStepsPanelProps {
    complete: boolean;
    exocamLicensed?: boolean;
    saveSceneInProject: boolean;
    onSaveSceneChange: (value: boolean) => void;
    designModelMode: 'quick' | 'select';
    onDesignModelModeChange: (value: 'quick' | 'select') => void;
    onChoose: (choice: NextStepChoice) => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function NextStepsPanel({
    complete,
    exocamLicensed = false,
    saveSceneInProject,
    onSaveSceneChange,
    designModelMode,
    onDesignModelModeChange,
    onChoose,
    onBack,
    onNext,
}: NextStepsPanelProps) {
    return (
        <div className="flex h-full w-[360px] flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-xl">
            <header className="flex items-center gap-2 bg-[#3B2B6F] px-4 py-3 text-white">
                <div className="text-sm font-semibold">Merge and Save Restorations</div>
            </header>

            <nav className="flex gap-0 bg-[#4A3983] text-[11px] font-semibold uppercase tracking-wider">
                <div className="flex-1 border-b-2 border-white px-3 py-2 text-white">Next step:</div>
                <div className="flex-1 px-3 py-2 text-white/70">Saved files</div>
            </nav>

            <div className="flex-1 overflow-y-auto px-4 py-4">
                <div className="mb-3 rounded-md border border-border bg-sky-500/10 px-3 py-2 text-[11px] text-text-primary">
                    <span className="font-semibold">
                        {complete ? 'Design finished.' : 'Awaiting merge.'}
                    </span>{' '}
                    {complete ? 'Files saved to project directory.' : 'Merge to unlock next steps.'}
                </div>

                <p className="mb-2 text-[10px] font-mono uppercase tracking-wider text-text-secondary">
                    Select next step:
                </p>
                <div className="flex flex-col gap-1.5">
                    <StepRow
                        label="I'm done"
                        icon="check"
                        onClick={() => onChoose('done')}
                        highlight
                    />
                    <StepRow
                        label="Design model"
                        icon="model"
                        onClick={() => onChoose('design-model')}
                        trailing={
                            <select
                                className="rounded border border-border bg-surface-sunken px-1.5 py-0.5 text-[11px]"
                                value={designModelMode}
                                onChange={(e) =>
                                    onDesignModelModeChange(
                                        e.currentTarget.value as 'quick' | 'select',
                                    )
                                }
                                onClick={(e) => e.stopPropagation()}
                            >
                                <option value="quick">Quick</option>
                                <option value="select">Select…</option>
                            </select>
                        }
                    />
                    <StepRow
                        label="Proceed to production"
                        icon="production"
                        onClick={() => onChoose('proceed-production')}
                        disabled={!exocamLicensed}
                        disabledTitle="Requires exocam license"
                    />
                    <StepRow
                        label="Free-form restorations"
                        icon="freeform"
                        onClick={() => onChoose('free-form')}
                    />
                    <StepRow
                        label="Expert mode"
                        icon="expert"
                        onClick={() => onChoose('expert-mode')}
                    />
                </div>

                <div className="mt-4 rounded-md border border-border bg-surface-sunken px-3 py-2 text-[11px]">
                    <div className="font-semibold text-text-primary">Close CAD software</div>
                    <label className="mt-1 flex items-center gap-2 text-text-primary">
                        <input
                            type="checkbox"
                            checked={saveSceneInProject}
                            onChange={(e) => onSaveSceneChange(e.currentTarget.checked)}
                            className="accent-sky-400"
                        />
                        <span>Save scene in project directory</span>
                    </label>
                </div>
            </div>

            <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-4 py-3">
                <Button type="button" variant="ghost" size="sm" onClick={onBack}>
                    ← Back
                </Button>
                <Button
                    type="button"
                    variant="default"
                    size="sm"
                    className="ml-auto"
                    onClick={onNext}
                    disabled={!complete}
                >
                    Next →
                </Button>
            </footer>
        </div>
    );
}

function StepRow({
    label,
    icon,
    onClick,
    trailing,
    disabled,
    disabledTitle,
    highlight,
}: {
    label: string;
    icon: 'check' | 'model' | 'production' | 'freeform' | 'expert';
    onClick: () => void;
    trailing?: React.ReactNode;
    disabled?: boolean;
    disabledTitle?: string;
    highlight?: boolean;
}) {
    return (
        <button
            type="button"
            disabled={disabled}
            onClick={onClick}
            title={disabled ? disabledTitle : undefined}
            className={[
                'flex items-center gap-3 rounded-md border px-3 py-2.5 text-left text-sm transition',
                disabled
                    ? 'cursor-not-allowed border-border bg-surface-sunken opacity-50'
                    : highlight
                      ? 'border-amber-400/60 bg-amber-500/10 text-amber-600 hover:bg-amber-500/15'
                      : 'border-border bg-surface-sunken text-text-primary hover:border-sky-400',
            ].join(' ')}
        >
            <StepIcon icon={icon} />
            <span className="flex-1 font-medium">{label}</span>
            {trailing}
        </button>
    );
}

function StepIcon({ icon }: { icon: 'check' | 'model' | 'production' | 'freeform' | 'expert' }) {
    const props = {
        viewBox: '0 0 24 24',
        width: 18,
        height: 18,
        fill: 'none',
        stroke: 'currentColor',
        strokeWidth: 1.6,
        strokeLinecap: 'round' as const,
        strokeLinejoin: 'round' as const,
    };
    switch (icon) {
        case 'check':
            return (
                <svg {...props}>
                    <path d="M5 12l4 4 10-10" />
                </svg>
            );
        case 'model':
            return (
                <svg {...props}>
                    <path d="M4 10c2-3 5-4 8-4s6 1 8 4" />
                    <path d="M5 10v6c0 2 2 3 4 3h6c2 0 4-1 4-3v-6" />
                    <path d="M8 13l2 2 2-2 2 2 2-2" />
                </svg>
            );
        case 'production':
            return (
                <svg {...props}>
                    <circle cx="12" cy="12" r="8" />
                    <circle cx="12" cy="12" r="2" fill="currentColor" fillOpacity="0.25" />
                </svg>
            );
        case 'freeform':
            return (
                <svg {...props}>
                    <path d="M3 16c3-2 5-6 9-6s6 4 9 6" />
                    <path d="M7 19h10" />
                </svg>
            );
        case 'expert':
            return (
                <svg {...props}>
                    <path d="M2 10l10-5 10 5-10 5z" />
                    <path d="M6 12v4c3 2 9 2 12 0v-4" />
                </svg>
            );
    }
}
