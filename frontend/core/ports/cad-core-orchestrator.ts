import type {
  AssetManifest,
  CadCoreDependency,
  CadCoreModuleId,
  CadCoreServiceContract,
  CadJobRequest,
  CadJobStatus,
  CaseFolderLayout,
  ModuleManifest,
  ToolRegistry,
} from '../domain/cad-core-platform'
import type { DentalCase } from '../domain/entities'

export interface CadCoreBootstrapRequest {
  caseId?: string | null
  moduleId?: CadCoreModuleId | string | null
}

export interface CadCoreBootstrap {
  offlineRequired: true
  storage: {
    database: string
    caseRoot: string
  }
  toolRegistry: ToolRegistry
  dependencies: readonly CadCoreDependency[]
  serviceContracts: readonly CadCoreServiceContract[]
  commands: readonly string[]
}

export interface CaseOpenResponse {
  dentalCase: DentalCase | null
  layout: CaseFolderLayout
}

export interface AssetImportPrepareRequest {
  caseId: string
  assetId: string
  fileName: string
  kind: AssetManifest['kind']
  moduleId?: CadCoreModuleId | string | null
}

export interface AssetImportPreparation {
  caseId: string
  assetId: string
  targetDirectory: string
  targetPath: string
  manifestPath: string
  allowedExtensions: readonly string[]
}

export interface ModulePermissionsRequest {
  moduleId: CadCoreModuleId | string
  features: readonly string[]
  role?: string | null
  installedDependencies?: readonly string[]
}

export interface ModulePermissionDecision {
  permission: string
  expression: string
  allowed: boolean
  reason: string
}

export interface ModulePermissionsResponse {
  module: ModuleManifest
  decisions: readonly ModulePermissionDecision[]
}

export interface CadCoreOrchestrator {
  bootstrap(request?: CadCoreBootstrapRequest): Promise<CadCoreBootstrap>
  openCase(caseId: string): Promise<CaseOpenResponse>
  prepareAssetImport(request: AssetImportPrepareRequest): Promise<AssetImportPreparation>
  startJob(request: CadJobRequest): Promise<CadJobStatus>
  getJobStatus(jobId: string): Promise<CadJobStatus>
  cancelJob(jobId: string): Promise<CadJobStatus>
  getModulePermissions(request: ModulePermissionsRequest): Promise<ModulePermissionsResponse>
}
