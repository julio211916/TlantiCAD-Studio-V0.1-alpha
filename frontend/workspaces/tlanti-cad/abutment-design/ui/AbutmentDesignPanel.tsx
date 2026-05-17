/**
 * AbutmentDesignPanel — V143.
 *
 * Implements the exocad "Editing Abutment design" workflow with 3 tabs:
 *   • Top — style picker (Cylindrical/Angular/Standard/Legacy), shoulder
 *     size, roundness, minimum angle, fissure controls, spacing.
 *   • Bottom — emergence profile (delegated to Crown Bottoms in clinical
 *     practice; this tab links to the wizard step).
 *   • Advanced — profile border, angulated screw channel, milling parameters.
 *
 * Pure presentational. State + callbacks owned by parent.
 */

import React from 'react';

import { Icon } from '@tlanticad/ui';
import {
    ABUTMENT_STYLES,
    setAllControlPointsStick,
    toggleControlPointStick,
    validateScrewChannelAngle,
    type AbutmentBottomShape,
    type AbutmentDesign,
    type AbutmentStyle,
    type AbutmentTab,
} from '../domain/abutment-params';

export interface AbutmentDesignPanelProps {
    design: AbutmentDesign;
    onChange: (next: AbutmentDesign) => void;
    activeTab: AbutmentTab;
    onTabChange: (tab: AbutmentTab) => void;
    onApplyStyle: () => void;
    onAdjustInsertionDirection: () => void;
    onSaveCustomDesign: () => void;
    onResetToInitial: () => void;
    onResetCustomized: () => void;
    onApply: () => void;
    onBack?: () => void;
    onNext?: () => void;
    isApplyBusy?: boolean;
    statusMessage?: string | null;
    errorMessage?: string | null;
}

export function AbutmentDesignPanel(props: AbutmentDesignPanelProps) {
    const { design, onChange, activeTab, onTabChange } = props;
    const advancedAngleWarning = validateScrewChannelAngle(
        design.advanced.angulatedScrewChannelDeg,
    );

    return (
        <aside
            role="dialog"
            aria-labelledby="abutment-design-title"
            className="pointer-events-auto flex w-[24rem] flex-col gap-2 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="abutment-design-panel"
        >
            <header className="flex items-center gap-2">
                <Icon
                    name="abutment.abutment-cylindrical"
                    size={18}
                    className="text-orange-300"
                    aria-hidden
                />
                <h3 id="abutment-design-title" className="text-sm font-semibold">
                    Abutment Design
                </h3>
            </header>

            <nav className="grid grid-cols-3 gap-1 rounded border border-white/10 bg-violet-900/30 p-1 text-[10px] uppercase tracking-wider">
                {(['top', 'bottom', 'advanced'] as AbutmentTab[]).map((tab) => (
                    <button
                        key={tab}
                        type="button"
                        onClick={() => onTabChange(tab)}
                        className={[
                            'rounded-sm px-2 py-1 transition',
                            activeTab === tab
                                ? 'bg-orange-500/30 text-orange-200'
                                : 'text-slate-300 hover:bg-white/10',
                        ].join(' ')}
                    >
                        {tab}
                    </button>
                ))}
            </nav>

            {activeTab === 'top' && (
                <TopTab
                    design={design}
                    onChange={onChange}
                    onAdjustInsertionDirection={props.onAdjustInsertionDirection}
                    onSaveCustomDesign={props.onSaveCustomDesign}
                    onResetToInitial={props.onResetToInitial}
                    onResetCustomized={props.onResetCustomized}
                />
            )}
            {activeTab === 'bottom' && (
                <BottomTab design={design} onChange={onChange} />
            )}
            {activeTab === 'advanced' && (
                <AdvancedTab
                    design={design}
                    onChange={onChange}
                    angleWarning={advancedAngleWarning}
                />
            )}

            {props.errorMessage ? (
                <p className="rounded-md border border-red-400/40 bg-red-500/10 px-2 py-1 text-[10px] text-red-100">
                    {props.errorMessage}
                </p>
            ) : null}
            {props.statusMessage ? (
                <p className="rounded-md border border-emerald-400/30 bg-emerald-500/10 px-2 py-1 text-[10px] text-emerald-100">
                    {props.statusMessage}
                </p>
            ) : null}

            <footer className="mt-2 flex items-center gap-2 border-t border-white/10 pt-2">
                <button
                    type="button"
                    className="rounded-md border border-white/15 px-3 py-1.5 text-xs"
                    onClick={props.onBack}
                    disabled={props.isApplyBusy}
                >
                    ← Back
                </button>
                <button
                    type="button"
                    className="rounded-md border border-white/15 px-3 py-1.5 text-xs"
                    onClick={props.onApply}
                    disabled={props.isApplyBusy}
                >
                    {props.isApplyBusy ? 'Generating…' : 'Apply'}
                </button>
                <button
                    type="button"
                    className="ml-auto rounded-md bg-sky-500 px-3 py-1.5 text-xs font-semibold text-white"
                    onClick={props.onNext}
                    disabled={props.isApplyBusy}
                >
                    Next →
                </button>
            </footer>
        </aside>
    );
}

function TopTab(props: {
    design: AbutmentDesign;
    onChange: (next: AbutmentDesign) => void;
    onAdjustInsertionDirection: () => void;
    onSaveCustomDesign: () => void;
    onResetToInitial: () => void;
    onResetCustomized: () => void;
}) {
    const { design, onChange } = props;
    const setStyle = (style: AbutmentStyle) => {
        const profile = ABUTMENT_STYLES.find((s) => s.id === style)!;
        onChange({
            ...design,
            top: {
                ...design.top,
                style,
                shoulderSizeMm: design.top.keepConstantShoulderSize
                    ? design.top.shoulderSizeMm
                    : profile.shoulderSizeMm,
                roundness: profile.roundness,
                minimumAngleDeg: profile.minimumAngleDeg,
            },
        });
    };
    const setTop = <K extends keyof AbutmentDesign['top']>(
        key: K,
        value: AbutmentDesign['top'][K],
    ) => onChange({ ...design, top: { ...design.top, [key]: value } });

    return (
        <div className="flex flex-col gap-2">
            <p className="text-[10px] uppercase tracking-wider text-slate-300">Abutment Style</p>
            <div className="grid grid-cols-2 gap-1.5">
                {ABUTMENT_STYLES.map((s) => (
                    <button
                        key={s.id}
                        type="button"
                        onClick={() => setStyle(s.id)}
                        title={s.description}
                        className={[
                            'flex flex-col items-start rounded-md border px-2 py-1.5 text-left text-[11px] transition',
                            design.top.style === s.id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/15 text-slate-300 hover:bg-white/10',
                        ].join(' ')}
                    >
                        <span className="font-semibold">{s.label}</span>
                        <span className="text-[10px] text-slate-400">{s.description}</span>
                    </button>
                ))}
            </div>

            <SliderRow
                label="Margin / shoulder size"
                unit="mm"
                min={0.1}
                max={1.2}
                step={0.05}
                value={design.top.shoulderSizeMm}
                onChange={(v) => setTop('shoulderSizeMm', v)}
            />
            <CheckRow
                label="Keep constant shoulder size"
                checked={design.top.keepConstantShoulderSize}
                onChange={(v) => setTop('keepConstantShoulderSize', v)}
            />
            <SliderRow
                label="Roundness / Angularity"
                unit=""
                min={0}
                max={1}
                step={0.05}
                value={design.top.roundness}
                onChange={(v) => setTop('roundness', v)}
            />
            <SliderRow
                label="Minimum angle"
                unit="°"
                min={0}
                max={20}
                step={0.5}
                value={design.top.minimumAngleDeg}
                onChange={(v) => setTop('minimumAngleDeg', v)}
            />

            <CheckRow
                label="Connect fissure controls"
                checked={design.top.connectFissureControls}
                onChange={(v) => setTop('connectFissureControls', v)}
            />
            <CheckRow
                label="Keep design within anatomy"
                checked={design.top.keepDesignWithinAnatomy}
                onChange={(v) => setTop('keepDesignWithinAnatomy', v)}
            />

            <SliderRow
                label="Spacing (distance to occlusal)"
                unit="mm"
                min={0}
                max={2}
                step={0.05}
                value={design.top.spacingMm}
                onChange={(v) => setTop('spacingMm', v)}
            />
            <CheckRow
                label="Auto adapt"
                checked={design.top.autoAdaptSpacing}
                onChange={(v) => setTop('autoAdaptSpacing', v)}
            />

            <CheckRow
                label="Distance to anatomy (on abutment)"
                checked={design.top.showDistanceOnAbutment}
                onChange={(v) => setTop('showDistanceOnAbutment', v)}
            />
            <CheckRow
                label="Distance to abutment (on anatomy)"
                checked={design.top.showDistanceOnAnatomy}
                onChange={(v) => setTop('showDistanceOnAnatomy', v)}
            />

            <div className="flex flex-wrap gap-1 pt-1">
                <ActionPill onClick={props.onAdjustInsertionDirection}>Adjust insertion direction</ActionPill>
                <ActionPill onClick={props.onResetToInitial}>Restore initial shape</ActionPill>
                <ActionPill onClick={props.onResetCustomized}>Reset customized</ActionPill>
                <ActionPill onClick={props.onSaveCustomDesign}>Save custom design…</ActionPill>
            </div>
        </div>
    );
}

function BottomTab({
    design,
    onChange,
}: {
    design: AbutmentDesign;
    onChange: (next: AbutmentDesign) => void;
}) {
    const setBottom = <K extends keyof AbutmentDesign['bottom']>(
        key: K,
        value: AbutmentDesign['bottom'][K],
    ) => onChange({ ...design, bottom: { ...design.bottom, [key]: value } });

    const shapes: Array<{ id: AbutmentBottomShape; label: string; svg: React.ReactNode }> = [
        {
            id: 'concave-dished',
            label: 'Concave',
            svg: (
                <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="1.4">
                    <path d="M4 18 q4 -10 8 0 q4 -10 8 0" />
                </svg>
            ),
        },
        {
            id: 'standard',
            label: 'Standard',
            svg: (
                <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="1.4">
                    <path d="M4 18 l4 -10 l4 0 l4 10" />
                </svg>
            ),
        },
        {
            id: 'convex',
            label: 'Convex',
            svg: (
                <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="1.4">
                    <path d="M4 18 q4 4 8 -6 q4 4 8 6" />
                </svg>
            ),
        },
    ];

    return (
        <div className="flex flex-col gap-2">
            <p className="text-[11px] leading-snug text-slate-300">
                Emergence profile (sub-gingival). Choose a shape preset, fine-tune
                with the upper/lower sliders, or switch to free-form to sculpt
                individual abutments.
            </p>

            <div className="flex flex-col gap-1">
                <span className="text-[10px] uppercase tracking-wider text-slate-400">
                    Shape
                </span>
                <div className="grid grid-cols-3 gap-1">
                    {shapes.map((s) => (
                        <button
                            key={s.id}
                            type="button"
                            onClick={() => setBottom('shape', s.id)}
                            className={[
                                'flex flex-col items-center gap-0.5 rounded-md border px-2 py-1.5 text-[10px]',
                                design.bottom.shape === s.id
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/15 text-slate-300 hover:bg-white/10',
                            ].join(' ')}
                        >
                            {s.svg}
                            <span>{s.label}</span>
                        </button>
                    ))}
                </div>
            </div>

            <SliderRow
                label="Upper shape"
                unit="mm"
                min={0}
                max={2}
                step={0.05}
                value={design.bottom.upperShapeMm}
                onChange={(v) => setBottom('upperShapeMm', v)}
            />
            <SliderRow
                label="Lower shape"
                unit="mm"
                min={0}
                max={2}
                step={0.05}
                value={design.bottom.lowerShapeMm}
                onChange={(v) => setBottom('lowerShapeMm', v)}
            />

            <label className="flex items-center gap-2 rounded-md border border-white/10 px-2 py-1.5 text-[11px]">
                <input
                    type="checkbox"
                    className="accent-sky-400"
                    checked={design.bottom.freeFormBottom}
                    onChange={(e) => setBottom('freeFormBottom', e.currentTarget.checked)}
                />
                <span>Free-form bottom</span>
                <span className="ml-auto text-[10px] text-slate-400">
                    Click adds material · Shift+Click removes · Ctrl+Click adds all-around
                </span>
            </label>

            <div className="rounded-md border border-white/10 px-2 py-1.5 text-[11px]">
                <p className="text-[10px] uppercase tracking-wider text-slate-400">
                    Visualize gingiva
                </p>
                <div className="mt-1 flex flex-col gap-1">
                    <label className="flex items-center gap-2">
                        <input
                            type="checkbox"
                            className="accent-sky-400"
                            checked={design.bottom.visualizeDistance}
                            onChange={(e) => setBottom('visualizeDistance', e.currentTarget.checked)}
                        />
                        <span>Distance limit (blue)</span>
                        <input
                            type="number"
                            step="0.05"
                            min={0}
                            max={2}
                            value={design.bottom.distanceLimitMm}
                            onChange={(e) => setBottom('distanceLimitMm', parseFloat(e.currentTarget.value) || 0)}
                            className="ml-auto w-16 rounded border border-white/15 bg-violet-900/30 px-1 py-0.5 text-right font-mono text-[10px]"
                        />
                        <span className="text-[10px] text-slate-400">mm</span>
                    </label>
                    <label className="flex items-center gap-2">
                        <input
                            type="checkbox"
                            className="accent-sky-400"
                            checked={design.bottom.visualizeIntersection}
                            onChange={(e) => setBottom('visualizeIntersection', e.currentTarget.checked)}
                        />
                        <span>Intersection limit (red)</span>
                        <input
                            type="number"
                            step="0.05"
                            min={0}
                            max={2}
                            value={design.bottom.intersectionLimitMm}
                            onChange={(e) => setBottom('intersectionLimitMm', parseFloat(e.currentTarget.value) || 0)}
                            className="ml-auto w-16 rounded border border-white/15 bg-violet-900/30 px-1 py-0.5 text-right font-mono text-[10px]"
                        />
                        <span className="text-[10px] text-slate-400">mm</span>
                    </label>
                </div>
            </div>

            <details className="rounded-md border border-white/10 px-2 py-1.5 text-[11px]">
                <summary className="cursor-pointer text-[10px] uppercase tracking-wider text-slate-300">
                    Standard emergence (legacy fields)
                </summary>
                <div className="mt-2 flex flex-col gap-1.5">
                    <SliderRow
                        label="Emergence height"
                        unit="mm"
                        min={0.2}
                        max={2}
                        step={0.05}
                        value={design.bottom.emergenceHeightMm}
                        onChange={(v) => setBottom('emergenceHeightMm', v)}
                    />
                    <SliderRow
                        label="Emergence angle"
                        unit="°"
                        min={0}
                        max={60}
                        step={1}
                        value={design.bottom.emergenceAngleDeg}
                        onChange={(v) => setBottom('emergenceAngleDeg', v)}
                    />
                    <SliderRow
                        label="Contact pressure"
                        unit="mm"
                        min={-0.2}
                        max={0.5}
                        step={0.01}
                        value={design.bottom.contactPressureMm}
                        onChange={(v) => setBottom('contactPressureMm', v)}
                    />
                </div>
            </details>

            <ControlPointsRow design={design} onChange={onChange} />
        </div>
    );
}

function AdvancedTab({
    design,
    onChange,
    angleWarning,
}: {
    design: AbutmentDesign;
    onChange: (next: AbutmentDesign) => void;
    angleWarning: string | null;
}) {
    const setAdv = <K extends keyof AbutmentDesign['advanced']>(
        key: K,
        value: AbutmentDesign['advanced'][K],
    ) => onChange({ ...design, advanced: { ...design.advanced, [key]: value } });
    return (
        <div className="flex flex-col gap-2">
            <p className="text-[10px] uppercase tracking-wider text-slate-300">Profile border</p>
            <SliderRow
                label="Height"
                unit="mm"
                min={0.1}
                max={1}
                step={0.05}
                value={design.advanced.profileBorderHeightMm}
                onChange={(v) => setAdv('profileBorderHeightMm', v)}
            />
            <SliderRow
                label="Radius"
                unit="mm"
                min={0}
                max={0.5}
                step={0.01}
                value={design.advanced.profileBorderRadiusMm}
                onChange={(v) => setAdv('profileBorderRadiusMm', v)}
            />

            <p className="mt-2 flex items-center gap-1 text-[10px] uppercase tracking-wider text-slate-300">
                <Icon
                    name="abutment.screw-channel-asc"
                    size={12}
                    aria-label="Angulated screw channel"
                />
                Angulated screw channel
            </p>
            <div className="flex gap-1">
                {(
                    [
                        ['straight', 'Straight'],
                        ['angulated-clickable', 'Click anatomy'],
                        ['angulated-draggable', 'Drag arrow'],
                    ] as const
                ).map(([id, label]) => (
                    <button
                        key={id}
                        type="button"
                        onClick={() => setAdv('screwChannelMode', id)}
                        className={[
                            'flex-1 rounded-md border px-2 py-1.5 text-[10px] capitalize transition',
                            design.advanced.screwChannelMode === id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/15 text-slate-300 hover:bg-white/10',
                        ].join(' ')}
                    >
                        {label}
                    </button>
                ))}
            </div>
            <SliderRow
                label="Channel angle"
                unit="°"
                min={0}
                max={25}
                step={0.5}
                value={design.advanced.angulatedScrewChannelDeg}
                onChange={(v) => setAdv('angulatedScrewChannelDeg', v)}
            />
            {angleWarning ? (
                <p className="rounded border border-amber-500/40 bg-amber-500/10 px-2 py-1 text-[10px] text-amber-200">
                    {angleWarning}
                </p>
            ) : null}

            <p className="mt-2 text-[10px] uppercase tracking-wider text-slate-300">
                Milling parameters
            </p>
            <SliderRow
                label="Min thickness"
                unit="mm"
                min={0.2}
                max={1.2}
                step={0.05}
                value={design.advanced.minThicknessMm}
                onChange={(v) => setAdv('minThicknessMm', v)}
            />
            <SliderRow
                label="Abutment tool ⌀"
                unit="mm"
                min={0.5}
                max={2}
                step={0.05}
                value={design.advanced.abutmentToolDiameterMm}
                onChange={(v) => setAdv('abutmentToolDiameterMm', v)}
            />
            <SliderRow
                label="Suprastructure tool ⌀"
                unit="mm"
                min={0.5}
                max={2}
                step={0.05}
                value={design.advanced.superstructureToolDiameterMm}
                onChange={(v) => setAdv('superstructureToolDiameterMm', v)}
            />
            <SliderRow
                label="Screw distance"
                unit="mm"
                min={0.2}
                max={2}
                step={0.05}
                value={design.advanced.screwChannelDistanceMm}
                onChange={(v) => setAdv('screwChannelDistanceMm', v)}
            />
            <SliderRow
                label="Margin angle"
                unit="°"
                min={45}
                max={90}
                step={1}
                value={design.advanced.marginAngleDeg}
                onChange={(v) => setAdv('marginAngleDeg', v)}
            />
        </div>
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

function CheckRow({
    label,
    checked,
    onChange,
}: {
    label: string;
    checked: boolean;
    onChange: (next: boolean) => void;
}) {
    return (
        <label className="flex items-center gap-2 text-[11px]">
            <input
                type="checkbox"
                className="accent-sky-400"
                checked={checked}
                onChange={(e) => onChange(e.currentTarget.checked)}
            />
            <span>{label}</span>
        </label>
    );
}

function ActionPill({
    children,
    onClick,
}: {
    children: React.ReactNode;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="rounded-md border border-white/15 px-2 py-1 text-[10px] text-slate-200 hover:bg-white/10"
        >
            {children}
        </button>
    );
}

/**
 * V213 — list of emergence margin control points with pink/green toggle disks.
 * Renders a fixed grid of 12 indices around the abutment circumference. The
 * actual 3D widgets land in V219; this gives the lab tech a deterministic 2D
 * proxy in the meantime so the stick/free state can be authored.
 */
function ControlPointsRow({
    design,
    onChange,
}: {
    design: AbutmentDesign;
    onChange: (next: AbutmentDesign) => void;
}) {
    const slots = 12;
    const points = design.bottom.controlPoints;
    const lookup = new Map(points.map((p) => [p.index, p]));

    const toggle = (index: number, all = false) => {
        const next = toggleControlPointStick(points, index, { all });
        onChange({ ...design, bottom: { ...design.bottom, controlPoints: next } });
    };

    const stickAll = (stick: 'stick' | 'free') => {
        const next = setAllControlPointsStick(points, stick);
        onChange({ ...design, bottom: { ...design.bottom, controlPoints: next } });
    };

    return (
        <div className="rounded-md border border-white/10 px-2 py-1.5 text-[11px]">
            <div className="mb-1 flex items-center gap-2">
                <p className="text-[10px] uppercase tracking-wider text-slate-400">
                    Emergence control points
                </p>
                <button
                    type="button"
                    onClick={() => stickAll('stick')}
                    className="ml-auto rounded border border-pink-400/40 bg-pink-500/10 px-1.5 py-0.5 text-[10px] text-pink-200 hover:bg-pink-500/20"
                    title="Restick all"
                >
                    Restick all
                </button>
                <button
                    type="button"
                    onClick={() => stickAll('free')}
                    className="rounded border border-emerald-400/40 bg-emerald-500/10 px-1.5 py-0.5 text-[10px] text-emerald-200 hover:bg-emerald-500/20"
                    title="Unstick all"
                >
                    Unstick all
                </button>
            </div>
            <p className="mb-1 text-[10px] text-slate-400">
                Click toggles stick (pink) ↔ free (green). Ctrl+click toggles all.
            </p>
            <div className="grid grid-cols-12 gap-0.5">
                {Array.from({ length: slots }).map((_, i) => {
                    const point = lookup.get(i);
                    const stick = point?.stick ?? 'stick';
                    return (
                        <button
                            key={i}
                            type="button"
                            onClick={(e) => toggle(i, e.ctrlKey || e.metaKey)}
                            className={[
                                'h-5 w-full rounded-full border text-[8px] font-mono',
                                stick === 'stick'
                                    ? 'border-pink-300 bg-pink-500/40 text-pink-100'
                                    : 'border-emerald-300 bg-emerald-500/40 text-emerald-100',
                            ].join(' ')}
                            title={`Point ${i}: ${stick}`}
                        >
                            {i}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
