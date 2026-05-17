/**
 * React hook that drives the Crown Segmentation panel. Wraps the port's
 * launch → poll → completion flow and exposes friendly state.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import type {
    CrownSegJob,
    CrownSegmentationLaunchArgs,
    ToothSegmentationPort,
} from '../application/tooth-segmentation-port';

interface UseCrownSegmentationResult {
    job: CrownSegJob | null;
    isRunning: boolean;
    launch: (args: CrownSegmentationLaunchArgs) => Promise<void>;
    cancel: () => Promise<void>;
    reset: () => void;
}

export function useCrownSegmentation(
    portFactory: () => ToothSegmentationPort,
): UseCrownSegmentationResult {
    const portRef = useRef<ToothSegmentationPort | null>(null);
    if (portRef.current === null) portRef.current = portFactory();

    const [job, setJob] = useState<CrownSegJob | null>(null);
    const [isRunning, setIsRunning] = useState(false);
    const abortRef = useRef(false);

    useEffect(() => {
        return () => {
            abortRef.current = true;
        };
    }, []);

    const launch = useCallback(async (args: CrownSegmentationLaunchArgs) => {
        if (!portRef.current) return;
        setIsRunning(true);
        abortRef.current = false;
        try {
            const initial = await portRef.current.launch(args);
            setJob(initial);

            let current = initial;
            while (
                !abortRef.current &&
                (current.status === 'queued' || current.status === 'running')
            ) {
                await new Promise((r) => setTimeout(r, 800));
                const polled = await portRef.current.poll(current.jobId);
                if (!polled) break;
                current = polled;
                setJob(polled);
            }
        } catch (err) {
            setJob((prev) => ({
                jobId: prev?.jobId ?? 'local',
                status: 'failed',
                progress: prev?.progress ?? 0,
                segmentedTeeth: prev?.segmentedTeeth ?? [],
                error: err instanceof Error ? err.message : String(err),
                stdoutTail: prev?.stdoutTail ?? [],
                stderrTail: prev?.stderrTail ?? [],
            }));
        } finally {
            setIsRunning(false);
        }
    }, []);

    const cancel = useCallback(async () => {
        abortRef.current = true;
        if (portRef.current && job) {
            await portRef.current.cancel(job.jobId);
            setJob({ ...job, status: 'cancelled', error: 'Cancelled by user.' });
        }
    }, [job]);

    const reset = useCallback(() => {
        abortRef.current = true;
        setJob(null);
        setIsRunning(false);
    }, []);

    return { job, isRunning, launch, cancel, reset };
}
