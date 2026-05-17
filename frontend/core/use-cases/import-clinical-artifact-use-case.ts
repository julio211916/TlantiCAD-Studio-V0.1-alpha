import type { CadCoreModuleId } from '../domain/cad-core-platform'
import type { MeshVaultImportUseCase } from './mesh-vault-import-use-case'

export interface ImportClinicalArtifactRequest {
  caseId: string
  artifactPath: string
  displayName?: string
  moduleId?: CadCoreModuleId
  sourceJobId?: string
  artifactKind?: string
}

export class ImportClinicalArtifactUseCase {
  constructor(
    private readonly meshVaultImport: MeshVaultImportUseCase,
  ) {}

  async execute(request: ImportClinicalArtifactRequest) {
    return this.meshVaultImport.execute({
      caseId: request.caseId,
      sourcePath: request.artifactPath,
      kind: 'stl-mesh',
      moduleId: request.moduleId,
      role: 'clinical-segmentation-mesh',
      displayName: request.displayName,
      metadata: {
        sourceJobId: request.sourceJobId,
        artifactKind: request.artifactKind ?? 'stl',
        source: 'slicer-clinical-runtime',
      },
    })
  }
}
