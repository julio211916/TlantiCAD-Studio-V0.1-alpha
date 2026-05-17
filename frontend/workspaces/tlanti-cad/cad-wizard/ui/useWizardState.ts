/**
 * useWizardState — minimal React reducer wrapping the WizardState domain.
 */

import { useCallback, useEffect, useState } from 'react';

import {
    canGoBack,
    canGoNext,
    createInitialWizardState,
    goBack,
    goNext,
    jumpTo,
    withSequence,
    type WizardSequenceConditions,
    type WizardState,
    type WizardStepId,
} from '../domain/wizard-steps';

export interface UseWizardStateResult {
    state: WizardState;
    canBack: boolean;
    canNext: boolean;
    back: () => void;
    next: () => void;
    jumpTo: (step: WizardStepId) => void;
    markComplete: (step: WizardStepId) => void;
    reset: () => void;
}

export function useWizardState(
    start?: WizardStepId,
    conditions: WizardSequenceConditions = {},
): UseWizardStateResult {
    const [state, setState] = useState<WizardState>(() =>
        withSequence(createInitialWizardState(start), conditions),
    );

    // Re-build the sequence when conditions change. Avoid a needless update if
    // the active step doesn't actually need to move.
    useEffect(() => {
        setState((prev) => {
            const updated = withSequence(prev, conditions);
            const sameSeq =
                updated.sequence.length === prev.sequence.length &&
                updated.sequence.every((s, i) => s.id === prev.sequence[i]?.id);
            if (sameSeq && updated.current === prev.current) return prev;
            return updated;
        });
    }, [conditions.needsPreopWaxup, conditions.needsAbutment]);

    const back = useCallback(() => setState((s) => goBack(s)), []);
    const next = useCallback(() => setState((s) => goNext(s)), []);
    const jump = useCallback(
        (step: WizardStepId) => setState((s) => jumpTo(s, step)),
        [],
    );
    const markComplete = useCallback(
        (step: WizardStepId) =>
            setState((s) => {
                const completed = new Set(s.completed);
                completed.add(step);
                return { ...s, completed };
            }),
        [],
    );
    const reset = useCallback(
        () => setState(() => withSequence(createInitialWizardState(start), conditions)),
        [start, conditions.needsPreopWaxup, conditions.needsAbutment],
    );

    return {
        state,
        canBack: canGoBack(state),
        canNext: canGoNext(state),
        back,
        next,
        jumpTo: jump,
        markComplete,
        reset,
    };
}
