import type { CadCoreAssetKind, CadCoreModuleId } from './cad-core-platform'

export type MeshVaultJobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'manual-review'
export type MeshVaultFormat = 'stl' | 'obj' | 'ply' | 'gltf' | 'glb' | 'dicom' | '3mf' | string

export interface MeshVaultGpuHints {
  preferredUpload: 'array-buffer-slice'
  indexType: 'uint16' | 'uint32' | 'source'
  interleaved: boolean
  lodReady: boolean
  renderUsage: 'static-draw-after-worker-parse'
}

export interface MeshVaultHandle {
  meshKey: string
  caseId: string
  assetId: string
  kind: CadCoreAssetKind
  format: MeshVaultFormat
  storagePath: string
  checksumSha256: string
  bytes: number
  chunkSizeBytes: number
  chunkCount: number
  ttl: 'default' | 'extended' | string
  gpuHints: MeshVaultGpuHints
}

export interface MeshVaultImportRequest {
  caseId: string
  assetId?: string
  sourcePath: string
  kind: CadCoreAssetKind
  moduleId?: CadCoreModuleId
  role?: string
  chunkSizeBytes?: number
  ttl?: 'default' | 'extended' | string
  metadata?: Record<string, unknown>
}

export interface MeshVaultJobSnapshot {
  jobId: string
  status: MeshVaultJobStatus
  progress: number
  stage: string
  handle?: MeshVaultHandle | null
  error?: string | null
}

export interface MeshVaultProgressEvent {
  jobId: string
  caseId: string
  status: MeshVaultJobStatus
  progress: number
  stage: string
  bytesWritten: number
  totalBytes: number
  meshKey?: string | null
  error?: string | null
}

export function isMeshVaultComplete(snapshot: MeshVaultJobSnapshot): boolean {
  return snapshot.status === 'completed' && Boolean(snapshot.handle?.meshKey)
}

export function estimateMeshVaultMemoryCeiling(handle: MeshVaultHandle): number {
  return Math.min(handle.chunkSizeBytes, handle.bytes)
}
