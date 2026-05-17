import type { TlantiCadProductModuleId } from '../domain/cad-product-module-registry'
import type {
  ClinicalCommandEvent,
  ClinicalCommandRecordRequest,
  ClinicalCommandRepository,
} from '../ports/clinical-command-repository'
import type { PersistibleClinicalToolId } from './cad-clinical-tool-use-case'

export interface RecordClinicalCommandRequest {
  caseId?: string | null
  moduleId: TlantiCadProductModuleId
  toolId: PersistibleClinicalToolId
  command: Record<string, unknown>
  inverseCommand?: Record<string, unknown> | null
  targetEventId?: string | null
  assetIds?: readonly string[]
  eventId?: string
}

function buildClinicalCommandRequest(request: RecordClinicalCommandRequest): ClinicalCommandRecordRequest {
  return {
    id: request.eventId,
    caseId: request.caseId,
    moduleId: request.moduleId,
    toolId: request.toolId,
    commandJson: JSON.stringify(request.command),
    inverseCommandJson: request.inverseCommand ? JSON.stringify(request.inverseCommand) : null,
    targetEventId: request.targetEventId ?? null,
    assetIdsJson: JSON.stringify(request.assetIds ?? []),
  }
}

export class ClinicalCommandUseCase {
  constructor(private readonly commands: ClinicalCommandRepository) {}

  record(request: RecordClinicalCommandRequest): Promise<ClinicalCommandEvent> {
    return this.commands.record(buildClinicalCommandRequest(request))
  }

  undo(request: RecordClinicalCommandRequest): Promise<ClinicalCommandEvent> {
    return this.commands.undo(buildClinicalCommandRequest(request))
  }

  redo(request: RecordClinicalCommandRequest): Promise<ClinicalCommandEvent> {
    return this.commands.redo(buildClinicalCommandRequest(request))
  }
}
