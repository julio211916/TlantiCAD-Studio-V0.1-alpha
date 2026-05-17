import type { FileData } from '../../types'
import type { LocalImagingEnginePort, LocalImagingSeriesManifest } from '../adapters/volview-local-imaging-engine'
import type { ImportPathBackedAssetRequest, MeshVaultImportUseCase } from './mesh-vault-import-use-case'

export interface ImportDicomSeriesWorkflowRequest {
  sourcePath: string
  caseId?: string
  assetId?: string
  displayName?: string
}

export interface ImportDicomSeriesWorkflowResult {
  imaging: LocalImagingSeriesManifest
  asset: FileData
  jobId: string
  stage: string
}

export class ImagingWorkflowUseCase {
  constructor(
    private readonly imagingEngine: LocalImagingEnginePort,
    private readonly meshVaultImport: MeshVaultImportUseCase,
  ) {}

  async importDicomSeries(request: ImportDicomSeriesWorkflowRequest): Promise<ImportDicomSeriesWorkflowResult> {
    const imaging = await this.imagingEngine.openSeries({
      sourcePath: request.sourcePath,
      caseId: request.caseId,
      modalityHint: 'dicom',
    })

    const importRequest: ImportPathBackedAssetRequest = {
      sourcePath: request.sourcePath,
      caseId: request.caseId ?? 'unassigned-case',
      kind: 'dicom-series',
    }
    if (request.assetId) importRequest.assetId = request.assetId
    if (request.displayName) importRequest.displayName = request.displayName
    const imported = await this.meshVaultImport.execute(importRequest)

    return {
      imaging,
      asset: imported.file,
      jobId: imported.jobId,
      stage: imported.stage,
    }
  }
}
