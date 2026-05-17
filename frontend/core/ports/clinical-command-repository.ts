import type { TlantiCadProductModuleId } from '../domain/cad-product-module-registry'

export type ClinicalCommandEventType = 'record' | 'undo' | 'redo'

export interface ClinicalCommandEvent {
  id: string
  caseId?: string | null
  moduleId: TlantiCadProductModuleId
  toolId: string
  eventType: ClinicalCommandEventType
  commandJson: string
  inverseCommandJson?: string | null
  targetEventId?: string | null
  assetIdsJson: string
  createdAt: string
}

export interface ClinicalCommandRecordRequest {
  id?: string
  caseId?: string | null
  moduleId: TlantiCadProductModuleId
  toolId: string
  commandJson?: string
  inverseCommandJson?: string | null
  targetEventId?: string | null
  assetIdsJson?: string
}

export interface ClinicalCommandRepository {
  record(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent>
  undo(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent>
  redo(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent>
  get(eventId: string): Promise<ClinicalCommandEvent>
}
