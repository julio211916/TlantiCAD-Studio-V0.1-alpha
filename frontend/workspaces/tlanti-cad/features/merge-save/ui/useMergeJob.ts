/**
 * useMergeJob — state machine wrapping MergePort with poll-based progress.
 *
 * V201 — when a CSG bridge is provided AND the case has real input STL paths
 * (e.g. per-tooth abutment + crown previews), `useMergeJob` invokes the
 * Tauri kernel after Python finalize to overwrite the stub STL with a real
 * boolean union. Without a bridge or inputs the Python `_cad.stl` stub is
 * kept verbatim.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import type { MergePort, MergeStartInput } from '../application/merge-port';
import {
    type MergeJobState,
    initialMergeJobState,
} from '../domain/merge-job';
import type { MeshOpResponse } from '../../../lib/csg-bridge';
import { logger } from '../../../lib/logger';

const POLL_INTERVAL_MS = 320;

/** Resolves the input STL paths the CSG kernel should union. */
export type CsgInputResolver = () =>
    | { inputs: string[]; output: string }
    | null;

export interface CsgBridge {
    resolve: CsgInputResolver;
    invoke: (request: {
        op: 'union' | 'subtract' | 'intersect';
        inputs: string[];
        output: string;
        repair?: boolean;
    }) => Promise<MeshOpResponse | null>;
}

export interface UseMergeJobResult {
    state: MergeJobState;
    startMerge: (input: MergeStartInput) => Promise<void>;
    cancelMerge: () => Promise<void>;
    removeMergedParts: (caseFolderPath: string) => Promise<number>;
    setOptimize: (value: boolean) => void;
    reset: () => void;
}

export function useMergeJob(port: MergePort, csgBridge?: CsgBridge): UseMergeJobResult {
    const [state, setState] = useState<MergeJobState>(() => initialMergeJobState());
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const stopPolling = useCallback(() => {
        if (timerRef.current !== null) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
    }, []);

    useEffect(() => () => stopPolling(), [stopPolling]);

    const pollOnce = useCallback(
        async (jobId: string) => {
            try {
                const progress = await port.progress(jobId);
                setState((prev) => ({
                    ...prev,
                    currentStage: progress.stage,
                    percent: progress.percent,
                }));
                if (progress.percent >= 100) {
                    const complete = await port.finalize(jobId);

                    // V201 — best-effort bridge to the real CSG kernel.
                    // Python finalize wrote a stub STL; if the case has real
                    // input meshes and we're in Tauri runtime, overwrite the
                    // stub with a real boolean union so the CAM handoff
                    // receives valid geometry.
                    let bridgeOutcome: {
                        triangles: number;
                        watertight: boolean;
                        volumeMm3: number;
                        backend: string;
                    } | null = null;
                    if (csgBridge) {
                        const resolved = csgBridge.resolve();
                        if (resolved && resolved.inputs.length >= 2) {
                            try {
                                const result = await csgBridge.invoke({
                                    op: 'union',
                                    inputs: resolved.inputs,
                                    output: resolved.output,
                                    repair: false,
                                });
                                if (result) {
                                    bridgeOutcome = {
                                        triangles: result.triangles,
                                        watertight: result.watertight,
                                        volumeMm3: result.volumeMm3,
                                        backend: result.backend,
                                    };
                                }
                            } catch (err) {
                                logger.warn('CSG bridge failed; keeping Python stub', err);
                            }
                        }
                    }

                    setState((prev) => ({
                        ...prev,
                        status: 'complete',
                        percent: 100,
                        savedFiles: complete.savedFiles,
                        watertight: bridgeOutcome?.watertight ?? complete.watertight,
                        backend: bridgeOutcome?.backend ?? complete.backend,
                    }));
                    return;
                }
                timerRef.current = setTimeout(() => void pollOnce(jobId), POLL_INTERVAL_MS);
            } catch (error) {
                setState((prev) => ({
                    ...prev,
                    status: 'error',
                    errorMessage: error instanceof Error ? error.message : String(error),
                }));
            }
        },
        [port, csgBridge],
    );

    const startMerge = useCallback(
        async (input: MergeStartInput) => {
            stopPolling();
            setState((prev) => ({
                ...initialMergeJobState(),
                optimizeFor3dPrint: input.optimizeFor3dPrint || prev.optimizeFor3dPrint,
                status: 'running',
            }));
            try {
                const started = await port.start(input);
                setState((prev) => ({
                    ...prev,
                    jobId: started.jobId,
                    stages: started.stages,
                    backend: started.backend,
                }));
                timerRef.current = setTimeout(
                    () => void pollOnce(started.jobId),
                    POLL_INTERVAL_MS,
                );
            } catch (error) {
                setState((prev) => ({
                    ...prev,
                    status: 'error',
                    errorMessage: error instanceof Error ? error.message : String(error),
                }));
            }
        },
        [pollOnce, port, stopPolling],
    );

    const cancelMerge = useCallback(async () => {
        stopPolling();
        if (!state.jobId) return;
        try {
            await port.cancel(state.jobId);
        } finally {
            setState((prev) => ({ ...prev, status: 'cancelled' }));
        }
    }, [port, state.jobId, stopPolling]);

    const removeMergedParts = useCallback(
        async (caseFolderPath: string) => {
            stopPolling();
            const removed = await port.remove(caseFolderPath);
            setState(() => initialMergeJobState());
            return removed;
        },
        [port, stopPolling],
    );

    const setOptimize = useCallback((value: boolean) => {
        setState((prev) => ({ ...prev, optimizeFor3dPrint: value }));
    }, []);

    const reset = useCallback(() => {
        stopPolling();
        setState(() => initialMergeJobState());
    }, [stopPolling]);

    return { state, startMerge, cancelMerge, removeMergedParts, setOptimize, reset };
}
