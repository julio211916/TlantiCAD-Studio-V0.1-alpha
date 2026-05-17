/**
 * SegmentStudyUseCase — orchestrates a CBCT / CT segmentation job by
 * posting the active study to the backend (which routes to
 * SlicerDentalSegmentator or AMASSS) and polling for completion.
 *
 * Pure TypeScript. Depends only on a `SegmentationBackendPort`. The UI
 * layer wires the real fetch adapter; tests inject a stub.
 */

import type { DicomStudy } from '../domain/dicom-study';

export type SegmentationJobStatus =
    | 'queued'
    | 'running'
    | 'succeeded'
    | 'failed'
    | 'cancelled';

export interface SegmentationLabel {
    labelId: number;
    labelName: string;
    voxelCount: number;
}

export interface SegmentationJob {
    jobId: string;
    status: SegmentationJobStatus;
    progress: number; // 0..1, null when backend doesn't report progress
    error: string | null;
    labels: SegmentationLabel[];
    /** Server-side path to the .nii.gz — downloaded on demand. */
    segmentationUrl: string | null;
    stdoutTail: string[];
    stderrTail: string[];
}

export interface SegmentationBackendPort {
    /** Launch a segmentation job, returns initial record. */
    launch(params: {
        studyInstanceUID: string;
        seriesInstanceUID: string;
        vendor: 'slicer_dental_segmentator' | 'amass';
    }): Promise<SegmentationJob>;
    /** Poll job status. Returns null if jobId unknown. */
    poll(jobId: string): Promise<SegmentationJob | null>;
    /** Cancel an in-flight job if supported. */
    cancel(jobId: string): Promise<void>;
}

export interface SegmentStudyUseCase {
    /**
     * Run a segmentation for the first volumetric series in the study.
     * Returns a generator that yields progress updates and finally the
     * final record. Callers await via `for await (const snapshot of ...)`.
     */
    run(study: DicomStudy): AsyncGenerator<SegmentationJob, SegmentationJob, void>;
    /** Cancel the active job (if any). */
    cancel(jobId: string): Promise<void>;
}

export interface SegmentStudyDeps {
    backend: SegmentationBackendPort;
    /** Interval between poll requests (ms). */
    pollIntervalMs?: number;
    /** Maximum total runtime before giving up (ms). */
    timeoutMs?: number;
    /** Delay helper — swap in tests. */
    sleep?: (ms: number) => Promise<void>;
}

export function createSegmentStudyUseCase(
    deps: SegmentStudyDeps,
): SegmentStudyUseCase {
    const pollIntervalMs = deps.pollIntervalMs ?? 1_500;
    const timeoutMs = deps.timeoutMs ?? 10 * 60 * 1_000;
    const sleep =
        deps.sleep ?? ((ms) => new Promise<void>((resolve) => setTimeout(resolve, ms)));

    function pickVolumetricSeries(study: DicomStudy) {
        const volumetric = study.series.find(
            (series) => series.isVolumetric && series.instanceCount >= 16,
        );
        return volumetric ?? null;
    }

    return {
        async *run(study) {
            const series = pickVolumetricSeries(study);
            if (!series) {
                throw new Error(
                    'Segmentation requires a volumetric series (≥16 slices, consistent spacing).',
                );
            }

            const initial = await deps.backend.launch({
                studyInstanceUID: study.studyInstanceUID,
                seriesInstanceUID: series.seriesInstanceUID,
                vendor: 'slicer_dental_segmentator',
            });
            yield initial;

            const startedAt = Date.now();
            let current = initial;

            while (current.status === 'queued' || current.status === 'running') {
                if (Date.now() - startedAt > timeoutMs) {
                    throw new Error('Segmentation timed out.');
                }
                await sleep(pollIntervalMs);
                const next = await deps.backend.poll(current.jobId);
                if (!next) {
                    throw new Error(`Job not found: ${current.jobId}`);
                }
                current = next;
                yield current;
            }

            return current;
        },

        async cancel(jobId) {
            await deps.backend.cancel(jobId);
        },
    };
}
