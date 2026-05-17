/**
 * ShowDistancesPanel — V204.
 *
 * Floating tool panel: master toggle, color scale slider, target toggles
 * (antagonist / mesial / distal / healthy), interference-mode dropdown,
 * dynamic-occlusion checkboxes (when articulator simulation has run).
 */

import React from 'react';

import { CheckRow, SliderRow } from '../../../components/atoms';
import {
    colorbarStops,
    type DistanceMode,
    type DistanceVisualizationState,
    type DynamicChannel,
} from '../domain/distance-vis';

export interface ShowDistancesPanelProps {
    state: DistanceVisualizationState;
    onChange: (next: DistanceVisualizationState) => void;
    onCompute: () => void;
    onClose?: () => void;
    /** Disabled when no restoration mesh is available yet. */
    canCompute?: boolean;
}

const MODE_LABELS: Record<DistanceMode, string> = {
    'interference-contacts': 'Interference contacts',
    spacing: 'Spacing',
    'contact-areas': 'Contact areas',
    'distance-to-scan': 'Distance to scan data',
    thickness: 'Visualize thickness',
};

export function ShowDistancesPanel({
    state,
    onChange,
    onCompute,
    onClose,
    canCompute = true,
}: ShowDistancesPanelProps) {
    const stops = colorbarStops(state.colorScaleMm);
    const updateField = <K extends keyof DistanceVisualizationState>(
        key: K,
        value: DistanceVisualizationState[K],
    ) => onChange({ ...state, [key]: value });

    const toggleChannel = (ch: DynamicChannel) => {
        const next = new Set(state.dynamicChannels);
        if (next.has(ch)) next.delete(ch);
        else next.add(ch);
        onChange({ ...state, dynamicChannels: next });
    };

    return (
        <aside
            role="dialog"
            aria-labelledby="show-distances-title"
            className="pointer-events-auto flex w-[22rem] flex-col gap-3 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="show-distances-panel"
        >
            <header className="flex items-center gap-2">
                <h3 id="show-distances-title" className="text-sm font-semibold">
                    Show Distances
                </h3>
                <button
                    type="button"
                    onClick={() => updateField('enabled', !state.enabled)}
                    className={[
                        'ml-auto rounded-md border px-2 py-1 text-[11px]',
                        state.enabled
                            ? 'border-emerald-400/50 bg-emerald-500/15 text-emerald-200'
                            : 'border-white/15 text-slate-300 hover:bg-white/10',
                    ].join(' ')}
                >
                    {state.enabled ? 'Visible' : 'Hidden'}
                </button>
                {onClose ? (
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded border border-white/15 px-2 py-0.5 text-[11px] text-slate-300 hover:bg-white/10"
                    >
                        ✕
                    </button>
                ) : null}
            </header>

            <div className="flex flex-col gap-1 rounded border border-white/10 bg-violet-900/30 p-2">
                <p className="text-[10px] uppercase tracking-wider text-slate-300">
                    Color reference (mm)
                </p>
                <div
                    className="h-2.5 rounded"
                    style={{
                        background: `linear-gradient(90deg, ${stops
                            .map((s) => `${s.color} ${s.pct}%`)
                            .join(', ')})`,
                    }}
                />
                <div className="flex justify-between font-mono text-[10px] text-slate-300">
                    <span>{stops[0].mm.toFixed(2)}</span>
                    <span>0</span>
                    <span>+{stops[4].mm.toFixed(2)}</span>
                </div>
            </div>

            <SliderRow
                label="Color scale"
                unit="mm"
                min={0.05}
                max={2}
                step={0.05}
                value={state.colorScaleMm}
                onChange={(v) => updateField('colorScaleMm', v)}
                helpText="Half-range; ±value sets the red↔blue extremes."
            />

            <div className="flex flex-col gap-1 border-t border-white/10 pt-2">
                <p className="text-[10px] uppercase tracking-wider text-slate-300">Targets</p>
                <CheckRow
                    label="Antagonist"
                    checked={state.showAntagonist}
                    onChange={(v) => updateField('showAntagonist', v)}
                />
                <CheckRow
                    label="Mesial neighbor"
                    checked={state.showMesial}
                    onChange={(v) => updateField('showMesial', v)}
                />
                <CheckRow
                    label="Distal neighbor"
                    checked={state.showDistal}
                    onChange={(v) => updateField('showDistal', v)}
                />
                <CheckRow
                    label="Include healthy teeth"
                    checked={state.includeHealthy}
                    onChange={(v) => updateField('includeHealthy', v)}
                    helpText="Show contacts on scanned, non-restored teeth."
                />
            </div>

            <div className="flex flex-col gap-1 border-t border-white/10 pt-2">
                <label className="text-[10px] uppercase tracking-wider text-slate-300">
                    Mode
                </label>
                <select
                    value={state.mode}
                    onChange={(e) =>
                        updateField('mode', e.currentTarget.value as DistanceMode)
                    }
                    className="rounded border border-white/15 bg-violet-900/40 px-2 py-1 text-[11px]"
                >
                    {(Object.keys(MODE_LABELS) as DistanceMode[]).map((m) => (
                        <option key={m} value={m}>
                            {MODE_LABELS[m]}
                        </option>
                    ))}
                </select>
            </div>

            <div className="flex flex-col gap-1 border-t border-white/10 pt-2">
                <CheckRow
                    label="Dynamic (jaw movement)"
                    checked={state.dynamicEnabled}
                    onChange={(v) => updateField('dynamicEnabled', v)}
                    helpText="Requires articulator simulation to have run."
                />
                {state.dynamicEnabled ? (
                    <div className="ml-5 flex flex-wrap gap-1 text-[10px]">
                        {(
                            ['protrusive', 'laterotrusive-l', 'laterotrusive-r', 'retrusive'] as DynamicChannel[]
                        ).map((ch) => (
                            <button
                                key={ch}
                                type="button"
                                onClick={() => toggleChannel(ch)}
                                className={[
                                    'rounded border px-2 py-0.5 capitalize',
                                    state.dynamicChannels.has(ch)
                                        ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                        : 'border-white/15 text-slate-300 hover:bg-white/10',
                                ].join(' ')}
                            >
                                {ch.replace('-', ' ')}
                            </button>
                        ))}
                    </div>
                ) : null}
            </div>

            <button
                type="button"
                onClick={onCompute}
                disabled={!canCompute || state.isBusy}
                className="rounded-md bg-sky-500 px-3 py-2 text-xs font-semibold text-white disabled:opacity-50"
            >
                {state.isBusy ? 'Computing…' : 'Compute distances'}
            </button>

            {state.stats.length > 0 ? (
                <div className="rounded border border-white/10 bg-violet-900/30 p-2 text-[10px]">
                    <p className="mb-1 uppercase tracking-wider text-slate-300">Stats</p>
                    <table className="w-full font-mono">
                        <thead>
                            <tr className="text-slate-400">
                                <th className="text-left">Target</th>
                                <th className="text-right">Min</th>
                                <th className="text-right">Mean</th>
                                <th className="text-right">Max</th>
                                <th className="text-right">∩</th>
                            </tr>
                        </thead>
                        <tbody>
                            {state.stats.map((s) => (
                                <tr key={s.label} className="text-slate-100">
                                    <td className="capitalize">{s.label}</td>
                                    <td className="text-right">{s.minMm.toFixed(2)}</td>
                                    <td className="text-right">{s.meanMm.toFixed(2)}</td>
                                    <td className="text-right">{s.maxMm.toFixed(2)}</td>
                                    <td className="text-right">{s.intersectionCount}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            ) : null}

            {state.error ? (
                <p className="rounded border border-rose-500/40 bg-rose-500/10 px-2 py-1 text-[11px] text-rose-200">
                    {state.error}
                </p>
            ) : null}
        </aside>
    );
}
