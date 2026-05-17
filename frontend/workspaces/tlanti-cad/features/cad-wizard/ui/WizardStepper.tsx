/**
 * WizardStepper — horizontal/vertical stepper showing the current step and
 * letting the clinician jump to any completed or current one.
 */

import React from 'react';

import { AppIcon } from '../../app-icons';
import {
    progressPercent,
    type WizardState,
    type WizardStepId,
} from '../domain/wizard-steps';

interface WizardStepperProps {
    state: WizardState;
    onJumpTo: (stepId: WizardStepId) => void;
    orientation?: 'horizontal' | 'vertical';
}

export function WizardStepper({
    state,
    onJumpTo,
    orientation = 'horizontal',
}: WizardStepperProps) {
    const vertical = orientation === 'vertical';
    return (
        <nav
            aria-label="CAD wizard steps"
            className={[
                'pointer-events-auto flex gap-1 rounded-xl border border-border bg-violet-950/70 p-1.5 text-[11px] text-slate-100 shadow-lg backdrop-blur',
                vertical ? 'flex-col w-44' : 'flex-row',
            ].join(' ')}
            data-visual-qa="wizard-stepper"
        >
            {state.sequence.map((step, index) => {
                const isCurrent = state.current === step.id;
                const isCompleted = state.completed.has(step.id);
                const reachable = isCompleted || isCurrent;
                const previousStep = index > 0 ? state.sequence[index - 1] : null;
                const tooltip = reachable
                    ? `${index + 1} · ${step.label} — ${step.description}`
                    : previousStep
                      ? `${index + 1} · ${step.label} — Completa "${previousStep.label}" para continuar`
                      : `${index + 1} · ${step.label}`;
                return (
                    <button
                        key={step.id}
                        type="button"
                        onClick={() => reachable && onJumpTo(step.id)}
                        disabled={!reachable}
                        title={tooltip}
                        aria-label={tooltip}
                        aria-current={isCurrent ? 'step' : undefined}
                        aria-disabled={!reachable}
                        className={[
                            'flex items-center gap-1.5 rounded-md border px-2 py-1.5 text-left transition',
                            isCurrent
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : isCompleted
                                  ? 'border-emerald-400/50 bg-emerald-500/10 text-emerald-200'
                                  : 'border-white/15 text-slate-400',
                            reachable ? 'hover:bg-white/10' : 'cursor-not-allowed opacity-50',
                        ].join(' ')}
                    >
                        <span
                            aria-hidden
                            className={[
                                'inline-flex h-4 min-w-[1rem] items-center justify-center rounded-full px-1 text-[9px] font-semibold tabular-nums',
                                isCurrent
                                    ? 'bg-orange-500/30 text-orange-100'
                                    : isCompleted
                                      ? 'bg-emerald-500/20 text-emerald-100'
                                      : 'bg-white/5 text-slate-400',
                            ].join(' ')}
                        >
                            {index + 1}
                        </span>
                        <AppIcon name={step.iconName} size={14} aria-hidden />
                        <span className={['truncate', vertical ? 'text-[11px]' : 'text-[10px]'].join(' ')}>
                            {vertical ? step.label : step.shortLabel}
                        </span>
                    </button>
                );
            })}
            <span className="ml-auto self-center px-2 text-[10px] uppercase tracking-wider text-slate-300">
                {progressPercent(state)}%
            </span>
        </nav>
    );
}
