import type { AssetStorage, AssetWriteRequest, StoredAssetRef } from '../ports/asset-storage'

type InvokeArgs = Record<string, unknown>

export interface TauriStoredAssetRef extends StoredAssetRef {
  checksumSha256: string
}

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

export class TauriAssetStorage implements AssetStorage {
  async writeAsset(request: AssetWriteRequest): Promise<TauriStoredAssetRef> {
    return invokeCommand<TauriStoredAssetRef>('asset_write', {
      request: {
        ...request,
        bytes: Array.from(request.bytes),
      },
    })
  }

  async readAsset(ref: StoredAssetRef): Promise<Uint8Array> {
    const bytes = await invokeCommand<number[]>('asset_read', { localPath: ref.localPath })
    return Uint8Array.from(bytes)
  }

  async revealAssetFolder(ref: StoredAssetRef): Promise<void> {
    const opener = await import('@tauri-apps/plugin-opener')
    await opener.revealItemInDir(ref.localPath)
  }
}
