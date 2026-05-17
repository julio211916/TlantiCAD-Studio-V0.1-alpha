import type { FileData } from '../../types'
import type { MeshVaultHandle, MeshVaultImportRequest, MeshVaultJobSnapshot } from '../domain/mesh-vault'
import type { MeshVault } from '../ports/mesh-vault'

export interface ImportPathBackedAssetRequest extends MeshVaultImportRequest {
  displayName?: string
}

export interface ImportPathBackedAssetResult {
  jobId: string
  status: MeshVaultJobSnapshot['status']
  stage: string
  file: FileData
}

export interface MeshVaultImportUseCaseOptions {
  pollIntervalMs?: number
  timeoutMs?: number
}

const DEFAULT_POLL_INTERVAL_MS = 500
const DEFAULT_TIMEOUT_MS = 5 * 60_000
const RUNNING_STATUSES = new Set<MeshVaultJobSnapshot['status']>(['queued', 'running'])
const TERMINAL_STATUSES = new Set<MeshVaultJobSnapshot['status']>(['completed', 'failed', 'cancelled'])

function fileTypeForMeshVaultKind(kind: MeshVaultImportRequest['kind']): FileData['type'] {
  if (kind === 'dicom-series') {
    return 'DICOM'
  }
  return 'MODEL'
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => globalThis.setTimeout(resolve, ms))
}

function buildMeshVaultMetadata(snapshot: MeshVaultJobSnapshot): NonNullable<FileData['metadata']> & {
  meshVaultJobId: string
  meshVaultJobStatus: MeshVaultJobSnapshot['status']
  meshVaultJobStage: string
} {
  return {
    vertices: 0,
    triangles: 0,
    volume: 0,
    area: 0,
    meshVaultJobId: snapshot.jobId,
    meshVaultJobStatus: snapshot.status,
    meshVaultJobStage: snapshot.stage,
  }
}

function createPathBackedFileData(
  request: ImportPathBackedAssetRequest,
  snapshot: MeshVaultJobSnapshot,
): FileData {
  const handle = snapshot.handle
  const name = request.displayName ?? request.sourcePath.split(/[\\/]/).filter(Boolean).at(-1) ?? request.kind
  return {
    id: request.assetId ?? handle?.assetId ?? `mesh-vault-${Date.now()}`,
    name,
    type: fileTypeForMeshVaultKind(request.kind),
    sourcePath: request.sourcePath,
    visible: true,
    opacity: 1,
    wireframe: false,
    position: [0, 0, 0],
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
    windowCenter: 40,
    windowWidth: 400,
    sliceIndex: 0,
    metadata: buildMeshVaultMetadata(snapshot),
    meshVault: handle
      ? {
          meshKey: handle.meshKey,
          checksumSha256: handle.checksumSha256,
          bytes: handle.bytes,
          chunkCount: handle.chunkCount,
          chunkSizeBytes: handle.chunkSizeBytes,
          storagePath: handle.storagePath,
          gpuHints: handle.gpuHints,
        }
      : undefined,
    dicomWorkspaceView: request.kind === 'dicom-series' ? 'review' : undefined,
  }
}

export class MeshVaultImportUseCase {
  private readonly pollIntervalMs: number
  private readonly timeoutMs: number

  constructor(
    private readonly meshVault: MeshVault,
    options: MeshVaultImportUseCaseOptions = {},
  ) {
    this.pollIntervalMs = options.pollIntervalMs ?? DEFAULT_POLL_INTERVAL_MS
    this.timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS
  }

  async execute(request: ImportPathBackedAssetRequest): Promise<ImportPathBackedAssetResult> {
    const started = await this.meshVault.importFromPath(request)
    const completed = await this.waitForTerminalSnapshot(started)
    return {
      jobId: completed.jobId,
      status: completed.status,
      stage: completed.stage,
      file: createPathBackedFileData(request, completed),
    }
  }

  private async waitForTerminalSnapshot(started: MeshVaultJobSnapshot): Promise<MeshVaultJobSnapshot> {
    let snapshot = started
    const deadline = Date.now() + this.timeoutMs

    while (RUNNING_STATUSES.has(snapshot.status)) {
      if (Date.now() >= deadline) {
        throw new Error(
          `Mesh Vault import timed out after ${this.timeoutMs}ms: job=${snapshot.jobId} status=${snapshot.status} stage=${snapshot.stage}`,
        )
      }

      await delay(Math.min(this.pollIntervalMs, Math.max(0, deadline - Date.now())))
      snapshot = await this.meshVault.getJobStatus(snapshot.jobId)
    }

    if (snapshot.status === 'completed' && snapshot.handle?.meshKey) {
      return snapshot
    }

    if (TERMINAL_STATUSES.has(snapshot.status)) {
      throw new Error(
        `Mesh Vault import ${snapshot.status}: job=${snapshot.jobId} stage=${snapshot.stage}${snapshot.error ? ` error=${snapshot.error}` : ''}`,
      )
    }

    throw new Error(
      `Mesh Vault import stopped in unsupported status: job=${snapshot.jobId} status=${snapshot.status} stage=${snapshot.stage}`,
    )
  }
}
