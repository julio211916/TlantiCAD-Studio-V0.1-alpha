/**
 * Port contract for the Merge & Save use case.
 * Adapters (HTTP, Tauri-IPC, in-process Rust) implement this interface.
 */

import type { MergeStage, SavedMergeFile } from '../domain/merge-job';

export interface MergeToothPayload {
    tooth: number;
    workTypeId?: string;
    material?: string;
    shade?: string;
    workTimeMinutes?: number;
    marginPolyline?: { x: number; y: number; z: number }[];
    insertionAxis?: { x: number; y: number; z: number };
}

export interface MergeStartInput {
    caseId: string;
    caseFolderPath: string;
    /**
     * Rich tooth payload — backend writes each entry to the `.constructionInfo`
     * sidecar so the CAM software receives margin/axis/material per restoration.
     */
    teeth: MergeToothPayload[];
    optimizeFor3dPrint: boolean;
}

export interface MergeStartOutput {
    jobId: string;
    stages: MergeStage[];
    backend: string;
}

export interface MergeProgressOutput {
    stage: MergeStage;
    percent: number;
}

export interface MergeCompleteOutput {
    savedFiles: SavedMergeFile[];
    watertight: boolean;
    backend: string;
}

export interface MergePort {
    start(input: MergeStartInput): Promise<MergeStartOutput>;
    progress(jobId: string): Promise<MergeProgressOutput>;
    cancel(jobId: string): Promise<void>;
    finalize(jobId: string): Promise<MergeCompleteOutput>;
    remove(caseFolderPath: string): Promise<number>;
}
