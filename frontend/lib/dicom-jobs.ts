import { ipc } from '@/lib/ipc';

export interface DicomSeriesManifest {
  manifestId: string;
  caseId: string;
  sourcePath: string;
  manifestPath: string;
  clinicalRole: string;
  engine: string;
  fileCount: number;
  bytes: number;
  studyUid: string;
  seriesUid: string;
  modality: string;
  rows: number;
  columns: number;
  anonymizationStatus: string;
  raw?: Record<string, unknown>;
}

export interface DicomJobStatus {
  jobId: string;
  kind: string;
  state: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  message: string;
  manifest?: DicomSeriesManifest | null;
  artifactPath?: string | null;
  error?: string | null;
  updatedAt: number;
}

export interface DicomSegmentationJobStatus extends DicomJobStatus {
  engine?: string;
  labelmapManifestPath?: string | null;
  segmentationArtifactPath?: string | null;
  meshArtifactPath?: string | null;
  clinicalExportAllowed?: boolean;
}

export interface DicomSeriesImportStartRequest {
  caseId: string;
  sourcePath: string;
  clinicalRole?: string;
}

export interface DicomImportPrepareFromPathRequest {
  caseId: string;
  sourcePath: string;
  clinicalRole?: string;
}

export interface DicomVolumeBuildStartRequest {
  caseId: string;
  seriesManifestPath: string;
}

export interface DicomSegmentationStartRequest {
  caseId: string;
  volumeId: string;
  targetLabel: string;
  lowerThreshold: number;
  upperThreshold: number;
}

export interface DicomSegmentationToMeshStartRequest {
  caseId: string;
  segmentationJobId: string;
}

interface RawDicomSegmentationJobStatus {
  jobId: string;
  state: DicomJobStatus['state'];
  progress: number;
  engine: string;
  message: string;
  labelmapManifestPath?: string | null;
  meshArtifactPath?: string | null;
  clinicalExportAllowed: boolean;
  error?: string | null;
  updatedAt: number;
  raw?: Record<string, unknown> | null;
}

function normalizeSegmentationStatus(
  status: RawDicomSegmentationJobStatus,
  kind = 'dicom-segmentation',
): DicomSegmentationJobStatus {
  return {
    jobId: status.jobId,
    kind,
    state: status.state,
    progress: status.progress,
    message: status.message,
    manifest: null,
    artifactPath: status.meshArtifactPath ?? status.labelmapManifestPath ?? null,
    error: status.error ?? null,
    updatedAt: status.updatedAt,
    engine: status.engine,
    labelmapManifestPath: status.labelmapManifestPath ?? null,
    segmentationArtifactPath: null,
    meshArtifactPath: status.meshArtifactPath ?? null,
    clinicalExportAllowed: status.clinicalExportAllowed,
  };
}

export async function dicomSeriesImportStart(request: DicomSeriesImportStartRequest): Promise<DicomJobStatus> {
  return ipc<{ request: Record<string, unknown> }, DicomJobStatus>('dicom_series_import_start', {
    request: request as unknown as Record<string, unknown>,
  });
}

export async function dicomImportPrepareFromPath(request: DicomImportPrepareFromPathRequest): Promise<DicomJobStatus> {
  return ipc<{ request: Record<string, unknown> }, DicomJobStatus>('dicom_import_prepare_from_path', {
    request: request as unknown as Record<string, unknown>,
  });
}

export async function dicomSeriesJobStatus(jobId: string): Promise<DicomJobStatus> {
  return ipc<{ jobId: string }, DicomJobStatus>('dicom_series_job_status', { jobId });
}

export async function dicomSeriesImportCancel(jobId: string): Promise<DicomJobStatus> {
  return ipc<{ jobId: string }, DicomJobStatus>('dicom_series_import_cancel', { jobId });
}

export async function dicomVolumeBuildStart(request: DicomVolumeBuildStartRequest): Promise<DicomJobStatus> {
  return ipc<{ request: Record<string, unknown> }, DicomJobStatus>('dicom_volume_build_start', {
    request: request as unknown as Record<string, unknown>,
  });
}

export async function dicomVolumeJobStatus(jobId: string): Promise<DicomJobStatus> {
  return ipc<{ jobId: string }, DicomJobStatus>('dicom_volume_job_status', { jobId });
}

export async function dicomSegmentationStart(
  request: DicomSegmentationStartRequest,
): Promise<DicomSegmentationJobStatus> {
  const status = await ipc<{ request: Record<string, unknown> }, RawDicomSegmentationJobStatus>(
    'dicom_segmentation_start',
    { request: request as unknown as Record<string, unknown> },
  );
  return normalizeSegmentationStatus(status);
}

export async function dicomSegmentationJobStatus(jobId: string): Promise<DicomSegmentationJobStatus> {
  const status = await ipc<{ jobId: string }, RawDicomSegmentationJobStatus>('dicom_segmentation_job_status', { jobId });
  return normalizeSegmentationStatus(status);
}

export async function dicomSegmentationToMeshStart(
  request: DicomSegmentationToMeshStartRequest,
): Promise<DicomSegmentationJobStatus> {
  const status = await ipc<{ request: Record<string, unknown> }, RawDicomSegmentationJobStatus>(
    'dicom_segmentation_to_mesh_start',
    { request: request as unknown as Record<string, unknown> },
  );
  return normalizeSegmentationStatus(status, 'dicom-segmentation-to-mesh');
}

export async function dicomSegmentationCancel(jobId: string): Promise<DicomSegmentationJobStatus> {
  const status = await ipc<{ jobId: string }, RawDicomSegmentationJobStatus>('dicom_segmentation_cancel', { jobId });
  return normalizeSegmentationStatus(status);
}
