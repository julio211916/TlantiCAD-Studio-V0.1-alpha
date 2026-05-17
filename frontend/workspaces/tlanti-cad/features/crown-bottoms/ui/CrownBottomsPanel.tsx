/**
 * CrownBottomsPanel — Wizard step UI, 3 tabs (GAP / BORDER / ADVANCED).
 * Pure presentational: parent owns params + callbacks.
 */

import React, { useMemo, useState } from 'react';

import { Button } from '@/components/ui/button';
import { AppIcon } from '../../app-icons';
import type { DentalMaterialType } from '@/lib/dental-workflow';
import {
    defaultCrownBottomParams,
    validateBorder,
    type CementBrushKind,
    type CrownBottomParams,
    type CrownBottomTab,
} from '../domain/crown-bottom-params';

interface CrownBottomsPanelProps {
    material: DentalMaterialType | null;
    params: CrownBottomParams;
    isApplyBusy: boolean;
    onChange: (next: CrownBottomParams) => void;
    onApply: () => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function CrownBottomsPanel({
    material,
    params,
    isApplyBusy,
    onChange,
    onApply,
    onBack,
    onNext,
}: CrownBottomsPanelProps) {
    const [tab, setTab] = useState<CrownBottomTab>('gap');
    const borderWarning = validateBorder(params.border, material);

    return (
        <aside
            role="dialog"
            aria-labelledby="crown-bottoms-title"
            className="pointer-events-auto flex w-[22rem] flex-col gap-2 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="crown-bottoms-panel"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="workflow.wizard-mode" size={18} aria-hidden />
                <h3 id="crown-bottoms-title" className="text-sm font-semibold">
                    Crown Bottoms
                </h3>
            </header>

            <div className="flex gap-1 text-[11px] font-semibold tracking-wider">
                {(['gap', 'border', 'advanced'] as const).map((id) => (
                    <button
                        key={id}
                        type="button"
                        onClick={() => setTab(id)}
                        className={[
                            'flex-1 rounded-md border px-2 py-1 uppercase',
                            tab === id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/20 text-slate-300',
                        ].join(' ')}
                    >
                        {id}
                    </button>
                ))}
            </div>

            {tab === 'gap' ? <GapTab params={params} onChange={onChange} /> : null}
            {tab === 'border' ? <BorderTab params={params} onChange={onChange} warning={borderWarning} /> : null}
            {tab === 'advanced' ? <AdvancedTab params={params} onChange={onChange} /> : null}

            <div className="mt-2 flex items-center justify-between gap-2">
                <Button type="button" variant="ghost" size="sm" onClick={onBack} disabled={!onBack}>
                    ← Back
                </Button>
                <div className="flex items-center gap-2">
                    <Button type="button" variant="secondary" size="sm" onClick={onApply} disabled={isApplyBusy}>
                        {isApplyBusy ? 'Applying…' : 'Apply'}
                    </Button>
                    <Button type="button" variant="default" size="sm" onClick={onNext} disabled={!onNext || isApplyBusy}>
                        Next →
                    </Button>
                </div>
            </div>

            <button
                type="button"
                className="mt-1 text-[10px] uppercase tracking-wider text-slate-400 hover:text-slate-200"
                onClick={() => onChange(defaultCrownBottomParams(material))}
            >
                Reset to material defaults
            </button>
        </aside>
    );
}

function GapTab({
    params,
    onChange,
}: {
    params: CrownBottomParams;
    onChange: (p: CrownBottomParams) => void;
}) {
    const setBrush = (kind: CementBrushKind) =>
        onChange({ ...params, gap: { ...params.gap, activeBrush: kind } });
    const setGapMm = (kind: CementBrushKind, value: number) =>
        onChange({
            ...params,
            gap: {
                ...params.gap,
                zones: { ...params.gap.zones, [kind]: { ...params.gap.zones[kind], gapMm: value } },
            },
        });

    return (
        <div className="flex flex-col gap-3">
            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1.5 text-[10px] uppercase tracking-wider text-slate-300">Cement Gap</p>
                {(Object.keys(params.gap.zones) as CementBrushKind[]).map((kind) => {
                    const z = params.gap.zones[kind];
                    const active = params.gap.activeBrush === kind;
                    return (
                        <div key={kind} className="mb-2 flex items-center gap-2">
                            <button
                                type="button"
                                aria-pressed={active}
                                onClick={() => setBrush(kind)}
                                className={[
                                    'h-5 w-5 rounded-sm border-2',
                                    active ? 'ring-2 ring-orange-400' : '',
                                ].join(' ')}
                                style={{ background: z.color, borderColor: z.color }}
                            />
                            <span className="flex-1 text-[11px]">{z.label}</span>
                            {kind !== 'no-cement' ? (
                                <input
                                    type="number"
                                    step="0.01"
                                    min={0}
                                    max={1}
                                    value={z.gapMm}
                                    onChange={(e) => setGapMm(kind, parseFloat(e.target.value) || 0)}
                                    className="w-16 rounded border border-white/20 bg-violet-900/60 px-1.5 py-0.5 text-right text-[11px] tabular-nums"
                                />
                            ) : null}
                            {kind !== 'no-cement' ? <span className="text-[10px] text-slate-400">mm</span> : null}
                            <AppIcon name="workflow.cement-brush" size={14} aria-hidden />
                        </div>
                    );
                })}
            </section>

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1.5 text-[10px] uppercase tracking-wider text-slate-300">
                    Select Zones by Distance
                </p>
                <label className="flex items-center gap-2 text-[11px]">
                    <span className="inline-block h-3 w-3 rounded-sm bg-emerald-400" />
                    <span className="flex-1">From margin</span>
                    <input
                        type="number"
                        step="0.1"
                        min={0}
                        max={5}
                        value={params.gap.distanceFromMarginMm}
                        onChange={(e) =>
                            onChange({
                                ...params,
                                gap: { ...params.gap, distanceFromMarginMm: parseFloat(e.target.value) || 0 },
                            })
                        }
                        className="w-16 rounded border border-white/20 bg-violet-900/60 px-1.5 py-0.5 text-right text-[11px] tabular-nums"
                    />
                    <span className="text-[10px] text-slate-400">mm</span>
                </label>
            </section>

            <section className="rounded-md bg-violet-900/40 p-2">
                <div className="mb-1 flex items-center gap-2">
                    <p className="flex-1 text-[10px] uppercase tracking-wider text-slate-300">
                        Additional Spacing
                    </p>
                    <button
                        type="button"
                        onClick={() =>
                            onChange({
                                ...params,
                                gap: { ...params.gap, lockAxialRadial: !params.gap.lockAxialRadial },
                            })
                        }
                        aria-pressed={params.gap.lockAxialRadial}
                        className="text-[11px] text-orange-300"
                    >
                        {params.gap.lockAxialRadial ? '🔒 locked' : '🔓 free'}
                    </button>
                </div>
                <SliderRow
                    label="Axial"
                    value={params.gap.axialSpacingMm}
                    min={0}
                    max={0.5}
                    step={0.01}
                    onChange={(v) =>
                        onChange({
                            ...params,
                            gap: {
                                ...params.gap,
                                axialSpacingMm: v,
                                radialSpacingMm: params.gap.lockAxialRadial ? v : params.gap.radialSpacingMm,
                            },
                        })
                    }
                />
                <SliderRow
                    label="Radial"
                    value={params.gap.radialSpacingMm}
                    min={0}
                    max={0.5}
                    step={0.01}
                    onChange={(v) =>
                        onChange({
                            ...params,
                            gap: {
                                ...params.gap,
                                radialSpacingMm: v,
                                axialSpacingMm: params.gap.lockAxialRadial ? v : params.gap.axialSpacingMm,
                            },
                        })
                    }
                />
            </section>
        </div>
    );
}

function BorderTab({
    params,
    onChange,
    warning,
}: {
    params: CrownBottomParams;
    onChange: (p: CrownBottomParams) => void;
    warning: string | null;
}) {
    const b = params.border;
    const set = (patch: Partial<typeof b>) => onChange({ ...params, border: { ...b, ...patch } });

    return (
        <div className="flex flex-col gap-2">
            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1.5 text-[10px] uppercase tracking-wider text-slate-300">
                    Crown Border Parameters
                </p>
                <SliderRow label="1. Horizontal" value={b.horizontalMm} min={0.05} max={1} step={0.01} onChange={(v) => set({ horizontalMm: v })} />
                <SliderRow label="2. Angled" value={b.angledMm} min={0} max={1} step={0.01} onChange={(v) => set({ angledMm: v })} />
                <SliderRow label="3. Angle" value={b.angleDeg} min={0} max={90} step={1} unit="°" onChange={(v) => set({ angleDeg: v })} />
                <SliderRow label="4. Vertical" value={b.verticalMm} min={0} max={1} step={0.01} onChange={(v) => set({ verticalMm: v })} />
                <SliderRow label="5. Below margin" value={b.belowMarginMm} min={-0.5} max={0.5} step={0.01} onChange={(v) => set({ belowMarginMm: v })} />
            </section>

            {warning ? (
                <p className="rounded-md border border-amber-400/40 bg-amber-500/20 p-2 text-[11px] text-amber-100">
                    ⚠ {warning}
                </p>
            ) : null}

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1 text-[10px] uppercase tracking-wider text-slate-300">Parameter Explanation</p>
                <svg viewBox="0 0 160 80" className="h-20 w-full">
                    <path d="M20 70 L60 70 L70 55 L100 55 L110 30 L140 30" stroke="#fbbf24" strokeWidth="6" fill="none" />
                    <text x="68" y="68" fill="#fca5a5" fontSize="8">1</text>
                    <text x="80" y="52" fill="#fca5a5" fontSize="8">2</text>
                    <text x="100" y="45" fill="#fca5a5" fontSize="8">3</text>
                    <text x="125" y="27" fill="#fca5a5" fontSize="8">4</text>
                    <text x="30" y="67" fill="#fca5a5" fontSize="8">5</text>
                </svg>
            </section>
        </div>
    );
}

function AdvancedTab({
    params,
    onChange,
}: {
    params: CrownBottomParams;
    onChange: (p: CrownBottomParams) => void;
}) {
    const a = params.advanced;
    const set = (patch: Partial<typeof a>) => onChange({ ...params, advanced: { ...a, ...patch } });

    return (
        <div className="flex flex-col gap-2">
            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1.5 text-[10px] uppercase tracking-wider text-slate-300">Undercuts</p>
                <label className="flex items-center gap-2 text-[11px]">
                    <input
                        type="checkbox"
                        checked={a.dontBlockOutUndercuts}
                        onChange={(e) => set({ dontBlockOutUndercuts: e.target.checked })}
                        className="accent-orange-400"
                    />
                    Do not block out undercuts
                </label>
                <SliderRow label="Angle" value={a.blockOutAngleDeg} min={0} max={20} step={0.5} unit="°" onChange={(v) => set({ blockOutAngleDeg: v })} />
                <p className="mt-1 text-[10px] text-slate-300">Protected zone near margin line</p>
                <SliderRow label="Size" value={a.protectedZoneSizeMm} min={0} max={2} step={0.01} onChange={(v) => set({ protectedZoneSizeMm: v })} />
            </section>

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1.5 text-[10px] uppercase tracking-wider text-slate-300">Milling</p>
                <label className="flex items-center gap-2 text-[11px]">
                    <input
                        type="checkbox"
                        checked={a.anticipateMilling}
                        onChange={(e) => set({ anticipateMilling: e.target.checked })}
                        className="accent-orange-400"
                    />
                    Anticipate milling
                </label>
                <SliderRow label="Diameter" value={a.toolDiameterMm} min={0.3} max={2.5} step={0.1} onChange={(v) => set({ toolDiameterMm: v })} />
                <label className="mt-1 flex items-center gap-2 text-[11px]">
                    <input
                        type="checkbox"
                        checked={a.bullnoseTool}
                        onChange={(e) => set({ bullnoseTool: e.target.checked })}
                        className="accent-orange-400"
                    />
                    Bullnose / flat tool
                </label>
                <SliderRow
                    label="Tool tip flat"
                    value={a.toolTipFlatPercent}
                    min={0}
                    max={100}
                    step={5}
                    unit="%"
                    onChange={(v) => set({ toolTipFlatPercent: v })}
                    disabled={!a.bullnoseTool}
                />
            </section>

            <label className="flex items-center gap-2 text-[11px]">
                <input
                    type="checkbox"
                    checked={a.showUndercuts}
                    onChange={(e) => set({ showUndercuts: e.target.checked })}
                    className="accent-orange-400"
                />
                Show undercuts
            </label>
        </div>
    );
}

function SliderRow({
    label,
    value,
    min,
    max,
    step,
    unit = 'mm',
    onChange,
    disabled,
}: {
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    unit?: string;
    onChange: (v: number) => void;
    disabled?: boolean;
}) {
    return (
        <div className={['mb-1', disabled ? 'opacity-50' : ''].join(' ')}>
            <div className="flex items-center justify-between text-[11px]">
                <span>{label}</span>
                <span className="tabular-nums text-slate-300">
                    {value.toFixed(step < 0.05 ? 2 : step < 1 ? 1 : 0)} {unit}
                </span>
            </div>
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                disabled={disabled}
                onChange={(e) => onChange(parseFloat(e.target.value))}
                className="w-full accent-orange-400"
            />
        </div>
    );
}
