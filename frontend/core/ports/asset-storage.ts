import type { ClinicalAsset } from '../domain/entities'

export interface AssetWriteRequest {
  caseId: string
  asset: ClinicalAsset
  bytes: Uint8Array
}

export interface StoredAssetRef {
  assetId: string
  caseId: string
  localPath: string
  bytes: number
}

export interface AssetStorage {
  writeAsset(request: AssetWriteRequest): Promise<StoredAssetRef>
  readAsset(ref: StoredAssetRef): Promise<Uint8Array>
  revealAssetFolder?(ref: StoredAssetRef): Promise<void>
}
