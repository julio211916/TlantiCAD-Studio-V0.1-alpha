/**
 * CAD Wizard state machine — orchestrates the exocad-style design steps.
 *
 * Default sequence (linear, back/next navigation):
 *   margin → insertion → preop-waxup? → crown-bottoms → abutment? → freeforming → connectors → done
 *
 * `preop-waxup` and `abutment` are conditional — only present in the active
 * sequence when the case requires them (V175/V148 wiring).
 */

export type WizardStepId =
    | 'margin'
    | 'insertion'
    | 'preop-waxup'
    | 'crown-bottoms'
    | 'abutment'
    | 'freeforming'
    | 'connectors'
    | 'done';

export interface WizardStepDefinition {
    id: WizardStepId;
    label: string;
    shortLabel: string;
    description: string;
    iconName: string;
}

/** Master catalogue of all possible steps. The active sequence is a subset. */
export const ALL_WIZARD_STEPS: WizardStepDefinition[] = [
    {
        id: 'margin',
        label: 'Margin Line Detection',
        shortLabel: 'Margin',
        description: 'Detect and refine the preparation margin line.',
        iconName: 'workflow.margin-detect',
    },
    {
        id: 'insertion',
        label: 'Insertion Direction',
        shortLabel: 'Insertion',
        description: 'Set the insertion axis and visualise undercuts.',
        iconName: 'workflow.insertion-direction',
    },
    {
        id: 'preop-waxup',
        label: 'Pre-op / Waxup',
        shortLabel: 'Pre-op',
        description: 'Adapt to a pre-op scan or copy-mill from a waxup.',
        iconName: 'workflow.preop-scan',
    },
    {
        id: 'crown-bottoms',
        label: 'Crown Bottoms',
        shortLabel: 'Bottoms',
        description: 'Configure cement gap, border, and milling parameters.',
        iconName: 'workflow.cement-gap',
    },
    {
        id: 'abutment',
        label: 'Abutment Design',
        shortLabel: 'Abutment',
        description: 'Style, profile, and angulated screw channel.',
        iconName: 'abutment.style-standard',
    },
    {
        id: 'freeforming',
        label: 'Free-Forming',
        shortLabel: 'Free-Form',
        description: 'Sculpt, adapt, and add attachments.',
        iconName: 'workflow.free-form-brush',
    },
    {
        id: 'connectors',
        label: 'Connectors',
        shortLabel: 'Connectors',
        description: 'Auto-suggest and edit bridge connectors.',
        iconName: 'workflow.connector',
    },
    {
        id: 'done',
        label: 'Review & Export',
        shortLabel: 'Export',
        description: 'Review geometry and export for production.',
        iconName: 'common.save',
    },
];

/** Steps in the default sequence (no Pre-op/Waxup, no abutment). */
export const WIZARD_STEPS: WizardStepDefinition[] = ALL_WIZARD_STEPS.filter(
    (step) => step.id !== 'preop-waxup' && step.id !== 'abutment',
);

export interface WizardSequenceConditions {
    /** Active tooth has additionalScans.preOpModel || additionalScans.waxup. */
    needsPreopWaxup?: boolean;
    /** Active tooth has workTypeId === 'custom-abutment' or implantMode === 'custom-abutment'. */
    needsAbutment?: boolean;
}

/**
 * Compose the active sequence based on per-case conditions. Always returns
 * `margin` first and `done` last; intermediate steps depend on conditions.
 */
export function buildWizardSequence(
    conditions: WizardSequenceConditions = {},
): WizardStepDefinition[] {
    return ALL_WIZARD_STEPS.filter((step) => {
        if (step.id === 'preop-waxup') return Boolean(conditions.needsPreopWaxup);
        if (step.id === 'abutment') return Boolean(conditions.needsAbutment);
        return true;
    });
}

export interface WizardState {
    current: WizardStepId;
    completed: Set<WizardStepId>;
    /** Active sequence — defaults to the static one without conditional steps. */
    sequence: WizardStepDefinition[];
}

export function createInitialWizardState(
    start: WizardStepId = 'margin',
    sequence: WizardStepDefinition[] = WIZARD_STEPS,
): WizardState {
    return { current: start, completed: new Set(), sequence };
}

export function canGoNext(state: WizardState): boolean {
    const idx = state.sequence.findIndex((s) => s.id === state.current);
    return idx >= 0 && idx < state.sequence.length - 1;
}

export function canGoBack(state: WizardState): boolean {
    const idx = state.sequence.findIndex((s) => s.id === state.current);
    return idx > 0;
}

export function goNext(state: WizardState): WizardState {
    const idx = state.sequence.findIndex((s) => s.id === state.current);
    if (idx < 0 || idx >= state.sequence.length - 1) return state;
    const completed = new Set(state.completed);
    completed.add(state.current);
    return { ...state, current: state.sequence[idx + 1].id, completed };
}

export function goBack(state: WizardState): WizardState {
    const idx = state.sequence.findIndex((s) => s.id === state.current);
    if (idx <= 0) return state;
    return { ...state, current: state.sequence[idx - 1].id };
}

export function jumpTo(state: WizardState, target: WizardStepId): WizardState {
    return { ...state, current: target };
}

export function progressPercent(state: WizardState): number {
    return Math.round((state.completed.size / (state.sequence.length - 1)) * 100);
}

/**
 * Re-build the sequence when conditions change. Preserves `current` if it's
 * still in the new sequence; otherwise jumps to the closest reachable step.
 */
export function withSequence(
    state: WizardState,
    conditions: WizardSequenceConditions,
): WizardState {
    const next = buildWizardSequence(conditions);
    const stillThere = next.some((s) => s.id === state.current);
    return {
        ...state,
        sequence: next,
        current: stillThere ? state.current : next[0].id,
    };
}
