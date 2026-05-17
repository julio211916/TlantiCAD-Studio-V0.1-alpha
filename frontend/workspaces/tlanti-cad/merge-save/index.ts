export type {
    MergeJobState,
    MergeStage,
    MergeStatus,
    SavedMergeFile,
} from './domain/merge-job';
export { initialMergeJobState, shouldDefaultOptimize, stageLabel } from './domain/merge-job';

export type {
    MergeCompleteOutput,
    MergePort,
    MergeProgressOutput,
    MergeStartInput,
    MergeStartOutput,
    MergeToothPayload,
} from './application/merge-port';

export { createBackendMergeAdapter } from './infrastructure/backend-merge-adapter';

export { MergeSavePanel } from './ui/MergeSavePanel';
export type { MergeSavePanelProps } from './ui/MergeSavePanel';
export { NextStepsPanel } from './ui/NextStepsPanel';
export type { NextStepChoice, NextStepsPanelProps } from './ui/NextStepsPanel';
export { useMergeJob } from './ui/useMergeJob';
export type { CsgBridge, CsgInputResolver, UseMergeJobResult } from './ui/useMergeJob';
