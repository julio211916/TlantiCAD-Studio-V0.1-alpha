/**
 * CrownSegmentationPanel — replica of the RealGUIDE "Crown Segmentation"
 * workflow: arch toggle, tooth chart, toggles, automatic segmentation.
 *
 * The component is presentational + a hook driver; domain/ports live in
 * siblings. Pass a `scanRef` (mesh path or study uid) plus a port factory.
 */

import React, { useCallback, useMemo, useState } from 'react';

import { Button } from '@/components/ui/button';
import { AppIcon } from '../../app-icons';

import { ToothChart } from './ToothChart';
import {
    PERMANENT_TEETH,
    defaultColorFor,
    findTooth,
    teethOfJaw,
    type JawKind,
    type ToothState,
    type ToothStatus,
} from '../domain/fdi-chart';
import { useCrownSegmentation } from './useCrownSegmentation';
import type {
    CrownSegmentationLaunchArgs,
    ToothSegmentationPort,
} from '../application/tooth-segmentation-port';

interface CrownSegmentationPanelProps {
    scanRef: CrownSegmentationLaunchArgs['scanRef'];
    portFactory: () => ToothSegmentationPort;
    onClose: () => void;
    previewSrc?: string;
}

function emptyStates(): Record<number, ToothState> {
    const out: Record<number, ToothState> = {};
    for (const t of PERMANENT_TEETH) {
        out[t.fdi] = { fdi: t.fdi, status: 'unsegmented', color: null };
    }
    return out;
}

function mergeSegmented(
    current: Record<number, ToothState>,
    segmented: number[],
): Record<number, ToothState> {
    const next = { ...current };
    for (const fdi of segmented) {
        const prev = next[fdi];
        if (!prev || prev.status === 'missing' || prev.status === 'locked') continue;
        next[fdi] = {
            fdi,
            status: 'segmented',
            color: prev.color ?? defaultColorFor(fdi),
        };
    }
    return next;
}

export function CrownSegmentationPanel({
    scanRef,
    portFactory,
    onClose,
    previewSrc,
}: CrownSegmentationPanelProps) {
    const [jaw, setJaw] = useState<JawKind>('maxilla');
    const [extractGingiva, setExtractGingiva] = useState(false);
    const [keepSegmented, setKeepSegmented] = useState(true);
    const [states, setStates] = useState<Record<number, ToothState>>(() => emptyStates());

    const { job, isRunning, launch, cancel, reset } = useCrownSegmentation(portFactory);

    // Lift segmented teeth from the live job into the chart state.
    const chartStates = useMemo(
        () => (job?.segmentedTeeth ? mergeSegmented(states, job.segmentedTeeth) : states),
        [job?.segmentedTeeth, states],
    );

    const handleToothClick = useCallback((fdi: number) => {
        setStates((prev) => {
            const cur = prev[fdi];
            const nextStatus: ToothStatus =
                cur?.status === 'segmented' ? 'locked' :
                cur?.status === 'locked' ? 'unsegmented' :
                'segmented';
            return {
                ...prev,
                [fdi]: {
                    fdi,
                    status: nextStatus,
                    color: nextStatus === 'unsegmented' ? null : defaultColorFor(fdi),
                },
            };
        });
    }, []);

    const handleToothAltClick = useCallback((fdi: number) => {
        setStates((prev) => ({
            ...prev,
            [fdi]: { fdi, status: 'missing', color: null },
        }));
    }, []);

    const handleClearAll = useCallback(() => {
        setStates(emptyStates());
        reset();
    }, [reset]);

    const handleRun = useCallback(() => {
        const skipTeeth = Object.values(states)
            .filter((s) => s.status === 'missing')
            .map((s) => s.fdi);
        launch({
            jaw,
            extractGingiva,
            keepSegmented,
            skipTeeth: skipTeeth.length ? skipTeeth : undefined,
            scanRef,
        });
    }, [states, jaw, extractGingiva, keepSegmented, scanRef, launch]);

    const visibleToothCount = teethOfJaw(jaw).length;
    const segmentedCount = Object.values(chartStates).filter(
        (s) => (s.status === 'segmented' || s.status === 'locked') && findTooth(s.fdi)?.jaw === jaw,
    ).length;

    return (
        <aside
            role="dialog"
            aria-labelledby="crown-segmentation-title"
            className="pointer-events-auto flex w-[22rem] flex-col gap-4 rounded-lg border border-border bg-surface-raised/95 p-4 shadow-xl backdrop-blur"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="crown-seg.toggle" size={18} aria-hidden />
                <h3
                    id="crown-segmentation-title"
                    className="flex-1 text-sm font-semibold text-text-primary"
                >
                    Crown Segmentation
                </h3>
                <button
                    type="button"
                    aria-label="Close panel"
                    onClick={onClose}
                    className="text-text-secondary hover:text-text-primary"
                >
                    <AppIcon name="common.close" size={16} aria-hidden />
                </button>
            </header>

            {previewSrc ? (
                <div className="overflow-hidden rounded-md bg-surface-sunken">
                    <img
                        src={previewSrc}
                        alt="Scan preview"
                        className="h-32 w-full object-cover"
                    />
                </div>
            ) : null}

            <div className="grid grid-cols-2 gap-2" role="group" aria-label="Select jaw">
                <button
                    type="button"
                    onClick={() => setJaw('maxilla')}
                    className={[
                        'flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-xs font-medium transition',
                        jaw === 'maxilla'
                            ? 'border-sky-400 bg-sky-500/10 text-text-primary'
                            : 'border-border bg-surface-sunken text-text-secondary hover:bg-surface-raised',
                    ].join(' ')}
                >
                    <AppIcon name="crown-seg.maxilla" size={16} aria-hidden />
                    Maxilla
                </button>
                <button
                    type="button"
                    onClick={() => setJaw('mandible')}
                    className={[
                        'flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-xs font-medium transition',
                        jaw === 'mandible'
                            ? 'border-sky-400 bg-sky-500/10 text-text-primary'
                            : 'border-border bg-surface-sunken text-text-secondary hover:bg-surface-raised',
                    ].join(' ')}
                >
                    <AppIcon name="crown-seg.mandible" size={16} aria-hidden />
                    Mandible
                </button>
            </div>

            <p className="text-[0.6875rem] leading-snug text-text-secondary">
                Click a tooth to add / select a crown.
                <br />
                ALT + click to mark as missing. Click again to unpin.
            </p>

            <ToothChart
                states={chartStates}
                onToothClick={handleToothClick}
                onToothAltClick={handleToothAltClick}
                jaw={jaw}
                compact
            />

            <div className="flex items-center justify-between text-[0.6875rem] text-text-secondary">
                <span>{segmentedCount} / {visibleToothCount} teeth detected</span>
                <button
                    type="button"
                    onClick={handleClearAll}
                    className="flex items-center gap-1 text-text-secondary hover:text-text-primary"
                >
                    <AppIcon name="crown-seg.clear-all" size={14} aria-hidden />
                    Clear All
                </button>
            </div>

            <ToggleRow
                icon="crown-seg.extract-gingiva"
                label="Extract Gingiva Segmentation"
                checked={extractGingiva}
                onChange={setExtractGingiva}
            />

            <ToggleRow
                icon="crown-seg.keep-crowns"
                label="Keep Segmented Crowns"
                checked={keepSegmented}
                onChange={setKeepSegmented}
            />

            {job?.error ? (
                <p className="rounded-md border border-red-500/40 bg-red-500/10 px-2 py-1.5 text-xs text-red-100">
                    {job.error}
                </p>
            ) : null}

            {isRunning ? (
                <div className="space-y-1.5">
                    <div
                        className="h-1.5 overflow-hidden rounded-full bg-surface-sunken"
                        role="progressbar"
                        aria-valuenow={Math.round((job?.progress ?? 0) * 100)}
                    >
                        <div
                            className="h-full bg-sky-400/80 transition-[width]"
                            style={{ width: `${Math.max(2, Math.round((job?.progress ?? 0) * 100))}%` }}
                        />
                    </div>
                    <p className="text-[0.6875rem] text-text-secondary">
                        Running TGN inference — {Math.round((job?.progress ?? 0) * 100)}%
                    </p>
                </div>
            ) : null}

            <footer className="flex gap-2">
                {isRunning ? (
                    <Button type="button" variant="ghost" size="sm" onClick={cancel}>
                        Cancel
                    </Button>
                ) : null}
                <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    className="ml-auto"
                    onClick={handleRun}
                    disabled={isRunning}
                >
                    <AppIcon name="crown-seg.auto" size={14} aria-hidden />
                    <span className="ml-2">Automatic Segmentation</span>
                </Button>
            </footer>
        </aside>
    );
}

function ToggleRow({
    icon,
    label,
    checked,
    onChange,
}: {
    icon: string;
    label: string;
    checked: boolean;
    onChange: (v: boolean) => void;
}) {
    return (
        <label className="flex cursor-pointer items-center gap-2 text-xs text-text-secondary">
            <button
                type="button"
                role="switch"
                aria-checked={checked}
                onClick={() => onChange(!checked)}
                className={[
                    'relative h-4 w-7 rounded-full border transition-colors',
                    checked ? 'border-sky-400 bg-sky-500/60' : 'border-border bg-surface-sunken',
                ].join(' ')}
            >
                <span
                    className={[
                        'absolute top-[1px] h-[14px] w-[14px] rounded-full bg-white transition-transform',
                        checked ? 'translate-x-[12px]' : 'translate-x-[1px]',
                    ].join(' ')}
                />
            </button>
            <AppIcon name={icon} size={14} aria-hidden />
            <span>{label}</span>
        </label>
    );
}
