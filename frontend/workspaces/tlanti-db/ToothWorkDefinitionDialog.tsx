/**
 * ToothWorkDefinitionDialog — exocad-faithful "Defining jobs for individual
 * teeth" dialog. Replaces the old right-side sheet with a 3-column layout:
 *
 *   [ WORK TYPE grid ]  [ MATERIAL grid ]  [ OPTIONS & PARAMETERS ]
 *
 * See exocad docs → "Defining jobs for individual teeth" for the full
 * catalog (crowns/copings, pontics, inlays, waxup, removables, bars,
 * residual dentition). The catalog lives in `lib/dental-work-catalog.ts`.
 */

import React, { useMemo, useState } from 'react';
import { ArrowLeft, Check, CheckCircle2, Eraser, HelpCircle } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { getToothNumberingLegend, type DentalNumberingSystem } from '@/lib/dental-numbering';
import type { TlantiToothState } from '@/stores/tlantidb-case-store';
import type { DentalMaterialType } from '@/lib/dental-workflow';
import {
    DENTAL_MATERIALS,
    DENTAL_WORK_CATEGORIES,
    DENTAL_WORK_TYPES,
    IMPLANT_OPTIONS,
    MATERIAL_SHADES,
    PRODUCTION_METHODS,
    resolveWorkType,
    type DentalWorkType,
    type ImplantOption,
} from '@/lib/dental-work-catalog';

interface ToothWorkDefinitionDialogProps {
    open: boolean;
    toothNumber: string | null;
    numberingSystem: DentalNumberingSystem;
    toothState?: TlantiToothState;
    onCancel: () => void;
    onClear: (toothNumber: string) => void;
    onConfirm: (toothNumber: string, patch: {
        workTypeId: string;
        legacyRestorationType: TlantiToothState['restorationType'];
        material: DentalMaterialType;
        shade: string;
        productionMethod: string;
        implantOption: ImplantOption;
        preOpModel: boolean;
        extraGingivaScan: boolean;
        substructureScan: boolean;
        waxup: boolean;
        minimalThicknessMm: number;
        cementGapMm: number;
        workTimeMinutes: number;
        biteSplintMode: TlantiToothState['biteSplintMode'];
        biteSplintAntagonistScan: TlantiToothState['biteSplintAntagonistScan'];
    }) => void;
}

interface LocalState {
    workTypeId: string;
    material: DentalMaterialType;
    shade: string;
    productionMethod: string;
    implantOption: ImplantOption;
    preOpModel: boolean;
    extraGingivaScan: boolean;
    substructureScan: boolean;
    waxup: boolean;
    minimalThicknessMm: number;
    cementGapMm: number;
    workTimeMinutes: number;
    workTimeTouched: boolean;
    biteSplintMode: NonNullable<TlantiToothState['biteSplintMode']>;
    biteSplintAntagonistScan: NonNullable<TlantiToothState['biteSplintAntagonistScan']>;
    showAdvanced: boolean;
}

function initialLocalState(toothState?: TlantiToothState): LocalState {
    const seedWorkTypeId = toothState?.workTypeId ?? 'anatomic-crown';
    const seedDefaultMinutes = resolveWorkType(seedWorkTypeId)?.defaultMinutes ?? 30;
    return {
        workTypeId: seedWorkTypeId,
        material: (toothState?.material as DentalMaterialType | undefined) ?? 'zirconia',
        shade: toothState?.shade ?? 'A2',
        productionMethod: '5-axis-laser-3dprint',
        implantOption: 'none',
        // V174 — read from structured `additionalScans` first; fall back to legacy
        // booleans `usePreOpModel` / `useExtraGingivaScan` for older case files.
        preOpModel:
            toothState?.additionalScans?.preOpModel ?? toothState?.usePreOpModel ?? false,
        extraGingivaScan:
            toothState?.additionalScans?.extraGingiva ??
            toothState?.useExtraGingivaScan ??
            false,
        substructureScan: toothState?.additionalScans?.substructureScan ?? false,
        waxup:
            toothState?.additionalScans?.waxup ??
            (toothState?.workTypeId === 'waxup' ||
                toothState?.workTypeId === 'anatomic-waxup' ||
                toothState?.workTypeId === 'reduced-waxup' ||
                toothState?.workTypeId === 'pontic-waxup'),
        minimalThicknessMm: toothState?.minimalThicknessMm ?? 0.5,
        cementGapMm: toothState?.cementGapMm ?? 0.1,
        workTimeMinutes: toothState?.workTimeMinutes ?? seedDefaultMinutes,
        workTimeTouched: toothState?.workTimeMinutes != null,
        biteSplintMode: toothState?.biteSplintMode ?? 'standard',
        biteSplintAntagonistScan: toothState?.biteSplintAntagonistScan ?? 'registered-jaw',
        showAdvanced: false,
    };
}

export function ToothWorkDefinitionDialog({
    open,
    toothNumber,
    numberingSystem,
    toothState,
    onCancel,
    onClear,
    onConfirm,
}: ToothWorkDefinitionDialogProps) {
    const [state, setState] = useState<LocalState>(() => initialLocalState(toothState));
    const [activeCategory, setActiveCategory] = useState<string>('crowns-copings');

    React.useEffect(() => {
        if (open) setState(initialLocalState(toothState));
    }, [open, toothState]);

    const numberingLegend = useMemo(
        () => (toothNumber ? getToothNumberingLegend(toothNumber, numberingSystem) : ''),
        [numberingSystem, toothNumber],
    );

    const selectedWorkType = useMemo(
        () => resolveWorkType(state.workTypeId),
        [state.workTypeId],
    );

    const availableMaterials = useMemo(
        () =>
            DENTAL_MATERIALS.filter((m) =>
                m.productionTags.includes(state.productionMethod as (typeof m.productionTags)[number]),
            ),
        [state.productionMethod],
    );

    if (!open || !toothNumber) return null;

    const handleConfirm = () => {
        const legacy = selectedWorkType?.legacyType ?? 'anatomic-crown';
        const isBiteSplint = state.workTypeId === 'bite-splint';
        onConfirm(toothNumber, {
            workTypeId: state.workTypeId,
            legacyRestorationType: legacy,
            material: state.material,
            shade: state.shade,
            productionMethod: state.productionMethod,
            implantOption: state.implantOption,
            preOpModel: state.preOpModel,
            extraGingivaScan: state.extraGingivaScan,
            substructureScan: state.substructureScan,
            waxup: state.waxup,
            minimalThicknessMm: state.minimalThicknessMm,
            cementGapMm: state.cementGapMm,
            workTimeMinutes: state.workTimeMinutes,
            biteSplintMode: isBiteSplint ? state.biteSplintMode : undefined,
            biteSplintAntagonistScan: isBiteSplint ? state.biteSplintAntagonistScan : undefined,
        });
    };

    const handleWorkTypeSelect = (nextWorkTypeId: string) => {
        setState((s) => {
            const nextDefaultMinutes = resolveWorkType(nextWorkTypeId)?.defaultMinutes ?? s.workTimeMinutes;
            return {
                ...s,
                workTypeId: nextWorkTypeId,
                workTimeMinutes: s.workTimeTouched ? s.workTimeMinutes : nextDefaultMinutes,
            };
        });
    };

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="tooth-work-dialog-title"
            className="fixed inset-0 z-[130] flex items-center justify-center bg-black/65 p-4 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onCancel();
            }}
        >
            <div className="relative flex h-[min(44rem,92vh)] w-[min(82rem,98vw)] flex-col overflow-hidden rounded-2xl border border-border bg-surface-raised shadow-2xl">
                <header className="flex items-center gap-3 border-b border-border px-5 py-3">
                    <button
                        type="button"
                        aria-label="Cancel and close"
                        onClick={onCancel}
                        className="rounded-md border border-border bg-surface-sunken p-1.5 text-text-secondary hover:text-text-primary"
                    >
                        <ArrowLeft className="size-4" />
                    </button>
                    <div className="min-w-0">
                        <h2 id="tooth-work-dialog-title" className="text-xl font-semibold text-text-display">
                            Tooth {toothNumber}
                        </h2>
                        <p className="text-xs text-text-secondary">
                            {numberingLegend || 'Individual tooth job'} ·{' '}
                            <span className="italic">Material configuration (local): Default</span>
                        </p>
                    </div>
                    <div className="ml-auto flex items-center gap-1 text-text-secondary">
                        <button
                            type="button"
                            className="rounded p-1 hover:bg-surface-sunken"
                            aria-label="Help"
                        >
                            <HelpCircle className="size-4" />
                        </button>
                    </div>
                </header>

                <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,1.4fr)] gap-0 divide-x divide-border overflow-hidden">
                    {/* Column 1 — WORK TYPE */}
                    <section className="flex min-h-0 flex-col">
                        <div className="flex items-center justify-between px-5 pt-4">
                            <h3 className="text-sm font-semibold text-text-primary">Work type</h3>
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary">
                                ① Job definition
                            </span>
                        </div>
                        <div className="flex gap-1 overflow-x-auto px-5 pt-2 text-[11px]">
                            {DENTAL_WORK_CATEGORIES.map((cat) => (
                                <button
                                    key={cat.id}
                                    type="button"
                                    onClick={() => setActiveCategory(cat.id)}
                                    className={[
                                        'shrink-0 rounded-md border px-2 py-1 transition',
                                        activeCategory === cat.id
                                            ? 'border-sky-400 bg-sky-500/15 text-text-primary'
                                            : 'border-border bg-surface-sunken text-text-secondary hover:bg-surface-raised',
                                    ].join(' ')}
                                >
                                    {cat.label}
                                </button>
                            ))}
                        </div>
                        <div className="mt-2 flex-1 overflow-y-auto px-5 pb-4">
                            <div className="grid grid-cols-2 gap-2">
                                {DENTAL_WORK_TYPES.filter((w) => w.category === activeCategory).map(
                                    (w) => (
                                        <WorkTypeButton
                                            key={w.id}
                                            workType={w}
                                            selected={state.workTypeId === w.id}
                                            onClick={() => handleWorkTypeSelect(w.id)}
                                        />
                                    ),
                                )}
                            </div>
                        </div>
                    </section>

                    {/* Column 2 — MATERIAL */}
                    <section className="flex min-h-0 flex-col bg-surface/40">
                        <div className="flex items-center justify-between px-5 pt-4">
                            <h3 className="text-sm font-semibold text-text-primary">Material</h3>
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary">
                                ② Material
                            </span>
                        </div>
                        <div className="px-5 pt-2">
                            <select
                                className="w-full rounded-md border border-border bg-surface-sunken px-2 py-1.5 text-xs text-text-primary"
                                value={state.productionMethod}
                                onChange={(event) => {
                                    const productionMethod = event.currentTarget.value;
                                    setState((s) => ({ ...s, productionMethod }));
                                }}
                            >
                                {PRODUCTION_METHODS.map((p) => (
                                    <option key={p.id} value={p.id}>
                                        {p.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                        <div className="mt-2 flex-1 overflow-y-auto px-5 pb-4">
                            <div className="grid grid-cols-2 gap-2">
                                {availableMaterials.map((m) => (
                                    <MaterialTile
                                        key={m.id}
                                        material={m}
                                        selected={state.material === (m.id as DentalMaterialType)}
                                        onClick={() =>
                                            setState((s) => ({
                                                ...s,
                                                material: m.id as DentalMaterialType,
                                            }))
                                        }
                                    />
                                ))}
                            </div>
                        </div>
                    </section>

                    {/* Column 3 — OPTIONS & PARAMETERS */}
                    <section className="flex min-h-0 flex-col bg-surface/60">
                        <div className="flex items-center justify-between px-5 pt-4">
                            <h3 className="text-sm font-semibold text-text-primary">
                                Options &amp; Parameters
                            </h3>
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary">
                                ③ Configure
                            </span>
                        </div>
                        <div className="flex-1 overflow-y-auto px-5 pb-4 pt-2">
                            <FieldSection label="Material shade?">
                                <div className="flex flex-wrap gap-1.5">
                                    {MATERIAL_SHADES.map((shade) => (
                                        <button
                                            key={shade}
                                            type="button"
                                            onClick={() => setState((s) => ({ ...s, shade }))}
                                            className={[
                                                'rounded border px-2 py-1 text-xs',
                                                state.shade === shade
                                                    ? 'border-sky-400 bg-sky-500/15 text-text-primary'
                                                    : 'border-border bg-surface-sunken text-text-secondary hover:bg-surface-raised',
                                            ].join(' ')}
                                        >
                                            {shade}
                                        </button>
                                    ))}
                                </div>
                            </FieldSection>

                            <FieldSection label="Implant-based?">
                                <div className="grid grid-cols-2 gap-1.5">
                                    {IMPLANT_OPTIONS.filter((o) => o.id !== 'none').map((opt) => (
                                        <ImplantTile
                                            key={opt.id}
                                            option={opt}
                                            selected={state.implantOption === opt.id}
                                            onClick={() =>
                                                setState((s) => ({
                                                    ...s,
                                                    implantOption:
                                                        s.implantOption === opt.id ? 'none' : opt.id,
                                                }))
                                            }
                                        />
                                    ))}
                                </div>
                            </FieldSection>

                            <FieldSection label="Additional scans">
                                <div className="flex flex-col gap-1.5">
                                    <ToggleRow
                                        label="Pre-op model"
                                        checked={state.preOpModel}
                                        onChange={(v) => setState((s) => ({ ...s, preOpModel: v }))}
                                    />
                                    <ToggleRow
                                        label="Extra gingiva scan"
                                        checked={state.extraGingivaScan}
                                        onChange={(v) =>
                                            setState((s) => ({ ...s, extraGingivaScan: v }))
                                        }
                                    />
                                    <ToggleRow
                                        label="On substructure scan"
                                        checked={state.substructureScan}
                                        onChange={(v) =>
                                            setState((s) => ({ ...s, substructureScan: v }))
                                        }
                                    />
                                    <ToggleRow
                                        label="Waxup (digital copy milling)"
                                        checked={state.waxup}
                                        onChange={(v) =>
                                            setState((s) => ({ ...s, waxup: v }))
                                        }
                                    />
                                </div>
                            </FieldSection>

                            {state.workTypeId === 'bite-splint' ? (
                                <>
                                    <FieldSection label="Bite splint mode">
                                        <div className="grid grid-cols-3 gap-1.5">
                                            <BiteSplintModeTile
                                                label="Standard"
                                                sublabel="A"
                                                color="#E07A79"
                                                selected={state.biteSplintMode === 'standard'}
                                                onClick={() =>
                                                    setState((s) => ({ ...s, biteSplintMode: 'standard' }))
                                                }
                                            />
                                            <BiteSplintModeTile
                                                label="Fill gap"
                                                sublabel="B"
                                                color="#3FA79C"
                                                selected={state.biteSplintMode === 'fill-gap'}
                                                onClick={() =>
                                                    setState((s) => ({ ...s, biteSplintMode: 'fill-gap' }))
                                                }
                                            />
                                            <BiteSplintModeTile
                                                label="Anatomical"
                                                sublabel="C"
                                                color="#3B7AB9"
                                                selected={state.biteSplintMode === 'anatomical'}
                                                onClick={() =>
                                                    setState((s) => ({ ...s, biteSplintMode: 'anatomical' }))
                                                }
                                            />
                                        </div>
                                    </FieldSection>

                                    <FieldSection label="Antagonist scan">
                                        <select
                                            className="w-full rounded-md border border-border bg-surface-sunken px-2 py-1.5 text-xs text-text-primary"
                                            value={state.biteSplintAntagonistScan}
                                            onChange={(event) => {
                                                const biteSplintAntagonistScan =
                                                    event.currentTarget.value as LocalState['biteSplintAntagonistScan'];
                                                setState((s) => ({
                                                    ...s,
                                                    biteSplintAntagonistScan,
                                                }));
                                            }}
                                        >
                                            <option value="bite-impression">Bite impression</option>
                                            <option value="registered-jaw">Registered jaw (no articulation)</option>
                                            <option value="type-a">Type A</option>
                                            <option value="type-s">Type S</option>
                                        </select>
                                    </FieldSection>
                                </>
                            ) : null}

                            <FieldSection label="Parameters">
                                <SliderRow
                                    label="Minimal thickness"
                                    unit="mm"
                                    min={0.2}
                                    max={2.0}
                                    step={0.05}
                                    value={state.minimalThicknessMm}
                                    onChange={(v) =>
                                        setState((s) => ({ ...s, minimalThicknessMm: v }))
                                    }
                                />
                                <SliderRow
                                    label="Gap width of cement"
                                    unit="mm"
                                    min={0.0}
                                    max={0.3}
                                    step={0.01}
                                    value={state.cementGapMm}
                                    onChange={(v) => setState((s) => ({ ...s, cementGapMm: v }))}
                                />
                            </FieldSection>

                            <FieldSection label="Lab work time">
                                <SliderRow
                                    label="Estimated work time"
                                    unit="min"
                                    min={0}
                                    max={240}
                                    step={5}
                                    value={state.workTimeMinutes}
                                    onChange={(v) =>
                                        setState((s) => ({
                                            ...s,
                                            workTimeMinutes: v,
                                            workTimeTouched: true,
                                        }))
                                    }
                                />
                                <p className="text-[10px] leading-snug text-text-secondary">
                                    Default from catalog:{' '}
                                    <span className="tabular-nums">
                                        {selectedWorkType?.defaultMinutes ?? 0} min
                                    </span>
                                    {state.workTimeTouched ? (
                                        <>
                                            {' '}·{' '}
                                            <button
                                                type="button"
                                                className="text-sky-300 hover:underline"
                                                onClick={() =>
                                                    setState((s) => ({
                                                        ...s,
                                                        workTimeMinutes:
                                                            resolveWorkType(s.workTypeId)
                                                                ?.defaultMinutes ?? s.workTimeMinutes,
                                                        workTimeTouched: false,
                                                    }))
                                                }
                                            >
                                                reset
                                            </button>
                                        </>
                                    ) : null}
                                </p>
                            </FieldSection>

                            <button
                                type="button"
                                onClick={() =>
                                    setState((s) => ({ ...s, showAdvanced: !s.showAdvanced }))
                                }
                                className="text-[11px] text-sky-300 hover:underline"
                            >
                                {state.showAdvanced ? 'Hide' : 'Show'} advanced parameters
                            </button>
                        </div>
                    </section>
                </div>

                <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-5 py-3">
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => onClear(toothNumber)}
                    >
                        <Eraser className="size-4" />
                        <span className="ml-2">Clear</span>
                    </Button>
                    <div className="ml-auto flex items-center gap-2">
                        <Button type="button" variant="secondary" size="sm" onClick={onCancel}>
                            Cancel
                        </Button>
                        <Button type="button" variant="default" size="sm" onClick={handleConfirm}>
                            <Check className="size-4" />
                            <span className="ml-2">OK</span>
                        </Button>
                    </div>
                </footer>
            </div>
        </div>
    );
}

function WorkTypeButton({
    workType,
    selected,
    onClick,
}: {
    workType: DentalWorkType;
    selected: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'relative overflow-hidden rounded-lg border px-3 py-2 text-left text-xs text-white transition',
                selected ? 'border-white shadow-[0_0_0_1px_rgba(255,255,255,0.6)]' : 'border-transparent',
            ].join(' ')}
            style={{ backgroundColor: workType.color }}
            title={workType.description}
        >
            <span className="line-clamp-2 pr-4 font-medium leading-tight">{workType.label}</span>
            {selected ? (
                <CheckCircle2 className="absolute right-1.5 top-1.5 size-3.5 text-white" />
            ) : null}
        </button>
    );
}

function MaterialTile({
    material,
    selected,
    onClick,
}: {
    material: (typeof DENTAL_MATERIALS)[number];
    selected: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'flex flex-col items-center justify-end gap-1 rounded-lg border p-2 text-[11px] transition',
                selected
                    ? 'border-amber-400 bg-amber-500/10 text-text-primary shadow-sm'
                    : 'border-border bg-surface-sunken text-text-secondary hover:bg-surface-raised',
            ].join(' ')}
            title={material.label}
        >
            <div
                className="flex aspect-square w-full items-center justify-center overflow-hidden rounded-md"
                style={{ backgroundColor: material.color }}
            >
                {material.previewUrl ? (
                    <img
                        src={material.previewUrl}
                        alt=""
                        className="h-full w-full object-contain opacity-90"
                        loading="lazy"
                    />
                ) : null}
            </div>
            <span className="text-center font-medium">{material.label}</span>
        </button>
    );
}

function BiteSplintModeTile({
    label,
    sublabel,
    color,
    selected,
    onClick,
}: {
    label: string;
    sublabel: string;
    color: string;
    selected: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'relative rounded-md border px-2 py-2 text-left text-white transition',
                selected ? 'border-white shadow-[0_0_0_1px_rgba(255,255,255,0.6)]' : 'border-transparent',
            ].join(' ')}
            style={{ backgroundColor: color }}
            title={label}
        >
            <div className="text-[10px] font-mono uppercase tracking-wider opacity-80">{sublabel}</div>
            <div className="text-xs font-semibold leading-tight">{label}</div>
            {selected ? <CheckCircle2 className="absolute right-1 top-1 size-3.5" /> : null}
        </button>
    );
}

function ImplantTile({
    option,
    selected,
    onClick,
}: {
    option: (typeof IMPLANT_OPTIONS)[number];
    selected: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'rounded-md border px-2 py-1.5 text-left text-[11px] transition',
                selected ? 'border-white text-white shadow-sm' : 'border-transparent text-white/90',
            ].join(' ')}
            style={{ backgroundColor: option.color }}
            title={option.description}
        >
            <span className="font-semibold">{option.label}</span>
        </button>
    );
}

function FieldSection({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="mb-4 flex flex-col gap-2">
            <p className="text-[10px] font-mono uppercase tracking-wider text-text-secondary">
                {label}
            </p>
            {children}
        </div>
    );
}

function ToggleRow({
    label,
    checked,
    onChange,
}: {
    label: string;
    checked: boolean;
    onChange: (v: boolean) => void;
}) {
    return (
        <label className="flex cursor-pointer items-center justify-between rounded-md border border-border bg-surface-sunken px-2 py-1.5 text-[11px] text-text-primary">
            <span>{label}</span>
            <button
                type="button"
                role="switch"
                aria-checked={checked}
                onClick={() => onChange(!checked)}
                className={[
                    'relative h-4 w-7 rounded-full border transition-colors',
                    checked ? 'border-sky-400 bg-sky-500/60' : 'border-border bg-surface-raised',
                ].join(' ')}
            >
                <span
                    className={[
                        'absolute top-[1px] h-[14px] w-[14px] rounded-full bg-white transition-transform',
                        checked ? 'translate-x-[12px]' : 'translate-x-[1px]',
                    ].join(' ')}
                />
            </button>
        </label>
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
}: {
    label: string;
    unit: string;
    min: number;
    max: number;
    step: number;
    value: number;
    onChange: (v: number) => void;
}) {
    const decimals = step >= 1 ? 0 : step >= 0.05 ? 2 : 2;
    return (
        <div className="mb-2 flex flex-col gap-1 rounded-md border border-border bg-surface-sunken px-2 py-1.5">
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
                onChange={(event) => {
                    const nextValue = Number.parseFloat(event.currentTarget.value);
                    onChange(Number.isFinite(nextValue) ? nextValue : value);
                }}
                className="h-1 w-full appearance-none rounded bg-border accent-sky-400"
            />
        </div>
    );
}
