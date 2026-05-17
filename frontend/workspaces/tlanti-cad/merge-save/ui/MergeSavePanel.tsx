/**
 * MergeSavePanel — Wizard panel replicating exocad's "Merge and Save
 * Restorations" (image #5). Pure presentational: parent owns state and
 * callbacks.
 */

import React from 'react';

import { Button } from '@/components/ui/button';
import { stageLabel, type MergeJobState } from '../domain/merge-job';

export interface MergeSavePanelProps {
    state: MergeJobState;
    onRestart: () => void;
    onCancel: () => void;
    onRemove: () => void;
    onToggleOptimize: (value: boolean) => void;
    onBack?: () => void;
    onNext?: () => void;
    activeTab: 'next-step' | 'saved-files' | 'screw-holes';
    onTabChange: (tab: 'next-step' | 'saved-files' | 'screw-holes') => void;
    screwHolesEnabled?: boolean;
}

export function MergeSavePanel({
    state,
    onRestart,
    onCancel,
    onRemove,
    onToggleOptimize,
    onBack,
    onNext,
    activeTab,
    onTabChange,
    screwHolesEnabled,
}: MergeSavePanelProps) {
    const running = state.status === 'running';
    const complete = state.status === 'complete';
    const canRemove = complete || state.savedFiles.length > 0;

    return (
        <div className="flex h-full w-[360px] flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-xl">
            <header className="flex items-center gap-3 bg-[#3B2B6F] px-4 py-3 text-white">
                <WandIcon />
                <div className="min-w-0 flex-1">
                    <h2 className="truncate text-sm font-semibold">Merge and Save Restorations</h2>
                </div>
                <button type="button" className="opacity-70 hover:opacity-100" title="Help">
                    <QuestionCircle />
                </button>
            </header>

            <nav className="flex gap-0 border-b border-border bg-[#4A3983] text-[11px] font-semibold uppercase tracking-wider">
                <TabButton active={activeTab === 'next-step'} onClick={() => onTabChange('next-step')}>
                    Next step:
                </TabButton>
                <TabButton active={activeTab === 'saved-files'} onClick={() => onTabChange('saved-files')}>
                    Saved files
                </TabButton>
                {screwHolesEnabled ? (
                    <TabButton
                        active={activeTab === 'screw-holes'}
                        onClick={() => onTabChange('screw-holes')}
                    >
                        Screw holes
                    </TabButton>
                ) : null}
            </nav>

            <div className="flex-1 overflow-y-auto px-4 py-4">
                {activeTab === 'saved-files' ? (
                    <SavedFilesTab
                        state={state}
                        onRestart={onRestart}
                        onCancel={onCancel}
                        onRemove={onRemove}
                        canRemove={canRemove}
                        running={running}
                        onToggleOptimize={onToggleOptimize}
                    />
                ) : activeTab === 'next-step' ? (
                    <NextStepPlaceholder complete={complete} />
                ) : null}
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
                    disabled={!complete}
                >
                    Next →
                </Button>
            </footer>
        </div>
    );
}

function TabButton({
    active,
    onClick,
    children,
}: {
    active: boolean;
    onClick: () => void;
    children: React.ReactNode;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={[
                'flex-1 px-3 py-2 transition',
                active ? 'border-b-2 border-white text-white' : 'text-white/70 hover:text-white',
            ].join(' ')}
        >
            {children}
        </button>
    );
}

function SavedFilesTab({
    state,
    onRestart,
    onCancel,
    onRemove,
    canRemove,
    running,
    onToggleOptimize,
}: {
    state: MergeJobState;
    onRestart: () => void;
    onCancel: () => void;
    onRemove: () => void;
    canRemove: boolean;
    running: boolean;
    onToggleOptimize: (value: boolean) => void;
}) {
    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
                <ActionButton label="Restart Merging" onClick={onRestart} disabled={running} />
                <ActionButton label="Cancel Merging" onClick={onCancel} disabled={!running} />
                <ActionButton
                    label="Remove Existing Merged Parts"
                    onClick={onRemove}
                    disabled={!canRemove}
                    variant="outline"
                />
            </div>

            <label className="flex items-start gap-2 rounded-md border border-border bg-surface-sunken px-2 py-2 text-xs text-text-primary">
                <input
                    type="checkbox"
                    checked={state.optimizeFor3dPrint}
                    onChange={(e) => onToggleOptimize(e.currentTarget.checked)}
                    disabled={running}
                    className="mt-[2px] accent-sky-400"
                />
                <span className="leading-snug">
                    Optimize for better free-forming and 3D printing.
                    <span className="mt-0.5 block text-[10px] text-text-secondary">
                        Watertight output — needed for SLM/3D printing and recommended if you plan
                        further free-forming.
                    </span>
                </span>
            </label>

            <ProgressStrip state={state} />

            <SavedFilesBox state={state} />
        </div>
    );
}

function ProgressStrip({ state }: { state: MergeJobState }) {
    if (state.status === 'idle') {
        return (
            <div className="rounded-md border border-dashed border-border bg-surface-sunken px-3 py-3 text-[11px] text-text-secondary">
                Press <span className="font-semibold text-text-primary">Restart Merging</span> to
                combine all designed parts into one or more producible STL meshes.
            </div>
        );
    }
    const stage = state.currentStage ?? 'union';
    const color =
        state.status === 'error'
            ? 'bg-rose-500'
            : state.status === 'cancelled'
              ? 'bg-amber-500'
              : 'bg-emerald-500';
    return (
        <div className="rounded-md border border-border bg-surface-sunken px-3 py-2">
            <div className="flex items-center justify-between text-[11px] text-text-primary">
                <span>{stageLabel(stage)}</span>
                <span className="tabular-nums text-text-secondary">{state.percent}%</span>
            </div>
            <div className="mt-1.5 h-1 overflow-hidden rounded bg-border">
                <div className={`h-full ${color} transition-all`} style={{ width: `${state.percent}%` }} />
            </div>
            {state.backend ? (
                <div className="mt-1 text-[10px] text-text-secondary">
                    backend: <span className="font-mono">{state.backend}</span>
                    {state.errorMessage ? (
                        <span className="ml-2 text-rose-400">· {state.errorMessage}</span>
                    ) : null}
                </div>
            ) : null}
        </div>
    );
}

function SavedFilesBox({ state }: { state: MergeJobState }) {
    const hasFiles = state.savedFiles.length > 0;
    return (
        <div className="min-h-[160px] rounded-md border border-border bg-surface-sunken px-3 py-2.5">
            {hasFiles ? (
                <ul className="flex flex-col gap-1.5">
                    {state.savedFiles.map((file) => (
                        <li
                            key={file.path}
                            draggable
                            className="cursor-grab rounded border border-border bg-surface-raised px-2 py-1.5 text-[11px] text-text-primary hover:border-sky-400"
                            title={`Drag to Finder, email, or CAM software\n${file.path}`}
                        >
                            <span className="font-semibold">{file.name}</span>
                            <span className="ml-2 text-[10px] uppercase tracking-wider text-text-secondary">
                                {file.kind === 'stl' ? 'mesh' : 'meta'}
                            </span>
                        </li>
                    ))}
                </ul>
            ) : (
                <p className="text-[11px] leading-snug text-text-secondary">
                    Once merging is complete, the resulting <code>_cad.stl</code> and{' '}
                    <code>.constructionInfo</code> files will appear here. Drag them directly into
                    Finder, your CAM software, an email composer, or an FTP app.
                </p>
            )}
        </div>
    );
}

function NextStepPlaceholder({ complete }: { complete: boolean }) {
    return (
        <div className="rounded-md border border-border bg-surface-sunken px-3 py-4 text-[11px] leading-snug text-text-secondary">
            {complete ? (
                <p>
                    Design finished. Files saved to project directory. Switch to{' '}
                    <span className="font-semibold text-text-primary">Saved files</span> to pick up
                    the mesh, or use the bottom toolbar to proceed.
                </p>
            ) : (
                <p>
                    Run Merge from the <span className="font-semibold">Saved files</span> tab first.
                </p>
            )}
        </div>
    );
}

function ActionButton({
    label,
    onClick,
    disabled,
    variant,
}: {
    label: string;
    onClick: () => void;
    disabled?: boolean;
    variant?: 'outline';
}) {
    const base =
        variant === 'outline'
            ? 'border border-border bg-surface-sunken hover:bg-surface-raised'
            : 'border border-border bg-surface-raised hover:bg-surface-sunken';
    return (
        <button
            type="button"
            disabled={disabled}
            onClick={onClick}
            className={[
                'rounded-md px-3 py-2 text-left text-xs font-semibold transition',
                base,
                disabled ? 'opacity-50' : 'text-text-primary',
            ].join(' ')}
        >
            {label}
        </button>
    );
}

function WandIcon() {
    return (
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M15 4l2 2" />
            <path d="M4 20l9-9" />
            <path d="M17 8l3 3" />
            <path d="M12 3l1 2-2 1 2 1-1 2-1-2 2-1-2-1z" />
        </svg>
    );
}

function QuestionCircle() {
    return (
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="9" />
            <path d="M9.5 9a2.5 2.5 0 1 1 3.5 2.3c-0.7 0.3-1 0.8-1 1.7" />
            <path d="M12 17h0.01" />
        </svg>
    );
}
