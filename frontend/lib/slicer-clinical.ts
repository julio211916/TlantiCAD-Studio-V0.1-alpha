import { ipc } from '@/lib/ipc';

export type SlicerClinicalState = 'queued' | 'downloading' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface SlicerRuntimeStatus {
  ready: boolean;
  slicerRoot: string;
  manifestPath: string;
  executable: string;
  executablePresent: boolean;
  extensionPath: string;
  extensionPresent: boolean;
  version?: string | null;
  downloadUrl?: string | null;
  sha512?: string | null;
}

export interface SlicerModelArtifactStatus {
  name: string;
  url: string;
  kind: string;
  target: string;
  present: boolean;
  bytes?: number | null;
  sha256?: string | null;
}

export interface SlicerModelStatus {
  id: string;
  label: string;
  workflowIds: string[];
  required: boolean;
  installed: boolean;
  state: string;
  artifacts: SlicerModelArtifactStatus[];
}

export interface SlicerModelsStatus {
  manifestPath: string;
  cacheRoot: string;
  total: number;
  installed: number;
  requiredInstalled: boolean;
  models: SlicerModelStatus[];
}

export interface SlicerFixtureStatusItem {
  id: string;
  label: string;
  modality?: string | null;
  format?: string | null;
  source?: string | null;
  url: string;
  target: string;
  expectedBytes?: number | null;
  expectedSha256?: string | null;
  present: boolean;
  bytes?: number | null;
  sha256?: string | null;
  validChecksum: boolean;
}

export interface SlicerFixturesStatus {
  manifestPath: string;
  cacheRoot: string;
  total: number;
  ready: boolean;
  fixtures: SlicerFixtureStatusItem[];
}

export interface SlicerClinicalJobRequest {
  caseId: string;
  workflowId: string;
  sourcePath: string;
  outputDir?: string | null;
  modelId?: string | null;
  options?: Record<string, unknown>;
}

export interface SlicerClinicalJobStatus {
  jobId: string;
  workflowId: string;
  state: SlicerClinicalState;
  progress: number;
  message: string;
  logs: string[];
  inputHandle?: string | null;
  outputArtifacts: Array<Record<string, unknown>>;
  modelStatus?: Record<string, unknown>;
  error?: string | null;
  updatedAt: number;
}

export async function slicerRuntimeStatus(): Promise<SlicerRuntimeStatus> {
  return ipc<void, SlicerRuntimeStatus>('slicer_runtime_status', undefined, { timeoutMs: 45_000 });
}

export async function slicerRuntimeDownload(): Promise<Record<string, unknown>> {
  return ipc<void, Record<string, unknown>>('slicer_runtime_download', undefined, { timeoutMs: 900_000 });
}

export async function slicerModelsStatus(): Promise<SlicerModelsStatus> {
  return ipc<void, SlicerModelsStatus>('slicer_models_status', undefined, { timeoutMs: 45_000 });
}

export async function slicerFixturesStatus(): Promise<SlicerFixturesStatus> {
  return ipc<void, SlicerFixturesStatus>('slicer_fixtures_status', undefined, { timeoutMs: 45_000 });
}

export async function slicerFixturesDownload(fixtureId: string): Promise<Record<string, unknown>> {
  return ipc<{ request: { fixtureId: string } }, Record<string, unknown>>(
    'slicer_fixtures_download',
    { request: { fixtureId } },
    { timeoutMs: 900_000 },
  );
}

export async function slicerModelsDownloadAll(includeOptional = true): Promise<Record<string, unknown>> {
  return ipc<{ request: { includeOptional: boolean } }, Record<string, unknown>>(
    'slicer_models_download_all',
    { request: { includeOptional } },
    { timeoutMs: 900_000 },
  );
}

export async function slicerClinicalJobStart(request: SlicerClinicalJobRequest): Promise<SlicerClinicalJobStatus> {
  return ipc<{ request: Record<string, unknown> }, SlicerClinicalJobStatus>(
    'slicer_clinical_job_start',
    { request: request as unknown as Record<string, unknown> },
    { timeoutMs: 900_000 },
  );
}

export async function slicerClinicalJobStatus(jobId: string): Promise<SlicerClinicalJobStatus> {
  return ipc<{ jobId: string }, SlicerClinicalJobStatus>('slicer_clinical_job_status', { jobId });
}

export async function slicerClinicalJobCancel(jobId: string): Promise<SlicerClinicalJobStatus> {
  return ipc<{ jobId: string }, SlicerClinicalJobStatus>('slicer_clinical_job_cancel', { jobId });
}
