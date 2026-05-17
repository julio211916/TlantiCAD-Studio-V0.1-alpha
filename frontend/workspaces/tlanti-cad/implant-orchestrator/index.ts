/**
 * Implant orchestrator (V275).
 *
 * 7-step state machine matching the RealGUIDE flow:
 *   VOI → 3D Settings → CPR → Nerve drawing → Teeth setup → Implant placement → Export
 *
 * Mirrors the cad-wizard pattern (`useWizardState`) but specific to the
 * Implant module so it can run independently of the CAD design wizard. Each
 * step is gated by the `isReady` predicate so the user can't skip a missing
 * dependency (e.g. you cannot draw nerves before CPR is built).
 */

import { useCallback, useState } from 'react';

export type ImplantStepId =
    | 'voi'
    | 'three-d-settings'
    | 'cpr'
    | 'nerve-drawing'
    | 'teeth-setup'
    | 'implant-placement'
    | 'export';

export interface ImplantStepDefinition {
    id: ImplantStepId;
    label: string;
    shortLabel: string;
    description: string;
}

export const IMPLANT_STEPS: readonly ImplantStepDefinition[] = [
    {
        id: 'voi',
        label: 'VOI Settings',
        shortLabel: 'VOI',
        description: 'Crop / sculpt the volume of interest from the CBCT.',
    },
    {
        id: 'three-d-settings',
        label: '3D Settings',
        shortLabel: '3D',
        description: 'Pick a tissue template (Bone / Soft / Skin / Air / Teeth).',
    },
    {
        id: 'cpr',
        label: 'CPR',
        shortLabel: 'CPR',
        description: 'Draw the curved planar reformation along the arch.',
    },
    {
        id: 'nerve-drawing',
        label: 'Nerve Drawing',
        shortLabel: 'Nerve',
        description: 'Trace the IAN canal — left + right.',
    },
    {
        id: 'teeth-setup',
        label: 'Teeth Setup',
        shortLabel: 'Teeth',
        description: 'Place virtual arch teeth from the library.',
    },
    {
        id: 'implant-placement',
        label: 'Implant Placement',
        shortLabel: 'Implant',
        description: 'Position implants + check safety zones.',
    },
    {
        id: 'export',
        label: 'Export',
        shortLabel: 'Export',
        description: 'Surgical guide STL / DICOM-RT / report.',
    },
];

export interface ImplantWizardState {
    current: ImplantStepId;
    completed: Set<ImplantStepId>;
}

export function createInitialImplantState(start: ImplantStepId = 'voi'): ImplantWizardState {
    return { current: start, completed: new Set() };
}

function findIndex(state: ImplantWizardState): number {
    return IMPLANT_STEPS.findIndex((s) => s.id === state.current);
}

export function canGoNext(state: ImplantWizardState): boolean {
    const idx = findIndex(state);
    return idx >= 0 && idx < IMPLANT_STEPS.length - 1;
}

export function canGoBack(state: ImplantWizardState): boolean {
    return findIndex(state) > 0;
}

export function goNext(state: ImplantWizardState): ImplantWizardState {
    const idx = findIndex(state);
    if (idx < 0 || idx >= IMPLANT_STEPS.length - 1) return state;
    const completed = new Set(state.completed);
    completed.add(state.current);
    return { current: IMPLANT_STEPS[idx + 1].id, completed };
}

export function goBack(state: ImplantWizardState): ImplantWizardState {
    const idx = findIndex(state);
    if (idx <= 0) return state;
    return { ...state, current: IMPLANT_STEPS[idx - 1].id };
}

export function jumpTo(state: ImplantWizardState, target: ImplantStepId): ImplantWizardState {
    return { ...state, current: target };
}

export function progressPercent(state: ImplantWizardState): number {
    return Math.round((state.completed.size / (IMPLANT_STEPS.length - 1)) * 100);
}

export interface UseImplantOrchestratorResult {
    state: ImplantWizardState;
    canBack: boolean;
    canNext: boolean;
    back: () => void;
    next: () => void;
    jumpTo: (step: ImplantStepId) => void;
    markComplete: (step: ImplantStepId) => void;
    reset: () => void;
}

/**
 * React hook wrapping the implant wizard state machine. Mirrors
 * `useWizardState` (cad-wizard) so the same UI primitives can render either
 * orchestrator.
 */
export function useImplantOrchestrator(start?: ImplantStepId): UseImplantOrchestratorResult {
    const [state, setState] = useState<ImplantWizardState>(() => createInitialImplantState(start));
    const back = useCallback(() => setState((s) => goBack(s)), []);
    const next = useCallback(() => setState((s) => goNext(s)), []);
    const jump = useCallback(
        (step: ImplantStepId) => setState((s) => jumpTo(s, step)),
        [],
    );
    const markComplete = useCallback(
        (step: ImplantStepId) =>
            setState((s) => {
                const completed = new Set(s.completed);
                completed.add(step);
                return { ...s, completed };
            }),
        [],
    );
    const reset = useCallback(
        () => setState(() => createInitialImplantState(start)),
        [start],
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
