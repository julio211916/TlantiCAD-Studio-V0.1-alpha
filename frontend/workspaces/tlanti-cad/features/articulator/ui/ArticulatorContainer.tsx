/**
 * ArticulatorContainer — V207 + V209 + V210 + V211.
 *
 * Stateful wrapper:
 *  - owns config + movement + simulation
 *  - opens the ArticulatorLibraryPicker (V208) and applies vendor presets
 *  - relays jaw frames to the canvas overlay
 *  - relays "Choose teeth" to the parent (V211)
 *
 * Vendor and influencing-teeth state is bidirectional via props so the
 * parent can persist them in `case.articulator`.
 */

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import {
    type ArticulatorConfig,
    type JawFrame,
    type JawMovement,
    defaultArticulatorConfig,
} from '../domain/articulator-config';
import {
    type ArticulatorPort,
    type ArticulatorSimulateOutput,
} from '../application/articulator-port';
import { createBackendArticulatorAdapter } from '../infrastructure/backend-articulator-adapter';
import { ArticulatorPanel } from './ArticulatorPanel';
import {
    ArticulatorLibraryPicker,
    createBackendArticulatorLibraryAdapter,
    defaultArticulatorLibraryState,
    type ArticulatorLibraryPort,
    type ArticulatorLibraryState,
} from '../../articulator-library';

export interface ArticulatorContainerProps {
    open: boolean;
    onClose: () => void;
    /** Optional override of the simulate port. */
    port?: ArticulatorPort;
    /** Optional override of the library port. */
    libraryPort?: ArticulatorLibraryPort;
    /** Notified whenever a simulation completes — canvas can subscribe. */
    onFramesChange?: (frames: JawFrame[]) => void;
    /** Notified when "Choose teeth" is clicked. */
    onOpenInfluencingTeeth?: () => void;
    /** Persisted vendor — when set, applied on mount. */
    vendorId?: string | null;
    /** Notified when the user picks a different vendor in the library picker. */
    onVendorChange?: (vendor: { id: string; label: string } | null) => void;
}

export function ArticulatorContainer({
    open,
    onClose,
    port,
    libraryPort,
    onFramesChange,
    onOpenInfluencingTeeth,
    vendorId,
    onVendorChange,
}: ArticulatorContainerProps) {
    const portRef = useRef<ArticulatorPort | null>(null);
    if (portRef.current === null) {
        portRef.current = port ?? createBackendArticulatorAdapter();
    }
    const libraryPortRef = useRef<ArticulatorLibraryPort | null>(null);
    if (libraryPortRef.current === null) {
        libraryPortRef.current = libraryPort ?? createBackendArticulatorLibraryAdapter();
    }

    const [config, setConfig] = useState<ArticulatorConfig>(() => defaultArticulatorConfig());
    const [movement, setMovement] = useState<JawMovement>('protrusive');
    const [isBusy, setIsBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [pickerOpen, setPickerOpen] = useState(false);
    const [libraryState, setLibraryState] = useState<ArticulatorLibraryState>(() =>
        defaultArticulatorLibraryState(),
    );

    // Lazy-load the catalog the first time the picker opens.
    useEffect(() => {
        if (!pickerOpen || libraryState.vendors.length > 0 || libraryState.isLoading) return;
        let cancelled = false;
        setLibraryState((s) => ({ ...s, isLoading: true, error: null }));
        libraryPortRef
            .current!.list()
            .then((dto) => {
                if (cancelled) return;
                setLibraryState((s) => ({
                    ...s,
                    vendors: dto.vendors,
                    backend: dto.backend,
                    isLoading: false,
                }));
            })
            .catch((err) => {
                if (cancelled) return;
                setLibraryState((s) => ({
                    ...s,
                    error: err instanceof Error ? err.message : String(err),
                    isLoading: false,
                }));
            });
        return () => {
            cancelled = true;
        };
    }, [pickerOpen, libraryState.vendors.length, libraryState.isLoading]);

    const applyVendor = useCallback(
        async (id: string) => {
            try {
                const preset = await libraryPortRef.current!.getPreset(id);
                setConfig(preset.config);
                setLibraryState((s) => ({ ...s, activeVendorId: id, activePreset: preset }));
                onVendorChange?.({ id: preset.id, label: preset.label });
            } catch (err) {
                setLibraryState((s) => ({
                    ...s,
                    error: err instanceof Error ? err.message : String(err),
                }));
            }
        },
        [onVendorChange],
    );

    // Apply persisted vendor on mount / when prop changes.
    const lastApplied = useRef<string | null>(null);
    useEffect(() => {
        if (vendorId && vendorId !== lastApplied.current) {
            lastApplied.current = vendorId;
            void applyVendor(vendorId);
        }
    }, [vendorId, applyVendor]);

    const onRecalculate = useMemo(
        () => async () => {
            if (!portRef.current) return;
            setIsBusy(true);
            setError(null);
            try {
                const result: ArticulatorSimulateOutput = await portRef.current.simulate({
                    config,
                    movement,
                    frames: 9,
                });
                onFramesChange?.(result.frames);
            } catch (err) {
                setError(err instanceof Error ? err.message : String(err));
            } finally {
                setIsBusy(false);
            }
        },
        [config, movement, onFramesChange],
    );

    if (!open && !pickerOpen) return null;

    return (
        <>
            {open ? (
                <div
                    className="pointer-events-none fixed inset-0 z-30 flex items-end justify-end p-4"
                    aria-label="Articulator floating panel"
                >
                    <div className="pointer-events-auto flex flex-col gap-2">
                        {libraryState.activePreset ? (
                            <div className="rounded-md border border-emerald-400/40 bg-emerald-500/10 px-3 py-1.5 text-[11px] text-emerald-200">
                                Vendor preset · {libraryState.activePreset.label}
                            </div>
                        ) : null}
                        <button
                            type="button"
                            onClick={() => setPickerOpen(true)}
                            className="rounded-md border border-white/15 bg-violet-900/30 px-3 py-1.5 text-[11px] text-slate-100 hover:bg-white/10"
                        >
                            {libraryState.activePreset
                                ? 'Change vendor preset…'
                                : 'Browse articulator library…'}
                        </button>
                        <ArticulatorPanel
                            config={config}
                            onConfigChange={setConfig}
                            movement={movement}
                            onMovementChange={setMovement}
                            onRecalculate={onRecalculate}
                            onResetDefaults={() => {
                                setConfig(defaultArticulatorConfig());
                                setLibraryState((s) => ({
                                    ...s,
                                    activeVendorId: null,
                                    activePreset: null,
                                }));
                                onVendorChange?.(null);
                            }}
                            onChooseInfluencingTeeth={onOpenInfluencingTeeth ?? (() => undefined)}
                            isBusy={isBusy}
                            error={error}
                            onBack={onClose}
                            onNext={onClose}
                        />
                    </div>
                </div>
            ) : null}

            <ArticulatorLibraryPicker
                open={pickerOpen}
                state={libraryState}
                onClose={() => setPickerOpen(false)}
                onSelect={(id) => {
                    void applyVendor(id);
                    setPickerOpen(false);
                }}
            />
        </>
    );
}
