import type {
  AssetImportPreparation,
  AssetImportPrepareRequest,
  CadCoreBootstrap,
  CadCoreBootstrapRequest,
  CadCoreOrchestrator,
  CaseOpenResponse,
  ModulePermissionsRequest,
  ModulePermissionsResponse,
} from '../ports/cad-core-orchestrator'
import type { CadJobRequest, CadJobStatus } from '../domain/cad-core-platform'

type InvokeArgs = Record<string, unknown>

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

export class TauriCadCoreOrchestrator implements CadCoreOrchestrator {
  async bootstrap(request: CadCoreBootstrapRequest = {}): Promise<CadCoreBootstrap> {
    return invokeCommand<CadCoreBootstrap>('cad_bootstrap', { request })
  }

  async openCase(caseId: string): Promise<CaseOpenResponse> {
    return invokeCommand<CaseOpenResponse>('case_open', { caseId })
  }

  async prepareAssetImport(request: AssetImportPrepareRequest): Promise<AssetImportPreparation> {
    return invokeCommand<AssetImportPreparation>('asset_import_prepare', { request })
  }

  async startJob(request: CadJobRequest): Promise<CadJobStatus> {
    return invokeCommand<CadJobStatus>('cad_job_start', { request })
  }

  async getJobStatus(jobId: string): Promise<CadJobStatus> {
    return invokeCommand<CadJobStatus>('cad_job_status', { jobId })
  }

  async cancelJob(jobId: string): Promise<CadJobStatus> {
    return invokeCommand<CadJobStatus>('cad_job_cancel', { jobId })
  }

  async getModulePermissions(request: ModulePermissionsRequest): Promise<ModulePermissionsResponse> {
    return invokeCommand<ModulePermissionsResponse>('module_permissions_get', { request })
  }
}
