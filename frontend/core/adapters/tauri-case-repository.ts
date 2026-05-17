import type { ClinicalAsset, DentalCase } from '../domain/entities'
import type { CaseCreateInput, CaseRepository } from '../ports/case-repository'

type InvokeArgs = Record<string, unknown>

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

export class TauriCaseRepository implements CaseRepository {
  async create(input: CaseCreateInput = {}): Promise<DentalCase> {
    return invokeCommand<DentalCase>('case_create', { request: input })
  }

  async findById(caseId: string): Promise<DentalCase | null> {
    return invokeCommand<DentalCase | null>('case_get_graph', { caseId })
  }

  async list(): Promise<readonly DentalCase[]> {
    return invokeCommand<DentalCase[]>('case_list')
  }

  async save(dentalCase: DentalCase): Promise<DentalCase> {
    return invokeCommand<DentalCase>('case_save', { dentalCase })
  }

  async saveAsset(caseId: string, asset: ClinicalAsset): Promise<ClinicalAsset> {
    return invokeCommand<ClinicalAsset>('case_save_asset', { caseId, asset })
  }
}
