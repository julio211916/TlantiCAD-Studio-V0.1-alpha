import type {
  MeshVaultHandle,
  MeshVaultImportRequest,
  MeshVaultJobSnapshot,
  MeshVaultProgressEvent,
} from '../domain/mesh-vault'

export type MeshVaultProgressUnsubscribe = () => void

export interface MeshVault {
  importFromPath(request: MeshVaultImportRequest): Promise<MeshVaultJobSnapshot>
  getJobStatus(jobId: string): Promise<MeshVaultJobSnapshot>
  cancel(jobId: string): Promise<MeshVaultJobSnapshot>
  find(meshKey: string): Promise<MeshVaultHandle | null>
  onProgress(listener: (event: MeshVaultProgressEvent) => void): Promise<MeshVaultProgressUnsubscribe>
}
