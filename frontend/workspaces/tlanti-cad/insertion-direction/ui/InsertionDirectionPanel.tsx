/**
 * InsertionDirectionPanel — wizard step UI for insertion axis editing.
 * Replicates exocad doc images #17 and #18: tooth chart, undercut colorbar,
 * "Set current view as insertion axis", unique-for-bridges toggle.
 */

import React, { useMemo } from 'react';

import { Button } from '@/components/ui/button';
import { AppIcon } from '../../app-icons';
import {
    PERMANENT_TEETH,
    type ToothState,
} from '../../tooth-segmentation/domain/fdi-chart';
import { ToothChart } from '../../tooth-segmentation/ui/ToothChart';
import type { InsertionAxis, Vec3 } from '../domain/insertion-axis';
import { UNDERCUT_LEGEND, axisAngleDeg } from '../domain/insertion-axis';

interface InsertionDirectionPanelProps {
    selectedToothFdi: number;
    axes: Record<number, InsertionAxis | undefined>;
    uniqueForBridge: boolean;
    isBusy: boolean;
    error: string | null;
    onChangeTooth: (fdi: number) => void;
    onRequestDetect: () => void;
    onSetCurrentViewAsAxis: () => void;
    onToggleUniqueForBridge: () => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function InsertionDirectionPanel({
    selectedToothFdi,
    axes,
    uniqueForBridge,
    isBusy,
    error,
    onChangeTooth,
    onRequestDetect,
    onSetCurrentViewAsAxis,
    onToggleUniqueForBridge,
    onBack,
    onNext,
}: InsertionDirectionPanelProps) {
    const activeAxis = axes[selectedToothFdi];
    const toothStates = useMemo<Record<number, ToothState>>(() => {
        const out: Record<number, ToothState> = {};
        for (const t of PERMANENT_TEETH) {
            const hasAxis = !!axes[t.fdi];
            out[t.fdi] = {
                fdi: t.fdi,
                status: t.fdi === selectedToothFdi ? 'locked' : hasAxis ? 'segmented' : 'unsegmented',
                color: t.fdi === selectedToothFdi ? '#3b82f6' : hasAxis ? '#22c55e' : null,
            };
        }
        return out;
    }, [axes, selectedToothFdi]);

    const bridgeAxes = Object.values(axes).filter(Boolean) as InsertionAxis[];
    const maxDeviation =
        bridgeAxes.length > 1
            ? Math.max(
                  ...bridgeAxes.map((a) => axisAngleDeg(a.vector, bridgeAxes[0].vector)),
              )
            : 0;

    return (
        <aside
            role="dialog"
            aria-labelledby="insertion-dir-title"
            className="pointer-events-auto flex w-[24rem] flex-col gap-3 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="insertion-direction-panel"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="workflow.insertion-direction" size={18} aria-hidden />
                <h3 id="insertion-dir-title" className="text-sm font-semibold">
                    Insertion Direction
                </h3>
            </header>

            <p className="text-[11px] uppercase tracking-wider text-slate-300">
                Define insertion direction for: {selectedToothFdi}
            </p>

            <div className="rounded-md bg-violet-900/40 p-2">
                <ToothChart
                    states={toothStates}
                    onToothClick={(fdi) => onChangeTooth(fdi)}
                    jaw="both"
                    compact
                />
            </div>

            <label className="flex items-center gap-2 text-[11px] text-slate-200">
                <input
                    type="checkbox"
                    checked={uniqueForBridge}
                    onChange={onToggleUniqueForBridge}
                    className="accent-orange-400"
                />
                Unique insertion direction for bridges
            </label>

            <section className="rounded-md bg-violet-900/40 px-3 py-2">
                <p className="text-[10px] uppercase tracking-wider text-slate-300">
                    Undercut Visualization
                </p>
                <div className="mt-1.5 flex h-2 w-full overflow-hidden rounded">
                    {UNDERCUT_LEGEND.map((stop, i) => {
                        const next = UNDERCUT_LEGEND[i + 1];
                        return (
                            <div
                                key={stop.value}
                                className="h-full flex-1"
                                style={{
                                    background: next
                                        ? `linear-gradient(to right, ${stop.color}, ${next.color})`
                                        : stop.color,
                                }}
                            />
                        );
                    })}
                </div>
                <div className="mt-1 flex justify-between text-[10px] text-slate-300 tabular-nums">
                    {UNDERCUT_LEGEND.map((s) => (
                        <span key={s.value}>{s.label}</span>
                    ))}
                </div>
            </section>

            {activeAxis ? (
                <dl className="grid grid-cols-2 gap-1 rounded-md bg-violet-900/40 px-3 py-2 text-[11px]">
                    <dt className="text-slate-300">Backend</dt>
                    <dd className="text-right font-mono">{activeAxis.backend}</dd>
                    <dt className="text-slate-300">Axis</dt>
                    <dd className="text-right font-mono tabular-nums">
                        ({activeAxis.vector.x.toFixed(2)}, {activeAxis.vector.y.toFixed(2)}, {activeAxis.vector.z.toFixed(2)})
                    </dd>
                    <dt className="text-slate-300">Undercut vol.</dt>
                    <dd className="text-right tabular-nums">{activeAxis.undercutVolumeMm3.toFixed(2)} mm³</dd>
                    <dt className="text-slate-300">Confidence</dt>
                    <dd className="text-right tabular-nums">{(activeAxis.confidence * 100).toFixed(0)}%</dd>
                    {bridgeAxes.length > 1 ? (
                        <>
                            <dt className="text-slate-300">Bridge Δ max</dt>
                            <dd
                                className={[
                                    'text-right tabular-nums',
                                    maxDeviation > 15 ? 'text-amber-300' : 'text-emerald-300',
                                ].join(' ')}
                            >
                                {maxDeviation.toFixed(1)}°
                            </dd>
                        </>
                    ) : null}
                </dl>
            ) : null}

            {error ? (
                <p className="rounded-md border border-red-400/50 bg-red-500/20 px-2 py-1.5 text-[11px]">
                    {error}
                </p>
            ) : null}

            <div className="grid grid-cols-2 gap-1.5">
                <Button type="button" variant="secondary" size="sm" onClick={onRequestDetect} disabled={isBusy}>
                    {isBusy ? 'Detecting…' : 'Detect'}
                </Button>
                <Button type="button" variant="ghost" size="sm" onClick={onSetCurrentViewAsAxis}>
                    Set view as axis
                </Button>
            </div>

            <footer className="mt-1 flex items-center justify-between">
                <Button type="button" variant="ghost" size="sm" onClick={onBack} disabled={!onBack}>
                    ← Back
                </Button>
                <Button type="button" variant="default" size="sm" onClick={onNext} disabled={!activeAxis}>
                    Next →
                </Button>
            </footer>
        </aside>
    );
}
