/**
 * CadWizardSlot — V113-V117 + V148 + V175 wiring.
 *
 * Renders the conditional CAD wizard sequence inside CadInterface as a single
 * floating right-rail panel. State per step lives in feature hooks; the
 * sequence (which steps appear) is conditional on the active tooth's flags
 * (`needsPreopWaxup`, `needsAbutment`).
 */

import React, { useEffect, useMemo, useState } from 'react';

import {
    AbutmentDesignPanel,
    AbutmentNextStepDialog,
    createTauriAbutmentAdapter,
    defaultAbutmentDesign,
} from './features/abutment-design';
import type {
    AbutmentDesign,
    AbutmentGenerateMeshResponse,
    AbutmentNextStep,
    AbutmentProfilePreset,
    AbutmentTab,
} from './features/abutment-design';
import {
    CrownBottomsPanel,
    defaultCrownBottomParams,
} from './features/crown-bottoms';
import type { CrownBottomParams } from './features/crown-bottoms';
import {
    FreeformingPanel,
    defaultFreeformState,
    useFreeformHotkeys,
} from './features/freeforming';
import type { CutKind, FreeformState } from './features/freeforming';
import { InsertionDirectionPanel } from './features/insertion-direction';
import type { InsertionAxis } from './features/insertion-direction';
import { createBackendInsertionAdapter } from './features/insertion-direction';
import {
    MergeSavePanel,
    NextStepsPanel,
    createBackendMergeAdapter,
    useMergeJob,
} from './features/merge-save';
import type {
    CsgBridge,
    MergeToothPayload,
    NextStepChoice,
} from './features/merge-save';
import { invokeMeshOpOrNull } from '@/lib/csg-bridge';
import { createBackendToothStlAdapter } from './features/tooth-stl-persistence';
import { logger } from '@/lib/logger';
import {
    MarginDetectPanel,
    createBackendMarginAdapter,
    useMarginDetection,
} from './features/margin-line';
import {
    PreopWaxupPanel,
    createTauriPreopWaxupAdapter,
    initialPreopWaxupState,
} from './features/preop-waxup';
import type { CopyStrategy, PreopWaxupState } from './features/preop-waxup';
import {
    WizardStepper,
    useWizardState,
} from './features/cad-wizard';
import type { DentalMaterialType } from '@/lib/dental-workflow';
import { commandRegistry } from './features/command-palette';

export interface CadWizardSlotProps {
    /** Selected tooth (FDI). Drives margin / insertion detection. */
    toothFdi?: number;
    /** Active material — re-seeds Crown Bottoms defaults. */
    material?: DentalMaterialType | null;
    /** Mesh path for the current preparation. */
    meshPath?: string | null;
    /** Pre-op scan path (V175). */
    preopPath?: string | null;
    /** Waxup scan path (V175). */
    waxupPath?: string | null;
    /** Project folder where merge writes `_cad.stl` + `.constructionInfo`. */
    caseFolderPath?: string | null;
    caseId?: string | null;
    /** Rich tooth payload that the merge backend writes to constructionInfo. */
    teethPayload?: MergeToothPayload[];
    /** Show the Pre-op/Waxup wizard step (V175). */
    needsPreopWaxup?: boolean;
    /** Show the Abutment design wizard step (V148). */
    needsAbutment?: boolean;
    /** Hide/show. */
    open: boolean;
    onClose: () => void;
}

const mergePort = createBackendMergeAdapter();
const preopWaxupPort = createTauriPreopWaxupAdapter();
const toothStlPort = createBackendToothStlAdapter();
const abutmentPort = createTauriAbutmentAdapter();

/**
 * V202 / V217 — persist a placeholder STL whenever a wizard step completes.
 * `prefix` defaults to 'tooth' (V202); abutment-completion writes 'abutment'
 * (V217) so the merge bridge can union the abutment with the suprastructure.
 */
async function persistToothPlaceholder(
    caseFolderPath: string | null,
    toothFdi: number | undefined,
    prefix: 'tooth' | 'abutment' | 'screwchannel' = 'tooth',
): Promise<void> {
    if (!caseFolderPath || !Number.isFinite(toothFdi)) return;
    try {
        await toothStlPort.write({
            caseFolderPath,
            toothFdi: toothFdi as number,
            prefix,
        });
    } catch (err) {
        logger.warn(`persistToothPlaceholder(${prefix}) failed`, err);
    }
}

export function CadWizardSlot(props: CadWizardSlotProps) {
    const conditions = useMemo(
        () => ({
            needsPreopWaxup: Boolean(props.needsPreopWaxup),
            needsAbutment: Boolean(props.needsAbutment),
        }),
        [props.needsPreopWaxup, props.needsAbutment],
    );
    const wizard = useWizardState(undefined, conditions);
    const margin = useMarginDetection(() => createBackendMarginAdapter());
    const [marginTool, setMarginTool] = useState<'detect' | 'correct-draw' | 'repair-draw'>(
        'detect',
    );
    const [marginMode, setMarginMode] = useState<'subgingival' | 'supragingival'>('supragingival');

    const [insertionAxes, setInsertionAxes] = useState<Record<number, InsertionAxis | undefined>>({});
    const [uniqueForBridge, setUniqueForBridge] = useState<boolean>(true);
    const [insertionBusy, setInsertionBusy] = useState<boolean>(false);
    const [insertionError, setInsertionError] = useState<string | null>(null);
    const insertionPort = useMemo(() => createBackendInsertionAdapter(), []);

    const [crownParams, setCrownParams] = useState<CrownBottomParams>(() =>
        defaultCrownBottomParams(props.material ?? null),
    );

    const [freeformState, setFreeformState] = useState<FreeformState>(() => defaultFreeformState());

    // V228 + V229 — Freeforming hotkeys active only when the step is current.
    useFreeformHotkeys({
        enabled: wizard.state.current === 'freeforming',
        state: freeformState,
        onChange: setFreeformState,
        onCut: (_kind: CutKind) => {
            // Real cut runs in the backend (cad_freeform.* — future sprint).
            // Today we just record the intent on the freeform state for the
            // next render to surface a "cut requested" badge.
        },
    });

    // V175 — Pre-op + Waxup local state.
    const [preopWaxupState, setPreopWaxupState] = useState<PreopWaxupState>(() =>
        initialPreopWaxupState(),
    );

    // V148 — Abutment design local state.
    const [abutment, setAbutment] = useState<AbutmentDesign>(() => defaultAbutmentDesign());
    const [abutmentTab, setAbutmentTab] = useState<AbutmentTab>('top');
    const [abutmentNextStepOpen, setAbutmentNextStepOpen] = useState(false); // V214/V215
    const [abutmentOutput, setAbutmentOutput] = useState<AbutmentGenerateMeshResponse | null>(null);
    const [abutmentBusy, setAbutmentBusy] = useState(false);
    const [abutmentError, setAbutmentError] = useState<string | null>(null);

    // V201 + V217 — opportunistic CSG bridge. Resolves per-tooth STL inputs
    // (and abutment STLs when V217 has written them) from the case folder, then
    // runs the Tauri kernel after Python finalize. Falls back silently when the
    // bridge is unavailable (browser preview / inputs missing).
    const csgBridge: CsgBridge = useMemo(
        () => ({
            resolve: () => {
                if (!props.caseFolderPath) return null;
                const fdis = (props.teethPayload ?? []).map((t) => t.tooth);
                if (fdis.length === 0) return null;
                // Always include the suprastructure tooth STLs.
                const toothInputs = fdis.map(
                    (fdi) => `${props.caseFolderPath}/tooth-${fdi}.stl`,
                );
                // Add abutment STLs for cases that designed custom abutments.
                const abutmentInputs = abutmentOutput?.outputPath
                    ? [abutmentOutput.outputPath]
                    : (props.teethPayload ?? [])
                          .filter(
                              (t) =>
                                  t.workTypeId === 'custom-abutment' ||
                                  (props.needsAbutment && t.tooth),
                          )
                          .map(
                              (t) =>
                                  `${props.caseFolderPath}/work/implants/abutments/abutment-${t.tooth}.stl`,
                          );
                const inputs = [...abutmentInputs, ...toothInputs];
                if (inputs.length < 2) return null;
                const output = `${props.caseFolderPath}/${
                    fdis.length === 1
                        ? `tooth-${fdis[0]}`
                        : `bridge-${fdis[0]}-to-${fdis[fdis.length - 1]}`
                }_cad.stl`;
                return { inputs, output };
            },
            invoke: invokeMeshOpOrNull,
        }),
        [props.caseFolderPath, props.teethPayload, props.needsAbutment, abutmentOutput],
    );
    const merge = useMergeJob(mergePort, csgBridge);
    const [mergeTab, setMergeTab] = useState<'next-step' | 'saved-files' | 'screw-holes'>(
        'saved-files',
    );
    const [nextStepsSaveScene, setNextStepsSaveScene] = useState<boolean>(true);
    const [designModelMode, setDesignModelMode] = useState<'quick' | 'select'>('quick');

    // Re-seed Crown Bottoms defaults when material changes.
    useEffect(() => {
        setCrownParams(defaultCrownBottomParams(props.material ?? null));
    }, [props.material]);

    // V202 + V217 + V222 — persist placeholder STLs as the wizard advances.
    // Re-runs are harmless (backend overwrites in place).
    const completedSize = wizard.state.completed.size;
    const abutmentCompleted = wizard.state.completed.has('abutment');
    const connectorsCompleted = wizard.state.completed.has('connectors');
    const hasAnyScrewHoleCut = (props.teethPayload ?? []).some(
        (t) => t.workTypeId?.includes('abutment') || t.workTypeId?.includes('screw'),
    );
    useEffect(() => {
        if (completedSize === 0) return;
        void persistToothPlaceholder(props.caseFolderPath ?? null, props.toothFdi, 'tooth');
        if (abutmentCompleted && !abutmentOutput) return;
        if (connectorsCompleted && hasAnyScrewHoleCut) {
            void persistToothPlaceholder(
                props.caseFolderPath ?? null,
                props.toothFdi,
                'screwchannel',
            );
        }
    }, [
        completedSize,
        abutmentCompleted,
        connectorsCompleted,
        hasAnyScrewHoleCut,
        props.caseFolderPath,
        props.toothFdi,
        abutmentOutput,
    ]);

    // Register wizard navigation actions in the command palette while open.
    // Uses the active sequence so conditional steps (preop-waxup, abutment)
    // are only registered when they're actually present.
    useEffect(() => {
        if (!props.open) return;
        const dispose = commandRegistry.registerAll(
            wizard.state.sequence.map((step) => ({
                id: `wizard.jump.${step.id}`,
                label: `Wizard: ${step.label}`,
                kind: 'wizard-step' as const,
                keywords: ['wizard', 'step', step.id],
                run: () => wizard.jumpTo(step.id),
                available: () =>
                    wizard.state.completed.has(step.id) || wizard.state.current === step.id,
            })),
        );
        return dispose;
    }, [wizard, props.open]);

    if (!props.open) return null;

    const teeth: MergeToothPayload[] =
        props.teethPayload && props.teethPayload.length > 0
            ? props.teethPayload
            : props.toothFdi
              ? [{ tooth: props.toothFdi, material: props.material ?? undefined }]
              : [];

    const handleNextStep = (choice: NextStepChoice) => {
        if (choice === 'done') props.onClose();
    };

    const requestInsertionDetect = async () => {
        const fdi = props.toothFdi ?? 16;
        if (!props.meshPath) {
            setInsertionError('No mesh available for the selected tooth');
            return;
        }
        setInsertionBusy(true);
        setInsertionError(null);
        try {
            const axis = await insertionPort.detect({ meshPath: props.meshPath, toothFdi: fdi });
            setInsertionAxes((prev) => ({ ...prev, [fdi]: axis }));
        } catch (err) {
            setInsertionError(err instanceof Error ? err.message : String(err));
        } finally {
            setInsertionBusy(false);
        }
    };

    const generateAbutmentMesh = async (): Promise<AbutmentGenerateMeshResponse | null> => {
        const fdi = props.toothFdi ?? 16;
        if (!props.caseFolderPath) {
            setAbutmentError('No case folder available for abutment STL output');
            return null;
        }
        if (!margin.margin || margin.margin.polyline.length < 6) {
            setAbutmentError('Detect or draw a closed margin loop before generating the abutment mesh');
            return null;
        }

        const axis = insertionAxes[fdi]?.vector ?? { x: 0, y: 0, z: 1 };
        setAbutmentBusy(true);
        setAbutmentError(null);
        try {
            const result = await abutmentPort.generateMesh({
                caseFolderPath: props.caseFolderPath,
                outputFileName: `abutment-${fdi}.stl`,
                marginPolyline: margin.margin.polyline.map((point) => [point.x, point.y, point.z]),
                implantAxis: [axis.x, axis.y, axis.z],
                implantDiameterMm: Math.max(3.0, abutment.advanced.abutmentToolDiameterMm * 2),
                emergenceHeightMm: Math.max(
                    2.5,
                    abutment.bottom.emergenceHeightMm +
                        abutment.advanced.profileBorderHeightMm +
                        abutment.bottom.lowerShapeMm +
                        abutment.bottom.upperShapeMm,
                ),
                shoulderWidthMm: abutment.top.shoulderSizeMm,
                taperDegrees: abutment.top.minimumAngleDeg || 6,
                axialRings: 16,
                profile: abutmentStyleToProfile(abutment.top.style),
            });
            setAbutmentOutput(result);
            return result;
        } catch (err) {
            setAbutmentError(err instanceof Error ? err.message : String(err));
            return null;
        } finally {
            setAbutmentBusy(false);
        }
    };

    const stepBody = (() => {
        switch (wizard.state.current) {
            case 'margin':
                return (
                    <MarginDetectPanel
                        toothFdi={props.toothFdi ?? 16}
                        meshPath={props.meshPath ?? null}
                        margin={margin.margin}
                        tool={marginTool}
                        mode={marginMode}
                        isBusy={margin.isBusy}
                        error={margin.error}
                        onChangeTool={setMarginTool}
                        onChangeMode={setMarginMode}
                        onRequestDetect={() =>
                            void margin.detect({
                                meshPath: props.meshPath ?? '',
                                seed: { x: 0, y: 0, z: 0 },
                                mode: marginMode,
                            })
                        }
                        onAdjustLightFromView={() => undefined}
                        onClear={margin.clear}
                        onBack={props.onClose}
                        onNext={() => {
                            wizard.markComplete('margin');
                            wizard.next();
                        }}
                    />
                );
            case 'insertion':
                return (
                    <InsertionDirectionPanel
                        selectedToothFdi={props.toothFdi ?? 16}
                        axes={insertionAxes}
                        uniqueForBridge={uniqueForBridge}
                        isBusy={insertionBusy}
                        error={insertionError}
                        onChangeTooth={() => undefined}
                        onRequestDetect={requestInsertionDetect}
                        onSetCurrentViewAsAxis={() => {
                            // Without a Three.js camera ref the slot can't compute the view-axis;
                            // CadInterface should pass the camera-forward vector once wired.
                            setInsertionError(
                                'Connect the live view to take the current camera direction as axis',
                            );
                        }}
                        onToggleUniqueForBridge={() => setUniqueForBridge((v) => !v)}
                        onBack={() => wizard.back()}
                        onNext={() => {
                            wizard.markComplete('insertion');
                            wizard.next();
                        }}
                    />
                );
            case 'preop-waxup':
                return (
                    <PreopWaxupPanel
                        state={preopWaxupState}
                        preopPath={props.preopPath ?? null}
                        waxupPath={props.waxupPath ?? null}
                        onStrategyChange={(strategy: CopyStrategy) =>
                            setPreopWaxupState((s) => ({ ...s, activeStrategy: strategy, error: null }))
                        }
                        onIterationsChange={(n) =>
                            setPreopWaxupState((s) => ({ ...s, iterations: n }))
                        }
                        onAlignPreop={async () => {
                            if (!props.preopPath || !props.meshPath) {
                                setPreopWaxupState((s) => ({
                                    ...s,
                                    error: 'Pre-op or model path not available',
                                }));
                                return;
                            }
                            try {
                                const alignment = await preopWaxupPort.alignPreop({
                                    preopPath: props.preopPath,
                                    modelPath: props.meshPath,
                                });
                                setPreopWaxupState((s) => ({ ...s, alignment, error: null }));
                            } catch (err) {
                                setPreopWaxupState((s) => ({
                                    ...s,
                                    error: err instanceof Error ? err.message : String(err),
                                }));
                            }
                        }}
                        onAdaptStart={async () => {
                            if (!props.preopPath || !props.meshPath) return;
                            setPreopWaxupState((s) => ({ ...s, isAdapting: true, error: null }));
                            try {
                                const result = await preopWaxupPort.adaptToPreop({
                                    preopPath: props.preopPath,
                                    toothPaths: [props.meshPath],
                                    iterations: preopWaxupState.iterations,
                                });
                                setPreopWaxupState((s) => ({
                                    ...s,
                                    lastAdapt: result,
                                    isAdapting: false,
                                }));
                            } catch (err) {
                                setPreopWaxupState((s) => ({
                                    ...s,
                                    isAdapting: false,
                                    error: err instanceof Error ? err.message : String(err),
                                }));
                            }
                        }}
                        onAdaptStop={() =>
                            setPreopWaxupState((s) => ({ ...s, isAdapting: false }))
                        }
                        onPrepareWaxup={async () => {
                            if (!props.waxupPath) return;
                            try {
                                const waxup = await preopWaxupPort.prepareWaxup({
                                    waxupPath: props.waxupPath,
                                    marginPolylinePerTooth: {},
                                    cropAboveMargin: true,
                                    closeHoles: true,
                                });
                                setPreopWaxupState((s) => ({ ...s, waxup, error: null }));
                            } catch (err) {
                                setPreopWaxupState((s) => ({
                                    ...s,
                                    error: err instanceof Error ? err.message : String(err),
                                }));
                            }
                        }}
                        onResetAlignment={() =>
                            setPreopWaxupState((s) => ({ ...s, alignment: null }))
                        }
                        onBack={() => wizard.back()}
                        onNext={() => {
                            wizard.markComplete('preop-waxup');
                            wizard.next();
                        }}
                    />
                );
            case 'crown-bottoms':
                return (
                    <CrownBottomsPanel
                        material={props.material ?? null}
                        params={crownParams}
                        isApplyBusy={false}
                        onChange={setCrownParams}
                        onApply={() => {
                            wizard.markComplete('crown-bottoms');
                            wizard.next();
                        }}
                        onBack={() => wizard.back()}
                        onNext={() => {
                            wizard.markComplete('crown-bottoms');
                            wizard.next();
                        }}
                    />
                );
            case 'abutment':
                return (
                    <AbutmentDesignPanel
                        design={abutment}
                        onChange={setAbutment}
                        activeTab={abutmentTab}
                        onTabChange={setAbutmentTab}
                        onApplyStyle={() => undefined}
                        onAdjustInsertionDirection={() => wizard.jumpTo('insertion')}
                        onSaveCustomDesign={() => undefined}
                        onResetToInitial={() => setAbutment(defaultAbutmentDesign(abutment.top.style))}
                        onResetCustomized={() => setAbutment(defaultAbutmentDesign())}
                        isApplyBusy={abutmentBusy}
                        statusMessage={
                            abutmentOutput
                                ? `Generated ${abutmentOutput.triangleCount} triangles with ${abutmentOutput.backend}`
                                : null
                        }
                        errorMessage={abutmentError}
                        onApply={() => {
                            void generateAbutmentMesh().then((result) => {
                                if (!result) return;
                                wizard.markComplete('abutment');
                                setAbutmentNextStepOpen(true); // V214 — prompt next-step
                            });
                        }}
                        onBack={() => wizard.back()}
                        onNext={() => {
                            void generateAbutmentMesh().then((result) => {
                                if (!result) return;
                                wizard.markComplete('abutment');
                                setAbutmentNextStepOpen(true);
                            });
                        }}
                    />
                );
            case 'freeforming':
                return (
                    <FreeformingPanel
                        state={freeformState}
                        onChange={setFreeformState}
                        onCutAllIntersections={() => undefined}
                        onApply={() => undefined}
                        onUnload={() => undefined}
                        onUndo={() => undefined}
                        onRedo={() => undefined}
                        onBack={() => wizard.back()}
                        onNext={() => {
                            wizard.markComplete('freeforming');
                            wizard.next();
                        }}
                    />
                );
            case 'connectors':
                return (
                    <ConnectorsPlaceholder
                        onBack={() => wizard.back()}
                        onNext={() => {
                            wizard.markComplete('connectors');
                            wizard.next();
                        }}
                    />
                );
            case 'done':
                if (merge.state.status === 'complete') {
                    return (
                        <NextStepsPanel
                            complete={true}
                            saveSceneInProject={nextStepsSaveScene}
                            onSaveSceneChange={setNextStepsSaveScene}
                            designModelMode={designModelMode}
                            onDesignModelModeChange={setDesignModelMode}
                            onChoose={handleNextStep}
                            onBack={() => merge.reset()}
                        />
                    );
                }
                return (
                    <MergeSavePanel
                        state={merge.state}
                        activeTab={mergeTab}
                        onTabChange={setMergeTab}
                        screwHolesEnabled={teeth.some(
                            (t) =>
                                t.workTypeId?.includes('abutment') ||
                                t.workTypeId?.includes('screw') ||
                                t.workTypeId?.includes('implant'),
                        )}
                        onRestart={() =>
                            void merge.startMerge({
                                caseId: props.caseId ?? 'unknown-case',
                                caseFolderPath: props.caseFolderPath ?? '/tmp',
                                teeth,
                                optimizeFor3dPrint: merge.state.optimizeFor3dPrint,
                            })
                        }
                        onCancel={() => void merge.cancelMerge()}
                        onRemove={() =>
                            void merge.removeMergedParts(props.caseFolderPath ?? '/tmp')
                        }
                        onToggleOptimize={merge.setOptimize}
                        onBack={() => wizard.back()}
                        onNext={() => undefined}
                    />
                );
            default:
                return null;
        }
    })();

    const handleAbutmentNextStep = (step: AbutmentNextStep) => {
        setAbutmentNextStepOpen(false);
        if (step === 'save-only') {
            // Skip to merge — Crown bottoms / freeforming / connectors are
            // not needed when shipping abutments-only.
            wizard.jumpTo('done');
            return;
        }
        // 'continue-suprastructure' — advance through the standard chain.
        wizard.next();
    };

    return (
        <div className="pointer-events-auto fixed right-4 top-[7.2rem] bottom-12 z-30 flex w-[400px] flex-col gap-3">
            <WizardStepper state={wizard.state} onJumpTo={wizard.jumpTo} />
            <div className="min-h-0 flex-1 overflow-hidden">{stepBody}</div>

            {/* V214/V215 — abutment next-step dialog. */}
            <AbutmentNextStepDialog
                open={abutmentNextStepOpen}
                onClose={() => setAbutmentNextStepOpen(false)}
                onChoose={handleAbutmentNextStep}
                abutmentCount={teeth.length}
            />
        </div>
    );
}

function abutmentStyleToProfile(style: AbutmentDesign['top']['style']): AbutmentProfilePreset {
    switch (style) {
        case 'cylindrical':
            return 'Round';
        case 'angular':
            return 'Rectangle';
        case 'legacy':
            return 'Shoulder';
        case 'standard':
        default:
            return 'Default';
    }
}

function ConnectorsPlaceholder({
    onBack,
    onNext,
}: {
    onBack: () => void;
    onNext: () => void;
}) {
    return (
        <aside
            role="dialog"
            aria-label="Connectors step"
            className="pointer-events-auto flex w-[22rem] flex-col gap-2 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
        >
            <header className="text-sm font-semibold">Connectors</header>
            <p className="text-[11px] leading-snug text-slate-300">
                Connector design ships in the V40+ block. For now, advance to Merge & Save.
            </p>
            <footer className="mt-auto flex items-center gap-2 pt-2">
                <button
                    type="button"
                    className="rounded-md border border-white/15 px-3 py-1.5 text-xs"
                    onClick={onBack}
                >
                    ← Back
                </button>
                <button
                    type="button"
                    className="ml-auto rounded-md bg-sky-500 px-3 py-1.5 text-xs font-semibold text-white"
                    onClick={onNext}
                >
                    Next →
                </button>
            </footer>
        </aside>
    );
}
