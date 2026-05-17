/**
 * ArticulatorPanel — V124 wizard UI.
 *
 * Sliders for condyle inclination, Bennett angle, immediate side shift,
 * intercondylar distance, incisal guidance + movement picker
 * (protrusive / laterotrusive / retrusive) + Recalculate Articulator
 * Simulation button. Pure presentational.
 */

import React from 'react';

import { Icon } from '@tlanticad/ui';
import type { ArticulatorConfig, JawMovement } from '../domain/articulator-config';
import { defaultArticulatorConfig } from '../domain/articulator-config';

export interface ArticulatorPanelProps {
    config: ArticulatorConfig;
    onConfigChange: (next: ArticulatorConfig) => void;
    movement: JawMovement;
    onMovementChange: (movement: JawMovement) => void;
    onRecalculate: () => void;
    onResetDefaults: () => void;
    onChooseInfluencingTeeth: () => void;
    isBusy?: boolean;
    error?: string | null;
    onBack?: () => void;
    onNext?: () => void;
}

export function ArticulatorPanel({
    config,
    onConfigChange,
    movement,
    onMovementChange,
    onRecalculate,
    onResetDefaults,
    onChooseInfluencingTeeth,
    isBusy,
    error,
    onBack,
    onNext,
}: ArticulatorPanelProps) {
    const updateField = <K extends keyof ArticulatorConfig>(key: K, value: number) =>
        onConfigChange({ ...config, [key]: value });

    return (
        <aside
            role="dialog"
            aria-labelledby="articulator-title"
            className="pointer-events-auto flex w-[22rem] flex-col gap-3 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="articulator-panel"
        >
            <header className="flex items-center gap-2">
                <Icon
                    name="articulator.bonwill-triangle"
                    size={18}
                    className="text-sky-300"
                    aria-hidden
                />
                <h3 id="articulator-title" className="text-sm font-semibold">
                    Virtual Articulator
                </h3>
                <button
                    type="button"
                    onClick={onResetDefaults}
                    className="ml-auto rounded border border-white/15 px-2 py-0.5 text-[10px] uppercase tracking-wider text-slate-300 hover:bg-white/10"
                >
                    Reset
                </button>
            </header>

            <SliderRow
                label="Condyle inclination"
                unit="°"
                min={0}
                max={60}
                step={0.5}
                value={config.condyleInclinationDeg}
                onChange={(v) => updateField('condyleInclinationDeg', v)}
            />
            <SliderRow
                label="Bennett angle"
                unit="°"
                min={0}
                max={20}
                step={0.5}
                value={config.bennettAngleDeg}
                onChange={(v) => updateField('bennettAngleDeg', v)}
            />
            <SliderRow
                label="Immediate side shift"
                unit="mm"
                min={0}
                max={3}
                step={0.05}
                value={config.immediateSideShiftMm}
                onChange={(v) => updateField('immediateSideShiftMm', v)}
            />
            <SliderRow
                label="Intercondylar distance"
                unit="mm"
                min={80}
                max={140}
                step={1}
                value={config.intercondylarDistanceMm}
                onChange={(v) => updateField('intercondylarDistanceMm', v)}
            />
            <SliderRow
                label="Incisal guidance"
                unit="°"
                min={0}
                max={45}
                step={0.5}
                value={config.incisalGuidanceDeg}
                onChange={(v) => updateField('incisalGuidanceDeg', v)}
            />

            <div className="flex flex-col gap-1.5 border-t border-white/10 pt-2">
                <p className="flex items-center gap-1 text-[10px] uppercase tracking-wider text-slate-300">
                    <Icon name="articulator.jaw-motion" size={12} aria-hidden />
                    Movement
                </p>
                <div className="flex gap-1">
                    {(['protrusive', 'laterotrusive', 'retrusive'] as JawMovement[]).map((m) => (
                        <button
                            key={m}
                            type="button"
                            onClick={() => onMovementChange(m)}
                            className={[
                                'flex-1 rounded-md border px-2 py-1.5 text-[11px] capitalize transition',
                                m === movement
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/15 text-slate-300 hover:bg-white/10',
                            ].join(' ')}
                        >
                            {m}
                        </button>
                    ))}
                </div>
            </div>

            <button
                type="button"
                onClick={onChooseInfluencingTeeth}
                className="rounded-md border border-white/15 px-2 py-1.5 text-[11px] text-slate-200 hover:bg-white/10"
            >
                Choose teeth that influence articulator movement…
            </button>

            <button
                type="button"
                onClick={onRecalculate}
                disabled={isBusy}
                className="rounded-md bg-sky-500 px-3 py-2 text-xs font-semibold text-white disabled:opacity-50"
            >
                {isBusy ? 'Recalculating…' : 'Recalculate articulator simulation'}
            </button>

            {error ? (
                <p className="rounded border border-rose-500/40 bg-rose-500/10 px-2 py-1 text-[11px] text-rose-200">
                    {error}
                </p>
            ) : null}

            <footer className="mt-auto flex items-center gap-2 border-t border-white/10 pt-2">
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

function SliderRow({
    label,
    unit,
    value,
    onChange,
    min,
    max,
    step,
}: {
    label: string;
    unit: string;
    value: number;
    onChange: (next: number) => void;
    min: number;
    max: number;
    step: number;
}) {
    return (
        <label className="flex flex-col gap-1 text-[11px]">
            <span className="flex items-center justify-between">
                <span>{label}</span>
                <span className="font-mono tabular-nums text-slate-300">
                    {value.toFixed(step < 1 ? 2 : 0)} {unit}
                </span>
            </span>
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => onChange(parseFloat(e.currentTarget.value))}
                className="accent-sky-400"
            />
        </label>
    );
}

ArticulatorPanel.defaultConfig = defaultArticulatorConfig;
