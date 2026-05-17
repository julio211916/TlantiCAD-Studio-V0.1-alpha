import type { ClinicalAsset, DentalCase, TlantiModuleId } from '../domain/entities'

export interface CaseCreateInput {
  caseNumber?: string
  name?: string
  activeModuleId?: TlantiModuleId
}

export interface CaseRepository {
  create?(input?: CaseCreateInput): Promise<DentalCase>
  findById(caseId: string): Promise<DentalCase | null>
  list(): Promise<readonly DentalCase[]>
  save(dentalCase: DentalCase): Promise<DentalCase>
  saveAsset?(caseId: string, asset: ClinicalAsset): Promise<ClinicalAsset>
}
