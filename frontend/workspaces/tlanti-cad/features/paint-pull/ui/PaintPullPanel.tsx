/**
 * PaintPullPanel — V231 Advanced Editing UI.
 *
 * Region picker (moving / elastic / static) + brush size + Pull strength +
 * Reset / Invert / Pull buttons + per-region vertex count. Pure
 * presentational; the actual painting happens in the canvas overlay (handled
 * by the parent that wires raycast onto vertex indices).
 */

import React, { useMemo } from 'react';

import { CheckRow, SliderRow } from '../../../components/atoms';
import {
    REGION_COLORS,
    paintHistogram,
    type PaintPullState,
    type PaintRegion,
} from '../domain/paint-region';

export interface PaintPullPanelProps {
    state: PaintPullState;
    onChange: (next: PaintPullState) => void;
    onClear: () => void;
    onInvert: () => void;
    onTogglePull: () => void;
    onApply: () => void;
}

const REGION_ORDER: PaintRegion[] = ['moving', 'elastic', 'static'];

export function PaintPullPanel({
    state,
    onChange,
    onClear,
    onInvert,
    onTogglePull,
    onApply,
}: PaintPullPanelProps) {
    const histogram = useMemo(() => paintHistogram(state), [state]);

    return (
        <aside
            role="dialog"
            aria-labelledby="paint-pull-title"
            className="pointer-events-auto flex w-[20rem] flex-col gap-2 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="paint-pull-panel"
        >
            <header>
                <h3 id="paint-pull-title" className="text-sm font-semibold">
                    Paint &amp; Pull (Advanced)
                </h3>
                <p className="text-[10px] uppercase tracking-wider text-slate-300">
                    3-color region editing
                </p>
            </header>

            <div className="flex flex-col gap-1.5 rounded-md border border-white/10 bg-violet-900/30 p-2">
                <span className="text-[10px] uppercase tracking-wider text-slate-300">
                    Active brush
                </span>
                <div className="grid grid-cols-3 gap-1">
                    {REGION_ORDER.map((region) => (
                        <button
                            key={region}
                            type="button"
                            onClick={() => onChange({ ...state, activeRegion: region })}
                            className={[
                                'flex flex-col items-center gap-0.5 rounded-md border px-2 py-1.5 text-[11px] capitalize transition',
                                state.activeRegion === region
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/15 text-slate-300 hover:bg-white/10',
                            ].join(' ')}
                        >
                            <span
                                className="size-3 rounded-full"
                                style={{ backgroundColor: REGION_COLORS[region] }}
                                aria-hidden
                            />
                            {region}
                        </button>
                    ))}
                </div>
            </div>

            <SliderRow
                label="Brush size"
                unit="mm"
                min={0.3}
                max={6}
                step={0.1}
                value={state.brushSizeMm}
                onChange={(v) => onChange({ ...state, brushSizeMm: v })}
            />
            <SliderRow
                label="Pull strength"
                unit=""
                min={0}
                max={1}
                step={0.05}
                value={state.pullStrength}
                onChange={(v) => onChange({ ...state, pullStrength: v })}
                helpText="0 = no movement; 1 = full follow."
            />

            <CheckRow
                label="Pull moving parts"
                checked={state.isPulling}
                onChange={onTogglePull}
                helpText="When ON, drag on the canvas pulls the green region; elastic band falls off."
            />

            <div className="flex gap-1">
                <button
                    type="button"
                    onClick={onClear}
                    className="flex-1 rounded-md border border-white/15 px-2 py-1.5 text-[11px]"
                >
                    Reset
                </button>
                <button
                    type="button"
                    onClick={onInvert}
                    className="flex-1 rounded-md border border-white/15 px-2 py-1.5 text-[11px]"
                >
                    Invert
                </button>
                <button
                    type="button"
                    onClick={onApply}
                    className="flex-1 rounded-md bg-sky-500 px-2 py-1.5 text-[11px] font-semibold text-white"
                >
                    Apply
                </button>
            </div>

            <div className="rounded-md border border-white/10 bg-violet-900/30 p-2 text-[10px]">
                <p className="uppercase tracking-wider text-slate-300">Painted</p>
                <ul className="mt-1 grid grid-cols-2 gap-x-2 gap-y-0.5 font-mono">
                    {REGION_ORDER.map((region) => (
                        <li key={region} className="flex items-center gap-1">
                            <span
                                className="inline-block size-2 rounded"
                                style={{ backgroundColor: REGION_COLORS[region] }}
                            />
                            <span className="capitalize">{region}</span>
                            <span className="ml-auto text-slate-300">{histogram[region]}</span>
                        </li>
                    ))}
                    <li className="flex items-center gap-1 col-span-2 text-slate-400">
                        <span>Unassigned</span>
                        <span className="ml-auto">{histogram.unassigned}</span>
                    </li>
                </ul>
            </div>
        </aside>
    );
}
