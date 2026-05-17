/**
 * useSegmentationRunner — stateful wrapper that consumes the
 * SegmentStudyUseCase generator and exposes React-friendly state.
 *
 * Callers:
 *   const { job, start, cancel, dismiss } = useSegmentationRunner();
 *   <button onClick={() => start(study)}>AI Segmentation</button>
 *   <DicomSegmentationOverlay job={job} onCancel={cancel} onDismiss={dismiss} />
 */

import { useCallback, useRef, useState } from 'react';

import type { DicomStudy } from '../domain/dicom-study';
import {
    createSegmentStudyUseCase,
    type SegmentationJob,
    type SegmentStudyUseCase,
} from '../application/segment-study.use-case';
import { createBackendSegmentationAdapter } from '../infrastructure/backend-segmentation-adapter';

interface UseSegmentationRunnerResult {
    job: SegmentationJob | null;
    isRunning: boolean;
    start: (study: DicomStudy) => Promise<void>;
    cancel: (jobId: string) => Promise<void>;
    dismiss: () => void;
}

function createDefaultUseCase(): SegmentStudyUseCase {
    return createSegmentStudyUseCase({
        backend: createBackendSegmentationAdapter(),
    });
}

export function useSegmentationRunner(
    useCaseFactory: () => SegmentStudyUseCase = createDefaultUseCase,
): UseSegmentationRunnerResult {
    const [job, setJob] = useState<SegmentationJob | null>(null);
    const [isRunning, setIsRunning] = useState(false);
    const abortRef = useRef<boolean>(false);
    const useCaseRef = useRef<SegmentStudyUseCase | null>(null);

    if (useCaseRef.current === null) {
        useCaseRef.current = useCaseFactory();
    }

    const start = useCallback(
        async (study: DicomStudy) => {
            if (!useCaseRef.current) return;
            setIsRunning(true);
            abortRef.current = false;
            try {
                const iterator = useCaseRef.current.run(study);
                while (true) {
                    const next = await iterator.next();
                    if (abortRef.current) {
                        break;
                    }
                    if (next.value) {
                        setJob(next.value);
                    }
                    if (next.done) break;
                }
            } catch (err) {
                setJob((current) => ({
                    jobId: current?.jobId ?? 'local',
                    status: 'failed',
                    progress: current?.progress ?? 0,
                    error: err instanceof Error ? err.message : String(err),
                    labels: [],
                    segmentationUrl: null,
                    stdoutTail: current?.stdoutTail ?? [],
                    stderrTail: current?.stderrTail ?? [],
                }));
            } finally {
                setIsRunning(false);
            }
        },
        [],
    );

    const cancel = useCallback(async (jobId: string) => {
        abortRef.current = true;
        if (useCaseRef.current) {
            try {
                await useCaseRef.current.cancel(jobId);
            } catch (err) {
                const message = err instanceof Error ? err.message : String(err);
                setJob((current) =>
                    current
                        ? {
                              ...current,
                              status: 'failed',
                              error: `Cancellation failed: ${message}`,
                              stderrTail: [...current.stderrTail, message].slice(-20),
                          }
                        : current,
                );
                throw err;
            }
        }
        setJob((current) =>
            current ? { ...current, status: 'cancelled', error: 'Cancelled by user.' } : current,
        );
    }, []);

    const dismiss = useCallback(() => {
        abortRef.current = true;
        setJob(null);
    }, []);

    return { job, isRunning, start, cancel, dismiss };
}
