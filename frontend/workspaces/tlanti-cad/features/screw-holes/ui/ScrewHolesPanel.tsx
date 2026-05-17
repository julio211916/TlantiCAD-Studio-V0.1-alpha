/**
 * ScrewHolesPanel — V92–V93 + V205.
 *
 * Mirrors exocad's "Screw hole design" workflow:
 *   - 3-mode per-tooth picker (anatomy/framework/none) — click cycles, color matches doc
 *   - Anatomy / Framework global default buttons
 *   - Snap All Off — unsticks every channel for free movement
 *   - Thickness / Height global sliders + per-tooth override hints
 *   - Lock indicator per tooth (CTRL+click on a toggle disk applies to all)
 */

import React from 'react';

import { Button } from '@/components/ui/button';
import {
    type ScrewChannelMode,
    type ScrewHolesParams,
    type ScrewHoleToothState,
    applyGlobalMode,
    clampHeightMm,
    clampMinDiameterMm,
    clampOffsetMm,
    clampThicknessMm,
    cycleScrewMode,
    summarizeScrewHoles,
    toggleLocked,
} from '../domain/screw-holes';

export interface ScrewHolesPanelProps {
    params: ScrewHolesParams;
    teeth: readonly ScrewHoleToothState[];
    onParamsChange: (patch: Partial<ScrewHolesParams>) => void;
    onTeethChange: (next: ScrewHoleToothState[]) => void;
    onApply: () => void;
    applying?: boolean;
    disabled?: boolean;
    onBack?: () => void;
    onNext?: () => void;
}

const MODE_COLORS: Record<ScrewChannelMode, { bg: string; border: string; legend: string }> = {
    anatomy: { bg: 'bg-white', border: 'border-slate-300', legend: '#ffffff' },
    framework: { bg: 'bg-emerald-500', border: 'border-emerald-600', legend: '#10b981' },
    none: { bg: 'bg-amber-400', border: 'border-amber-500', legend: '#fbbf24' },
};

export function ScrewHolesPanel({
    params,
    teeth,
    onParamsChange,
    onTeethChange,
    onApply,
    applying,
    disabled,
    onBack,
    onNext,
}: ScrewHolesPanelProps) {
    const summary = summarizeScrewHoles(teeth);

    const handleToothClick = (fdi: string, ev: React.MouseEvent) => {
        const target = teeth.find((t) => t.fdi === fdi);
        if (!target) return;
        if (ev.altKey) {
            // Alt+click toggles lock (mirrors the toggle-disk hotkey in exocad).
            onTeethChange(toggleLocked(teeth, fdi, { all: ev.ctrlKey || ev.metaKey }));
            return;
        }
        const nextMode = cycleScrewMode(target.mode);
        onTeethChange(teeth.map((t) => (t.fdi === fdi ? { ...t, mode: nextMode } : t)));
    };

    return (
        <div className="flex h-full w-[380px] flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-xl">
            <header className="flex items-center gap-3 bg-[#3B2B6F] px-4 py-3 text-white">
                <div className="min-w-0 flex-1">
                    <h2 className="truncate text-sm font-semibold">Screw hole design</h2>
                    <p className="text-[10px] uppercase tracking-wider opacity-80">
                        Per-tooth · Anatomy / Framework / None
                    </p>
                </div>
            </header>

            <div className="flex-1 overflow-y-auto px-4 py-4">
                <div className="mb-3 flex gap-2">
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="flex-1"
                        onClick={() => onTeethChange(applyGlobalMode(teeth, 'anatomy'))}
                        disabled={disabled}
                    >
                        Anatomy
                    </Button>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="flex-1"
                        onClick={() => onTeethChange(applyGlobalMode(teeth, 'framework'))}
                        disabled={disabled}
                    >
                        Framework
                    </Button>
                </div>

                <details open className="mb-3 rounded-md border border-border bg-surface-sunken">
                    <summary className="cursor-pointer px-3 py-2 text-xs font-semibold text-text-primary">
                        Channel parameters
                    </summary>
                    <div className="flex flex-col gap-3 px-3 pb-3">
                        <SliderRow
                            label="Thickness"
                            unit="mm"
                            min={0.2}
                            max={2}
                            step={0.05}
                            value={params.thicknessMm}
                            onChange={(v) =>
                                onParamsChange({ thicknessMm: clampThicknessMm(v) })
                            }
                            disabled={disabled}
                        />
                        <SliderRow
                            label="Height"
                            unit="mm"
                            min={-5}
                            max={5}
                            step={0.1}
                            value={params.heightMm}
                            onChange={(v) =>
                                onParamsChange({ heightMm: clampHeightMm(v) })
                            }
                            disabled={disabled}
                            help="Above zero → channel above anatomy. Below zero → recessed."
                        />
                        <SliderRow
                            label="Cut offset"
                            unit="mm"
                            min={-0.3}
                            max={0.5}
                            step={0.05}
                            value={params.offsetMm}
                            onChange={(v) =>
                                onParamsChange({ offsetMm: clampOffsetMm(v) })
                            }
                            disabled={disabled}
                        />
                        <SliderRow
                            label="Minimum diameter"
                            unit="mm"
                            min={0.5}
                            max={3}
                            step={0.1}
                            value={params.minDiameterMm}
                            onChange={(v) =>
                                onParamsChange({ minDiameterMm: clampMinDiameterMm(v) })
                            }
                            disabled={disabled}
                        />
                        <label className="flex items-center gap-2 text-[11px] text-text-primary">
                            <input
                                type="checkbox"
                                checked={params.snapAllOff}
                                onChange={(e) =>
                                    onParamsChange({ snapAllOff: e.currentTarget.checked })
                                }
                                disabled={disabled}
                                className="accent-sky-400"
                            />
                            <span>
                                Snap All Off — unstick every channel for free movement
                            </span>
                        </label>
                    </div>
                </details>

                <details open className="mb-3 rounded-md border border-border bg-surface-sunken">
                    <summary className="cursor-pointer px-3 py-2 text-xs font-semibold text-text-primary">
                        Individual teeth
                    </summary>
                    <div className="px-3 pb-3">
                        <p className="mb-2 text-[10px] leading-snug text-text-secondary">
                            Click a tooth to cycle Anatomy → Framework → None. Alt+click toggles
                            the channel lock; Alt+Ctrl+click locks/unlocks all.
                        </p>
                        <ToothArch teeth={teeth} onToothClick={handleToothClick} disabled={disabled} />
                        <div className="mt-2 flex flex-wrap items-center gap-3 text-[10px] text-text-secondary">
                            <LegendChip color={MODE_COLORS.anatomy.legend} label={`Anatomy · ${summary.anatomy}`} />
                            <LegendChip
                                color={MODE_COLORS.framework.legend}
                                label={`Framework · ${summary.framework}`}
                            />
                            <LegendChip color={MODE_COLORS.none.legend} label={`No hole · ${summary.none}`} />
                        </div>
                    </div>
                </details>

                <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    className="w-full"
                    onClick={onApply}
                    disabled={disabled || applying}
                >
                    {applying ? 'Applying…' : 'Restart to apply changes'}
                </Button>
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
                >
                    Next →
                </Button>
            </footer>
        </div>
    );
}

function LegendChip({ color, label }: { color: string; label: string }) {
    return (
        <span className="flex items-center gap-1.5">
            <span
                className="inline-block size-2.5 rounded border border-white/40"
                style={{ backgroundColor: color }}
            />
            {label}
        </span>
    );
}

function ToothArch({
    teeth,
    onToothClick,
    disabled,
}: {
    teeth: readonly ScrewHoleToothState[];
    onToothClick: (fdi: string, ev: React.MouseEvent) => void;
    disabled?: boolean;
}) {
    const cols = Math.max(8, Math.ceil(teeth.length / 2));
    return (
        <div
            role="group"
            aria-label="Screw hole per-tooth chart"
            className="grid gap-1"
            style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}
        >
            {teeth.map((tooth) => {
                const colors = MODE_COLORS[tooth.mode];
                return (
                    <button
                        key={tooth.fdi}
                        type="button"
                        onClick={(ev) => onToothClick(tooth.fdi, ev)}
                        disabled={disabled}
                        className={[
                            'relative rounded border px-1 py-1.5 text-center text-[10px] font-semibold transition',
                            colors.bg,
                            colors.border,
                            tooth.mode === 'anatomy' ? 'text-slate-900' : 'text-white',
                            disabled ? 'opacity-60' : 'hover:scale-105',
                        ].join(' ')}
                        title={`Tooth ${tooth.fdi}: ${tooth.mode}${tooth.locked ? ' (locked)' : ' (free)'}`}
                    >
                        <span>{tooth.fdi}</span>
                        {/* Toggle disk indicator: black=locked, green=free */}
                        <span
                            className={[
                                'absolute -bottom-0.5 right-0.5 size-1.5 rounded-full',
                                tooth.locked ? 'bg-slate-900' : 'bg-emerald-400',
                            ].join(' ')}
                            aria-hidden
                        />
                    </button>
                );
            })}
        </div>
    );
}

function SliderRow({
    label,
    unit,
    min,
    max,
    step,
    value,
    onChange,
    disabled,
    help,
}: {
    label: string;
    unit: string;
    min: number;
    max: number;
    step: number;
    value: number;
    onChange: (v: number) => void;
    disabled?: boolean;
    help?: string;
}) {
    const decimals = step >= 1 ? 0 : 2;
    return (
        <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between text-[11px] text-text-primary">
                <span>{label}</span>
                <span className="tabular-nums text-text-secondary">
                    {value.toFixed(decimals)} {unit}
                </span>
            </div>
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => onChange(parseFloat(e.currentTarget.value))}
                disabled={disabled}
                className="accent-sky-400"
            />
            {help ? <span className="text-[10px] text-text-secondary">{help}</span> : null}
        </div>
    );
}
