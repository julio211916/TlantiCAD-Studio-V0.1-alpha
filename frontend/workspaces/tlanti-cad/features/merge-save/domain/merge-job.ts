/**
 * Pure domain model for the Merge & Save wizard step (V88-V89).
 *
 * Replicates exocad's "Merge and Save Restorations" workflow: combine the
 * individual constructed parts (copings, connectors, attachments, …) into one
 * or more watertight STL meshes and save them to the project directory along
 * with a `.constructionInfo` sidecar consumed by CAM software.
 */

export type MergeStage =
    | 'union'
    | 'screw-holes'
    | 'optimize-3dprint'
    | 'export-stl'
    | 'export-info';

export type MergeStatus = 'idle' | 'running' | 'complete' | 'cancelled' | 'error';

export interface SavedMergeFile {
    name: string;
    path: string;
    kind: 'stl' | 'constructionInfo';
    sizeBytes: number;
}

export interface MergeJobState {
    jobId: string | null;
    status: MergeStatus;
    currentStage: MergeStage | null;
    stages: MergeStage[];
    percent: number;
    optimizeFor3dPrint: boolean;
    savedFiles: SavedMergeFile[];
    watertight: boolean;
    backend: string | null;
    errorMessage: string | null;
}

export function initialMergeJobState(): MergeJobState {
    return {
        jobId: null,
        status: 'idle',
        currentStage: null,
        stages: [],
        percent: 0,
        optimizeFor3dPrint: false,
        savedFiles: [],
        watertight: false,
        backend: null,
        errorMessage: null,
    };
}

const LASER_MELTING_MATERIALS = new Set(['cobalt-chrome', 'titanium-laser', 'titanium']);

export function shouldDefaultOptimize(material: string | undefined): boolean {
    if (!material) return false;
    return LASER_MELTING_MATERIALS.has(material);
}

export function stageLabel(stage: MergeStage): string {
    switch (stage) {
        case 'union':
            return 'Combining parts';
        case 'screw-holes':
            return 'Cutting screw holes';
        case 'optimize-3dprint':
            return 'Optimizing for 3D printing';
        case 'export-stl':
            return 'Writing STL meshes';
        case 'export-info':
            return 'Writing constructionInfo';
        default:
            return stage;
    }
}
