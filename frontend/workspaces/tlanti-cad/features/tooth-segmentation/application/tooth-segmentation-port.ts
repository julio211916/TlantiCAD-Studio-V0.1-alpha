/**
 * Port that the Crown Segmentation panel talks to. An adapter backed by the
 * FastAPI clinical-vendors queue implements it (tooth_group_network vendor).
 */

import type { JawKind } from '../domain/fdi-chart';

export type CrownSegJobStatus =
    | 'queued'
    | 'running'
    | 'succeeded'
    | 'failed'
    | 'cancelled';

export interface CrownSegJob {
    jobId: string;
    status: CrownSegJobStatus;
    progress: number; // 0..1
    /** FDI numbers the backend has finalised so far. */
    segmentedTeeth: number[];
    error: string | null;
    stdoutTail: string[];
    stderrTail: string[];
}

export interface CrownSegmentationLaunchArgs {
    jaw: JawKind;
    extractGingiva: boolean;
    keepSegmented: boolean;
    /** Optional hint of teeth the clinician already marked as missing. */
    skipTeeth?: number[];
    /** Reference to the scan — mesh path or study/series UIDs. */
    scanRef: { kind: 'mesh'; path: string } | { kind: 'study'; uid: string };
}

export interface ToothSegmentationPort {
    launch(args: CrownSegmentationLaunchArgs): Promise<CrownSegJob>;
    poll(jobId: string): Promise<CrownSegJob | null>;
    cancel(jobId: string): Promise<void>;
}
