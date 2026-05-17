import type {
  ClinicalCommandEvent,
  ClinicalCommandRecordRequest,
  ClinicalCommandRepository,
} from '../ports/clinical-command-repository'

type InvokeArgs = Record<string, unknown>

async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

export class TauriClinicalCommandRepository implements ClinicalCommandRepository {
  async record(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent> {
    return invokeCommand<ClinicalCommandEvent>('clinical_command_record', { request })
  }

  async undo(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent> {
    return invokeCommand<ClinicalCommandEvent>('clinical_command_undo', { request })
  }

  async redo(request: ClinicalCommandRecordRequest): Promise<ClinicalCommandEvent> {
    return invokeCommand<ClinicalCommandEvent>('clinical_command_redo', { request })
  }

  async get(eventId: string): Promise<ClinicalCommandEvent> {
    return invokeCommand<ClinicalCommandEvent>('clinical_command_event_get', { eventId })
  }
}
