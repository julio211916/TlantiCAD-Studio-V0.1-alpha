/**
 * FreeformingPanel — wizard step UI with 3 tabs (Free / Anatomic / Attachment).
 * Pure presentational; does not trigger mesh ops (that lives in the V55-V79 backend).
 */

import React from 'react';

import { Button } from '@/components/ui/button';
import { AppIcon } from '../../app-icons';
import { Icon } from '@tlanticad/ui';
import type {
    AnatomicPreset,
    AttachmentMode,
    FreeformBrushType,
    FreeformMode,
    FreeformState,
    FreeformTab,
    InsertionDirectionSource,
    MovementRestriction,
} from '../domain/freeform-brush';

interface FreeformingPanelProps {
    state: FreeformState;
    onChange: (next: FreeformState) => void;
    onCutAllIntersections: () => void;
    onApply: () => void;
    onUnload: () => void;
    onUndo: () => void;
    onRedo: () => void;
    onBack?: () => void;
    onNext?: () => void;
}

export function FreeformingPanel({
    state,
    onChange,
    onCutAllIntersections,
    onApply,
    onUnload,
    onUndo,
    onRedo,
    onBack,
    onNext,
}: FreeformingPanelProps) {
    const setTab = (tab: FreeformTab) => onChange({ ...state, tab });

    return (
        <aside
            role="dialog"
            aria-labelledby="freeforming-title"
            className="pointer-events-auto flex w-[22rem] flex-col gap-2 rounded-xl border border-border bg-violet-950/70 p-4 text-slate-50 shadow-xl backdrop-blur"
            data-visual-qa="freeforming-panel"
        >
            <header className="flex items-center gap-2">
                <AppIcon name="workflow.wizard-mode" size={18} aria-hidden />
                <h3 id="freeforming-title" className="text-sm font-semibold">
                    Free-Forming
                </h3>
                <Icon
                    name="freeform.paint-pull"
                    size={16}
                    className="ml-auto text-orange-300"
                    aria-label="Paint & Pull brush family"
                />
            </header>

            <div className="flex gap-1 text-[11px] font-semibold tracking-wider">
                {(['free', 'anatomic', 'attachment'] as const).map((id) => (
                    <button
                        key={id}
                        type="button"
                        onClick={() => setTab(id)}
                        className={[
                            'flex-1 rounded-md border px-2 py-1 uppercase',
                            state.tab === id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/20 text-slate-300',
                        ].join(' ')}
                    >
                        {id}
                    </button>
                ))}
            </div>

            {state.tab === 'free' ? <FreeTab state={state} onChange={onChange} /> : null}
            {state.tab === 'anatomic' ? <AnatomicTab state={state} onChange={onChange} /> : null}
            {state.tab === 'attachment' ? <AttachmentTab state={state} onChange={onChange} onApply={onApply} onUnload={onUnload} /> : null}

            <button
                type="button"
                onClick={onCutAllIntersections}
                className="mt-1 flex items-center justify-center gap-2 rounded-md border border-white/20 bg-violet-900/40 px-3 py-2 text-[11px] hover:bg-violet-900/70"
            >
                <AppIcon name="freeforming.cut-intersections" size={14} aria-hidden />
                Cut all intersections
            </button>

            <div className="flex items-center justify-between gap-2">
                <div className="flex gap-1">
                    <Button type="button" variant="ghost" size="sm" onClick={onUndo} disabled={state.undoDepth === 0}>
                        ↶ Undo
                    </Button>
                    <Button type="button" variant="ghost" size="sm" onClick={onRedo} disabled={state.redoDepth === 0}>
                        ↷ Redo
                    </Button>
                </div>
            </div>

            <footer className="mt-1 flex items-center justify-between">
                <Button type="button" variant="ghost" size="sm" onClick={onBack} disabled={!onBack}>
                    ← Back
                </Button>
                <Button type="button" variant="default" size="sm" onClick={onNext} disabled={!onNext}>
                    Next →
                </Button>
            </footer>
        </aside>
    );
}

function FreeTab({
    state,
    onChange,
}: {
    state: FreeformState;
    onChange: (next: FreeformState) => void;
}) {
    const setMode = (mode: FreeformMode) =>
        onChange({ ...state, brush: { ...state.brush, mode } });
    const setBrushType = (brushType: FreeformBrushType) =>
        onChange({ ...state, brush: { ...state.brush, brushType } });

    const modes: Array<{ id: FreeformMode; label: string; icon: string }> = [
        { id: 'add-remove', label: 'Add/ Remove', icon: 'workflow.cement-brush' },
        { id: 'smooth-flatten', label: 'Smooth/ Flatten', icon: 'workflow.free-form-brush' },
        { id: 'adapt', label: 'Adapt', icon: 'freeforming.adapt' },
    ];
    const brushTypes: Array<{ id: FreeformBrushType; icon: string }> = [
        { id: 'round-ball', icon: 'freeforming.brush-round' },
        { id: 'pointed-knife', icon: 'freeforming.brush-pointed' },
        { id: 'flat-cylinder', icon: 'freeforming.brush-cylinder' },
    ];

    return (
        <div className="flex flex-col gap-2">
            <div className="grid grid-cols-3 gap-1.5">
                {modes.map((m) => (
                    <button
                        key={m.id}
                        type="button"
                        onClick={() => setMode(m.id)}
                        className={[
                            'flex flex-col items-center gap-1 rounded-md border px-2 py-2 text-[10px]',
                            state.brush.mode === m.id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/20 text-slate-200',
                        ].join(' ')}
                    >
                        <AppIcon name={m.icon} size={18} aria-hidden />
                        <span className="text-center leading-tight">{m.label}</span>
                    </button>
                ))}
            </div>

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1 flex items-center gap-1 text-[10px] uppercase tracking-wider text-slate-300">
                    <Icon name="freeform.paint-smooth" size={12} aria-hidden />
                    Brush
                </p>
                <Row label="Strength (CTRL + wheel)">
                    <input
                        type="range"
                        min={0}
                        max={1}
                        step={0.05}
                        value={state.brush.strength}
                        onChange={(e) =>
                            onChange({
                                ...state,
                                brush: { ...state.brush, strength: parseFloat(e.target.value) },
                            })
                        }
                        className="w-full accent-orange-400"
                    />
                </Row>
                <Row label="Size (SHIFT + wheel)">
                    <input
                        type="range"
                        min={0.2}
                        max={5}
                        step={0.1}
                        value={state.brush.sizeMm}
                        onChange={(e) =>
                            onChange({
                                ...state,
                                brush: { ...state.brush, sizeMm: parseFloat(e.target.value) },
                            })
                        }
                        className="w-full accent-orange-400"
                    />
                </Row>
                <div className="mt-2 flex items-center gap-2 text-[11px]">
                    <span>Type:</span>
                    {brushTypes.map((t) => (
                        <button
                            key={t.id}
                            type="button"
                            onClick={() => setBrushType(t.id)}
                            className={[
                                'flex h-8 w-8 items-center justify-center rounded-md border',
                                state.brush.brushType === t.id
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/20 text-slate-200',
                            ].join(' ')}
                        >
                            <AppIcon name={t.icon} size={18} aria-hidden />
                        </button>
                    ))}
                </div>
            </section>
        </div>
    );
}

function AnatomicTab({
    state,
    onChange,
}: {
    state: FreeformState;
    onChange: (next: FreeformState) => void;
}) {
    const setPreset = (preset: AnatomicPreset) =>
        onChange({ ...state, anatomic: { ...state.anatomic, preset } });
    const toggleRestriction = (r: MovementRestriction) => {
        const next = new Set(state.anatomic.restrictions);
        if (next.has(r)) next.delete(r);
        else next.add(r);
        onChange({ ...state, anatomic: { ...state.anatomic, restrictions: next } });
    };

    const presets: Array<{ id: AnatomicPreset; label: string; icon: string }> = [
        { id: 'cusps', label: 'Cusps', icon: 'freeforming.preset-cusps' },
        { id: 'tooth-parts', label: 'Tooth parts', icon: 'freeforming.preset-tooth-parts' },
        { id: 'entire-tooth', label: 'Entire tooth', icon: 'freeforming.preset-entire-tooth' },
        { id: 'ridge', label: 'Ridge', icon: 'freeforming.preset-ridge' },
    ];
    const restrictions: Array<{ id: MovementRestriction; label: string; icon: string }> = [
        { id: 'occlusal-only', label: 'Occlusal only', icon: 'freeforming.occlusal-only' },
        { id: 'lock-cusp-tips', label: 'Lock cusp tips', icon: 'freeforming.lock-cusp-tips' },
        { id: 'lock-equator', label: 'Lock equator', icon: 'freeforming.lock-equator' },
    ];

    return (
        <div className="flex flex-col gap-2">
            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1 text-[10px] uppercase tracking-wider text-slate-300">Presets</p>
                <div className="grid grid-cols-2 gap-1.5">
                    {presets.map((p) => (
                        <button
                            key={p.id}
                            type="button"
                            onClick={() => setPreset(p.id)}
                            className={[
                                'flex flex-col items-center gap-1 rounded-md border px-2 py-2 text-[10px]',
                                state.anatomic.preset === p.id
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/20 text-slate-200',
                            ].join(' ')}
                        >
                            <AppIcon name={p.icon} size={22} aria-hidden />
                            <span className="leading-tight">{p.label}</span>
                        </button>
                    ))}
                </div>
            </section>

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1 text-[10px] uppercase tracking-wider text-slate-300">Movement restrictions</p>
                <div className="grid grid-cols-3 gap-1.5">
                    {restrictions.map((r) => {
                        const active = state.anatomic.restrictions.has(r.id);
                        return (
                            <button
                                key={r.id}
                                type="button"
                                onClick={() => toggleRestriction(r.id)}
                                className={[
                                    'flex flex-col items-center gap-1 rounded-md border px-1 py-1.5 text-[10px]',
                                    active
                                        ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                        : 'border-white/20 text-slate-200',
                                ].join(' ')}
                            >
                                <AppIcon name={r.icon} size={18} aria-hidden />
                                <span className="text-center leading-tight">{r.label}</span>
                            </button>
                        );
                    })}
                </div>
            </section>

            <label className="flex items-center gap-2 text-[11px]">
                <input
                    type="checkbox"
                    checked={state.anatomic.advancedPaintPull}
                    onChange={(e) =>
                        onChange({
                            ...state,
                            anatomic: { ...state.anatomic, advancedPaintPull: e.target.checked },
                        })
                    }
                    className="accent-orange-400"
                />
                Advanced Paint &amp; Pull
            </label>
        </div>
    );
}

function AttachmentTab({
    state,
    onChange,
    onApply,
    onUnload,
}: {
    state: FreeformState;
    onChange: (next: FreeformState) => void;
    onApply: () => void;
    onUnload: () => void;
}) {
    const set = (patch: Partial<typeof state.attachment>) =>
        onChange({ ...state, attachment: { ...state.attachment, ...patch } });

    const modeButtons: Array<{ id: AttachmentMode; label: string; icon: string }> = [
        { id: 'add', label: 'Add', icon: 'common.add' },
        { id: 'subtract', label: 'Subtract', icon: 'common.remove' },
    ];
    const insertionButtons: Array<{ id: InsertionDirectionSource; label: string; icon: string }> = [
        { id: 'top', label: 'Top', icon: 'freeforming.insertion-top' },
        { id: 'view', label: 'View', icon: 'freeforming.insertion-view' },
        { id: 'surface', label: 'Surface', icon: 'freeforming.insertion-surface' },
    ];

    return (
        <div className="flex flex-col gap-2">
            <div className="grid grid-cols-2 gap-1.5">
                {modeButtons.map((m) => (
                    <button
                        key={m.id}
                        type="button"
                        onClick={() => set({ mode: m.id })}
                        className={[
                            'flex items-center justify-center gap-1 rounded-md border px-2 py-2 text-[11px]',
                            state.attachment.mode === m.id
                                ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                : 'border-white/20 text-slate-200',
                        ].join(' ')}
                    >
                        <AppIcon name={m.icon} size={14} aria-hidden />
                        {m.label}
                    </button>
                ))}
            </div>

            <label className="flex items-center gap-2 text-[11px]">
                <span className="w-16">Library:</span>
                <input
                    type="text"
                    value={state.attachment.library}
                    onChange={(e) => set({ library: e.currentTarget.value })}
                    className="flex-1 rounded border border-white/20 bg-violet-900/60 px-2 py-1 text-[11px]"
                />
            </label>
            <label className="flex items-center gap-2 text-[11px]">
                <span className="w-16">Type:</span>
                <input
                    type="text"
                    value={state.attachment.type}
                    onChange={(e) => set({ type: e.currentTarget.value })}
                    className="flex-1 rounded border border-white/20 bg-violet-900/60 px-2 py-1 text-[11px]"
                />
            </label>

            <section className="rounded-md bg-violet-900/40 p-2">
                <p className="mb-1 text-[10px] uppercase tracking-wider text-slate-300">
                    Insertion Direction
                </p>
                <div className="grid grid-cols-3 gap-1.5">
                    {insertionButtons.map((b) => (
                        <button
                            key={b.id}
                            type="button"
                            onClick={() => set({ insertionDirection: b.id })}
                            className={[
                                'flex flex-col items-center gap-0.5 rounded-md border px-2 py-1.5 text-[10px]',
                                state.attachment.insertionDirection === b.id
                                    ? 'border-orange-400 bg-orange-500/20 text-orange-200'
                                    : 'border-white/20 text-slate-200',
                            ].join(' ')}
                        >
                            <AppIcon name={b.icon} size={16} aria-hidden />
                            {b.label}
                        </button>
                    ))}
                </div>
            </section>

            <label className="flex items-center gap-2 text-[11px]">
                <input
                    type="checkbox"
                    checked={state.attachment.cutOnGingiva}
                    onChange={(e) => set({ cutOnGingiva: e.target.checked })}
                    className="accent-orange-400"
                />
                Cut on gingiva
                <input
                    type="number"
                    step="0.1"
                    value={state.attachment.cutDistanceMm}
                    onChange={(e) => set({ cutDistanceMm: parseFloat(e.target.value) || 0 })}
                    className="ml-auto w-16 rounded border border-white/20 bg-violet-900/60 px-1.5 py-0.5 text-right text-[11px] tabular-nums"
                />
                <span className="text-[10px] text-slate-400">mm</span>
            </label>

            <div className="flex gap-2">
                <Button type="button" variant="secondary" size="sm" className="flex-1" onClick={onApply}>
                    Apply
                </Button>
                <Button type="button" variant="ghost" size="sm" className="flex-1" onClick={onUnload}>
                    Unload
                </Button>
            </div>
        </div>
    );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="mb-1">
            <p className="mb-0.5 text-[10px] text-slate-300">{label}</p>
            {children}
        </div>
    );
}
