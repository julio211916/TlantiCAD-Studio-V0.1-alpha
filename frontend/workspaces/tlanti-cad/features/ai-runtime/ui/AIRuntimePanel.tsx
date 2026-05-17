/**
 * AI runtime panel (V320).
 *
 * Surfaces the sidecar's current state to the user:
 *   - Which torch backend is active (CUDA / MPS / CPU)
 *   - Approximate memory headroom + voxel-budget hint
 *   - The 4 model entries with installed / verified status
 *   - Buttons: verify integrity, prune cache, clear cache
 *
 * Pure presentational; the parent owns data + callbacks. Designed for the
 * `/settings/ai` route (lab admin tab) but also openable via Cmd+K.
 */

import React from 'react';

import {
    deviceLatencyLabel,
    formatBytes,
    modelHealthLabel,
    type AICacheState,
    type AIRuntimeStatus,
} from '../domain/runtime';

export interface AIRuntimePanelProps {
    status: AIRuntimeStatus | null;
    cache: AICacheState | null;
    isLoading: boolean;
    error: string | null;
    onRefresh: () => void;
    onVerifyModel: (modelId: string) => void;
    onPruneCache: () => void;
    onClearCache: () => void;
}

export function AIRuntimePanel({
    status,
    cache,
    isLoading,
    error,
    onRefresh,
    onVerifyModel,
    onPruneCache,
    onClearCache,
}: AIRuntimePanelProps) {
    return (
        <section
            aria-label="AI runtime"
            className="flex flex-col gap-3 rounded-lg border border-border bg-surface-raised p-4 text-sm"
        >
            <header className="flex items-center justify-between">
                <h2 className="text-sm font-semibold text-text-primary">AI runtime</h2>
                <button
                    type="button"
                    onClick={onRefresh}
                    disabled={isLoading}
                    className="rounded-md border border-border bg-surface-sunken px-2.5 py-1 text-[11px] hover:bg-surface-raised disabled:opacity-40"
                >
                    {isLoading ? 'Refreshing…' : '↻ Refresh'}
                </button>
            </header>

            {error ? (
                <p className="rounded border border-rose-500/40 bg-rose-500/10 px-2 py-1 text-[11px] text-rose-200">
                    {error}
                </p>
            ) : null}

            {status ? (
                <>
                    <DeviceBlock status={status} />
                    <ModelsBlock status={status} onVerify={onVerifyModel} />
                </>
            ) : (
                <p className="text-text-secondary">No status yet — click Refresh.</p>
            )}

            {cache ? (
                <CacheBlock cache={cache} onPrune={onPruneCache} onClear={onClearCache} />
            ) : null}
        </section>
    );
}

function DeviceBlock({ status }: { status: AIRuntimeStatus }) {
    const memPct = status.memory.totalBytes
        ? Math.round((status.memory.usedBytes * 100) / status.memory.totalBytes)
        : 0;
    return (
        <div className="rounded border border-border bg-surface-sunken/40 p-3">
            <div className="flex items-baseline justify-between">
                <span className="text-[10px] uppercase tracking-wider text-text-secondary">
                    Backend
                </span>
                <span
                    className={[
                        'rounded px-1.5 py-0.5 font-mono text-[10px] uppercase',
                        status.device.kind === 'cuda'
                            ? 'bg-emerald-500/20 text-emerald-300'
                            : status.device.kind === 'mps'
                              ? 'bg-sky-500/20 text-sky-300'
                              : 'bg-amber-500/20 text-amber-300',
                    ].join(' ')}
                >
                    {status.device.kind}
                </span>
            </div>
            <p className="mt-1 text-sm font-semibold text-text-primary">{status.device.name}</p>
            <p className="text-[11px] text-text-secondary">{status.device.detail}</p>
            <p className="mt-2 text-[10px] uppercase tracking-wider text-text-secondary">
                Memory · {memPct}% used
            </p>
            <div className="mt-1 h-1.5 w-full rounded bg-surface-raised">
                <div
                    className="h-full rounded bg-sky-400"
                    style={{ width: `${memPct}%` }}
                    aria-hidden
                />
            </div>
            <p className="mt-1 font-mono text-[10px] text-text-secondary">
                {formatBytes(status.memory.usedBytes)} / {formatBytes(status.memory.totalBytes)} ·
                safe for ~{Math.cbrt(status.memory.safeForVolumeVoxels).toFixed(0)}³ voxels
            </p>
            <p className="mt-1 text-[10px] text-text-secondary">{deviceLatencyLabel(status.device)}</p>
        </div>
    );
}

function ModelsBlock({
    status,
    onVerify,
}: {
    status: AIRuntimeStatus;
    onVerify: (id: string) => void;
}) {
    return (
        <div className="rounded border border-border bg-surface-sunken/40 p-3">
            <p className="text-[10px] uppercase tracking-wider text-text-secondary">
                Models ({status.models.length})
            </p>
            <ul className="mt-2 flex flex-col gap-1.5">
                {status.models.map((model) => {
                    const health = modelHealthLabel(model);
                    return (
                        <li
                            key={model.modelId}
                            className="flex items-center gap-3 rounded border border-border bg-surface-raised px-2 py-1.5"
                        >
                            <div className="min-w-0 flex-1">
                                <p className="truncate font-mono text-[11px] text-text-primary">
                                    {model.modelId}
                                </p>
                                <p className="truncate text-[10px] text-text-secondary">
                                    {model.installed
                                        ? `${formatBytes(model.sizeBytes)}`
                                        : `not on disk · ~${formatBytes(model.expectedBytes)}`}
                                    {model.lastError ? ` · ${model.lastError}` : ''}
                                </p>
                            </div>
                            <HealthBadge health={health} />
                            <button
                                type="button"
                                onClick={() => onVerify(model.modelId)}
                                disabled={!model.installed}
                                className="shrink-0 rounded border border-border px-1.5 py-0.5 text-[10px] hover:bg-surface-sunken disabled:opacity-30"
                            >
                                Verify
                            </button>
                        </li>
                    );
                })}
            </ul>
        </div>
    );
}

function HealthBadge({ health }: { health: 'missing' | 'corrupt' | 'unverified' | 'ok' }) {
    const styles = {
        missing: 'bg-zinc-500/20 text-zinc-300',
        corrupt: 'bg-rose-500/20 text-rose-300',
        unverified: 'bg-amber-500/20 text-amber-300',
        ok: 'bg-emerald-500/20 text-emerald-300',
    } as const;
    return (
        <span
            className={`shrink-0 rounded px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-wider ${styles[health]}`}
        >
            {health}
        </span>
    );
}

function CacheBlock({
    cache,
    onPrune,
    onClear,
}: {
    cache: AICacheState;
    onPrune: () => void;
    onClear: () => void;
}) {
    const usagePct = cache.budgetBytes
        ? Math.round((cache.totalBytes * 100) / cache.budgetBytes)
        : 0;
    return (
        <div className="rounded border border-border bg-surface-sunken/40 p-3">
            <div className="flex items-baseline justify-between">
                <p className="text-[10px] uppercase tracking-wider text-text-secondary">
                    Inference cache · {cache.entryCount} entries
                </p>
                <div className="flex gap-1">
                    <button
                        type="button"
                        onClick={onPrune}
                        className="rounded border border-border px-2 py-0.5 text-[10px] hover:bg-surface-sunken"
                    >
                        Prune
                    </button>
                    <button
                        type="button"
                        onClick={onClear}
                        className="rounded border border-rose-500/40 px-2 py-0.5 text-[10px] text-rose-300 hover:bg-rose-500/10"
                    >
                        Clear
                    </button>
                </div>
            </div>
            <p className="mt-1 font-mono text-[10px] text-text-secondary">
                {formatBytes(cache.totalBytes)} / {formatBytes(cache.budgetBytes)} ({usagePct}%)
            </p>
            <div className="mt-1 h-1 w-full rounded bg-surface-raised">
                <div
                    className={`h-full rounded ${usagePct > 90 ? 'bg-rose-500' : 'bg-emerald-400'}`}
                    style={{ width: `${Math.min(usagePct, 100)}%` }}
                    aria-hidden
                />
            </div>
        </div>
    );
}
