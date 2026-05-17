import type {
  MeshVaultHandle,
  MeshVaultImportRequest,
  MeshVaultJobSnapshot,
  MeshVaultProgressEvent,
} from '../domain/mesh-vault'
import type { MeshVault, MeshVaultProgressUnsubscribe } from '../ports/mesh-vault'
import { IPC_COMMANDS, ipc } from '../../lib/ipc'
import type {
  CmdMeshVaultCancelArgs,
  CmdMeshVaultFindArgs,
  CmdMeshVaultImportStartArgs,
  CmdMeshVaultJobStatusArgs,
  JsonObject,
} from '../../lib/ipc'

type InvokeArgs = Record<string, unknown>

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  return ipc<InvokeArgs, T>(command, args)
}

export class TauriMeshVault implements MeshVault {
  async importFromPath(request: MeshVaultImportRequest): Promise<MeshVaultJobSnapshot> {
    return invokeCommand<MeshVaultJobSnapshot>(IPC_COMMANDS.meshVaultImportStart, { request: request as unknown as JsonObject } satisfies CmdMeshVaultImportStartArgs)
  }

  async getJobStatus(jobId: string): Promise<MeshVaultJobSnapshot> {
    return invokeCommand<MeshVaultJobSnapshot>(IPC_COMMANDS.meshVaultJobStatus, { jobId } satisfies CmdMeshVaultJobStatusArgs)
  }

  async cancel(jobId: string): Promise<MeshVaultJobSnapshot> {
    return invokeCommand<MeshVaultJobSnapshot>(IPC_COMMANDS.meshVaultCancel, { jobId } satisfies CmdMeshVaultCancelArgs)
  }

  async find(meshKey: string): Promise<MeshVaultHandle | null> {
    return invokeCommand<MeshVaultHandle | null>(IPC_COMMANDS.meshVaultFind, { request: { meshKey } } satisfies CmdMeshVaultFindArgs)
  }

  async onProgress(listener: (event: MeshVaultProgressEvent) => void): Promise<MeshVaultProgressUnsubscribe> {
    const { listen } = await import('@tauri-apps/api/event')
    const unlisten = await listen<MeshVaultProgressEvent>('mesh-vault://job-progress', (event) => {
      listener(event.payload)
    })

    return unlisten
  }
}
