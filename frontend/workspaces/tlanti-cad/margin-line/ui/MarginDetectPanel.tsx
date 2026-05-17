/**
 * MarginDetectPanel — Wizard step UI for preparation margin detection.
 *
 * Three mode buttons (Detect / Correct-Draw / Repair-Draw), supra/subgingival
 * switch, Adjust-light-from-view stub, and Clear margin. Matches the exocad
 * reference (docs image #17 & #29).
 */

import React, { useCallback } from 'react';

import { Button } from '@/components/ui/button';
import { AppIcon } from '../../app-icons';
import { polylinePerimeterMm } from '../domain/margin-line';

import type {
    MarginLine,
    MarginMode,
    MarginTool,
} from '../domain/margin-line';

interface MarginDetectPanelProps {
    toothFdi: number;
    meshPath: string | null;
    margin: MarginLine | null;
    tool: MarginTool;
    mode: MarginMode;
    isBusy: boolean;
    error: string | null;
    onChangeTool: (tool: MarginTool) => void;
    onChangeMode: (mode: MarginMode) => void;
    onRequestDetect: () => void;
    onAdjustLightFromView: () => void;
    onClear: () => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function MarginDetectPanel({
    toothFdi,
    meshPath,
    margin,
    tool,
    mode,
    isBusy,
    error,
    onChangeTool,
    onChangeMode,
    onRequestDetect,
    onAdjustLightFromView,
    onClear,
    onBack,
    onNext,
}: MarginDetectPanelProps) {
    const perimeter = margin ? polylinePerimeterMm(margin.polyline) : 0;
    const canNext = margin !== null && margin.polyline.length >= 12;
    const handleKey = useCallback(
        (e: React.KeyboardEvent<HTMLDivElement>) => {
            // Match exocad hotkeys 1/2/3/4 for tool switching.
            if (e.key === '1') onChangeTool('detect');
            if (e.key === '2') onChangeTool('correct-draw');
            if (e.key === '3') onChangeTool('correct-draw');
            if (e.key === '4') onChangeTool('correct-draw');
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'x') onClear();
        },
        [onChangeTool, onClear],
    );

    return (
        <aside
            role="dialog"
            aria-labelledby="margin-detect-title"
            tabIndex={0}
            onKeyDown={handleKey}
            className="pointer-events-auto flex w-[22rem] flex-col gap-3 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="margin-detect-panel"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="workflow.wizard-mode" size={18} aria-hidden />
                <h3
                    id="margin-detect-title"
                    className="text-sm font-semibold tracking-wide"
                >
                    Margin Line Detection
                </h3>
            </header>
            <p className="text-[11px] uppercase tracking-wider text-slate-300">
                Tooth {toothFdi || '—'}
            </p>

            <p className="rounded-md bg-violet-900/60 px-2.5 py-1.5 text-[11px] leading-snug text-slate-100">
                {tool === 'detect'
                    ? 'Click on margin line to start detection.'
                    : tool === 'correct-draw'
                      ? 'Drag points to edit and click to add a point.'
                      : 'Repair the margin and scan geometry to proceed.'}
            </p>

            <div className="grid grid-cols-3 gap-1.5">
                <ToolButton
                    icon="workflow.margin-detect"
                    label="Detect"
                    active={tool === 'detect'}
                    onClick={() => onChangeTool('detect')}
                />
                <ToolButton
                    icon="workflow.margin-correct-draw"
                    label="Correct / Draw"
                    active={tool === 'correct-draw'}
                    onClick={() => onChangeTool('correct-draw')}
                />
                <ToolButton
                    icon="workflow.margin-repair-draw"
                    label="Repair / Draw"
                    active={tool === 'repair-draw'}
                    onClick={() => onChangeTool('repair-draw')}
                />
            </div>

            <div className="grid grid-cols-2 gap-1.5 text-[11px]">
                <button
                    type="button"
                    onClick={() => onChangeMode('subgingival')}
                    className={[
                        'flex items-center justify-center gap-1.5 rounded-md border px-2 py-1.5',
                        mode === 'subgingival'
                            ? 'border-white bg-white/20'
                            : 'border-white/30 bg-transparent',
                    ].join(' ')}
                >
                    <AppIcon name="workflow.margin-subgingival" size={14} aria-hidden />
                    Subgingival
                </button>
                <button
                    type="button"
                    onClick={() => onChangeMode('supragingival')}
                    className={[
                        'flex items-center justify-center gap-1.5 rounded-md border px-2 py-1.5',
                        mode === 'supragingival'
                            ? 'border-orange-400 bg-orange-500/30'
                            : 'border-white/30 bg-transparent',
                    ].join(' ')}
                >
                    <AppIcon name="workflow.margin-supragingival" size={14} aria-hidden />
                    Supragingival
                </button>
            </div>

            <button
                type="button"
                onClick={onAdjustLightFromView}
                className="flex items-center justify-center gap-2 rounded-md border border-white/30 bg-transparent px-2 py-1.5 text-[11px]"
            >
                <AppIcon name="workflow.adjust-light" size={14} aria-hidden />
                Adjust light from view
            </button>

            {error ? (
                <p className="rounded-md border border-red-400/50 bg-red-500/20 px-2 py-1.5 text-[11px]">
                    {error}
                </p>
            ) : null}

            {margin ? (
                <dl className="grid grid-cols-2 gap-1.5 rounded-md bg-violet-900/40 px-2 py-1.5 text-[10px]">
                    <dt className="uppercase tracking-wider text-slate-300">Backend</dt>
                    <dd className="text-right font-mono">{margin.backend}</dd>
                    <dt className="uppercase tracking-wider text-slate-300">Points</dt>
                    <dd className="text-right tabular-nums">{margin.polyline.length}</dd>
                    <dt className="uppercase tracking-wider text-slate-300">Perimeter</dt>
                    <dd className="text-right tabular-nums">{perimeter.toFixed(2)} u</dd>
                    <dt className="uppercase tracking-wider text-slate-300">Confidence</dt>
                    <dd className="text-right tabular-nums">
                        {(margin.confidence * 100).toFixed(0)}%
                    </dd>
                </dl>
            ) : null}

            <div className="flex gap-1.5">
                <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    className="flex-1"
                    onClick={onRequestDetect}
                    disabled={isBusy || !meshPath}
                >
                    {isBusy ? 'Working…' : margin ? 'Re-run detect' : 'Detect'}
                </Button>
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={onClear}
                    disabled={!margin || isBusy}
                >
                    Clear
                </Button>
            </div>

            <footer className="mt-1 flex items-center justify-between">
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={onBack}
                    disabled={!onBack}
                >
                    ← Back
                </Button>
                <Button
                    type="button"
                    variant="default"
                    size="sm"
                    onClick={onNext}
                    disabled={!canNext}
                >
                    Next →
                </Button>
            </footer>
        </aside>
    );
}

function ToolButton({
    icon,
    label,
    active,
    onClick,
}: {
    icon: string;
    label: string;
    active: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'flex flex-col items-center gap-1 rounded-md border px-2 py-2 text-[10px] transition',
                active
                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                    : 'border-white/30 bg-transparent text-slate-200 hover:bg-white/10',
            ].join(' ')}
        >
            <AppIcon name={icon} size={18} aria-hidden />
            <span className="text-center leading-tight">{label}</span>
        </button>
    );
}
