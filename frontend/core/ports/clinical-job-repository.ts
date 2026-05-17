export type ClinicalJobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'manual-review'

export interface ClinicalJobRecord {
  id: string
  caseId?: string | null
  kind: string
  status: ClinicalJobStatus
  progress: number
  vendor?: string | null
  modelId?: string | null
  checkpointSha256?: string | null
  paramsJson: string
  resultJson?: string | null
  error?: string | null
  createdAt: string
  updatedAt: string
}

export interface ClinicalJobRecordRequest {
  id?: string
  caseId?: string | null
  kind: string
  status?: ClinicalJobStatus
  progress?: number
  vendor?: string | null
  modelId?: string | null
  checkpointSha256?: string | null
  paramsJson?: string
  resultJson?: string | null
  error?: string | null
}

export interface ClinicalArtifactRecordRequest {
  id?: string
  jobId?: string | null
  caseId?: string | null
  assetId?: string | null
  artifactType: string
  storagePath?: string | null
  checksumSha256?: string | null
  metadataJson?: string
}

export interface ClinicalArtifactRecord {
  id: string
  jobId?: string | null
  caseId?: string | null
  assetId?: string | null
  artifactType: string
  storagePath?: string | null
  checksumSha256?: string | null
  metadataJson: string
  createdAt: string
}

export interface ClinicalJobRepository {
  record(request: ClinicalJobRecordRequest): Promise<ClinicalJobRecord>
  get(jobId: string): Promise<ClinicalJobRecord>
  list(caseId?: string | null): Promise<readonly ClinicalJobRecord[]>
  cancel(jobId: string): Promise<ClinicalJobRecord>
  recordArtifact(request: ClinicalArtifactRecordRequest): Promise<ClinicalArtifactRecord>
}
