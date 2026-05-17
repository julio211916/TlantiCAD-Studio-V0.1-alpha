/**
 * PreopWaxupPanel — V173/V174 wizard UI.
 *
 * Two strategies share one panel because they appear at the same wizard
 * insertion point (after margin detection, before tooth placement). The
 * Waxup tab disables tooth-model placement and freeforming.
 */

import React from 'react';

import type {
    CopyStrategy,
    PreopWaxupState,
    WaxupPreparation,
} from '../domain/preop-waxup';
import { validateWaxupInput } from '../domain/preop-waxup';

export interface PreopWaxupPanelProps {
    state: PreopWaxupState;
    preopPath: string | null;
    waxupPath: string | null;
    onStrategyChange: (strategy: CopyStrategy) => void;
    onIterationsChange: (n: number) => void;
    onAlignPreop: () => void;
    onAdaptStart: () => void;
    onAdaptStop: () => void;
    onPrepareWaxup: () => void;
    onResetAlignment: () => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function PreopWaxupPanel(props: PreopWaxupPanelProps) {
    const { state } = props;
    const waxupErr = props.waxupPath
        ? validateWaxupInput({ waxupPath: props.waxupPath })
        : null;

    return (
        <aside
            role="dialog"
            aria-labelledby="preop-waxup-title"
            className="pointer-events-auto flex w-[24rem] flex-col gap-3 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="preop-waxup-panel"
        >
            <header>
                <h3 id="preop-waxup-title" className="text-sm font-semibold">
                    Pre-op scan & Waxup
                </h3>
                <p className="text-[10px] uppercase tracking-wider text-slate-300">
                    Copy predefined tooth shapes
                </p>
            </header>

            <nav className="grid grid-cols-2 gap-1 rounded border border-white/10 bg-violet-900/30 p-1 text-[11px] uppercase tracking-wider">
                <button
                    type="button"
                    onClick={() => props.onStrategyChange('preop')}
                    className={tabClass(state.activeStrategy === 'preop')}
                >
                    Pre-op scan
                </button>
                <button
                    type="button"
                    onClick={() => props.onStrategyChange('waxup')}
                    className={tabClass(state.activeStrategy === 'waxup')}
                >
                    Waxup
                </button>
            </nav>

            {state.activeStrategy === 'preop' ? (
                <PreopBody {...props} />
            ) : (
                <WaxupBody {...props} waxupError={waxupErr} />
            )}

            {state.error ? (
                <p className="rounded border border-rose-500/40 bg-rose-500/10 px-2 py-1 text-[11px] text-rose-200">
                    {state.error}
                </p>
            ) : null}

            <footer className="mt-auto flex items-center gap-2 border-t border-white/10 pt-2">
                <button
                    type="button"
                    className="rounded-md border border-white/15 px-3 py-1.5 text-xs"
                    onClick={props.onBack}
                >
                    ← Back
                </button>
                <button
                    type="button"
                    className="ml-auto rounded-md bg-sky-500 px-3 py-1.5 text-xs font-semibold text-white"
                    onClick={props.onNext}
                >
                    Next →
                </button>
            </footer>
        </aside>
    );
}

function PreopBody(props: PreopWaxupPanelProps) {
    const { state, preopPath } = props;
    return (
        <div className="flex flex-col gap-2">
            <PathRow label="Pre-op scan" value={preopPath} />
            <button
                type="button"
                onClick={props.onAlignPreop}
                disabled={!preopPath}
                className="rounded-md bg-sky-500 px-3 py-2 text-xs font-semibold text-white disabled:opacity-50"
            >
                Correct placement (align)
            </button>
            {state.alignment ? (
                <p className="rounded border border-emerald-500/40 bg-emerald-500/10 px-2 py-1 text-[11px] text-emerald-200">
                    Aligned · RMS {state.alignment.rmsMm.toFixed(3)} mm · backend: {state.alignment.backend}
                </p>
            ) : null}
            <SliderRow
                label="Adapt iterations"
                unit=""
                min={5}
                max={500}
                step={5}
                value={state.iterations}
                onChange={props.onIterationsChange}
            />
            <div className="flex gap-2">
                <button
                    type="button"
                    onClick={props.onAdaptStart}
                    disabled={!preopPath || state.isAdapting}
                    className="flex-1 rounded-md border border-white/15 bg-white/5 px-3 py-2 text-xs disabled:opacity-50"
                >
                    Adapt tooth models
                </button>
                <button
                    type="button"
                    onClick={props.onAdaptStop}
                    disabled={!state.isAdapting}
                    className="rounded-md border border-rose-500/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-200 disabled:opacity-50"
                >
                    Stop
                </button>
            </div>
            {state.lastAdapt ? (
                <p className="rounded border border-emerald-500/40 bg-emerald-500/10 px-2 py-1 text-[11px] text-emerald-200">
                    Adapted · {state.lastAdapt.iterationsRun} iters · RMS {state.lastAdapt.rmsMm.toFixed(3)} mm
                </p>
            ) : null}
            <p className="text-[10px] leading-snug text-slate-400">
                Connectors are <em>not</em> copied from the pre-op scan; design them
                normally in the next wizard steps.
            </p>
        </div>
    );
}

function WaxupBody(
    props: PreopWaxupPanelProps & { waxupError: string | null },
) {
    const { state, waxupPath, waxupError } = props;
    return (
        <div className="flex flex-col gap-2">
            <PathRow label="Waxup scan" value={waxupPath} />
            {waxupError ? (
                <p className="rounded border border-amber-500/40 bg-amber-500/10 px-2 py-1 text-[11px] text-amber-200">
                    {waxupError}
                </p>
            ) : null}
            <p className="text-[10px] leading-snug text-slate-300">
                <strong>Digital copy milling.</strong> No tooth models are loaded;
                the reconstruction is created directly from the wax scan.
                Connectors present in the wax are copied verbatim.
            </p>
            <button
                type="button"
                onClick={props.onPrepareWaxup}
                disabled={!waxupPath || waxupError !== null}
                className="rounded-md bg-sky-500 px-3 py-2 text-xs font-semibold text-white disabled:opacity-50"
            >
                Prepare waxup (close holes + crop)
            </button>
            {state.waxup ? <WaxupSummary waxup={state.waxup} /> : null}
            <p className="text-[10px] leading-snug text-amber-300">
                ⚠ Waxup-based restorations cannot guarantee SLM/3D-print
                requirements even with the optimization flag on. Verify watertightness
                before committing.
            </p>
        </div>
    );
}

function WaxupSummary({ waxup }: { waxup: WaxupPreparation }) {
    return (
        <div className="rounded border border-emerald-500/40 bg-emerald-500/10 px-2 py-2 text-[11px] text-emerald-200">
            <div className="font-semibold">Prepared</div>
            <div>Path · <code className="font-mono text-[10px]">{waxup.preparedPath}</code></div>
            <div>Holes closed · {waxup.holesClosed}</div>
            <div>Cropped above margin · {waxup.cropped ? 'yes' : 'no'}</div>
            <div>Backend · {waxup.backend}</div>
            {waxup.warnings.length > 0 ? (
                <ul className="mt-1 list-disc pl-4 text-amber-300">
                    {waxup.warnings.map((w) => (
                        <li key={w}>{w}</li>
                    ))}
                </ul>
            ) : null}
        </div>
    );
}

function PathRow({ label, value }: { label: string; value: string | null }) {
    return (
        <div className="rounded border border-white/10 bg-white/5 px-2 py-1.5 text-[11px]">
            <div className="text-[10px] uppercase tracking-wider text-slate-400">{label}</div>
            <div className="truncate font-mono text-[11px]">{value ?? '— no file selected —'}</div>
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
}: {
    label: string;
    unit: string;
    min: number;
    max: number;
    step: number;
    value: number;
    onChange: (n: number) => void;
}) {
    return (
        <label className="flex flex-col gap-1 text-[11px]">
            <span className="flex items-center justify-between">
                <span>{label}</span>
                <span className="font-mono tabular-nums text-slate-300">
                    {value} {unit}
                </span>
            </span>
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => onChange(parseInt(e.currentTarget.value, 10))}
                className="accent-sky-400"
            />
        </label>
    );
}

function tabClass(active: boolean): string {
    return [
        'rounded-sm px-2 py-1 transition',
        active ? 'bg-orange-500/30 text-orange-200' : 'text-slate-300 hover:bg-white/10',
    ].join(' ');
}
